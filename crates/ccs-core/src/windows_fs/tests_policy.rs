//! Milestone 2 safety fixtures for the Windows filesystem policy.
//!
//! Every mutation in this module is confined to an isolated `tempfile`
//! directory. The two process-policy tests are deliberately asymmetric:
//! `allows_standard_process` belongs to the ordinary standard-account run,
//! while `refuses_elevated_process` is `#[ignore]`d and is invoked explicitly
//! by the elevated CI job (`cargo test -- --ignored refuses_elevated_process`).

use super::*;

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

// Bit values mirrored from `windows-sys::Win32::Storage::FileSystem` (WinNT.h).
// They are kept local so these fixtures do not depend on which constants the
// implementation happens to import into the parent module's namespace.
const BIT_READONLY: u32 = 0x0000_0001;
const BIT_HIDDEN: u32 = 0x0000_0002;
const BIT_SYSTEM: u32 = 0x0000_0004;
const BIT_DIRECTORY: u32 = 0x0000_0010;
const BIT_ARCHIVE: u32 = 0x0000_0020;
const BIT_DEVICE: u32 = 0x0000_0040;
const BIT_NORMAL: u32 = 0x0000_0080;
const BIT_TEMPORARY: u32 = 0x0000_0100;
const BIT_SPARSE_FILE: u32 = 0x0000_0200;
const BIT_REPARSE_POINT: u32 = 0x0000_0400;
const BIT_COMPRESSED: u32 = 0x0000_0800;
const BIT_OFFLINE: u32 = 0x0000_1000;
const BIT_NOT_CONTENT_INDEXED: u32 = 0x0000_2000;
const BIT_ENCRYPTED: u32 = 0x0000_4000;
const BIT_INTEGRITY_STREAM: u32 = 0x0000_8000;
const BIT_VIRTUAL: u32 = 0x0001_0000;
const BIT_NO_SCRUB_DATA: u32 = 0x0002_0000;
// RECALL_ON_OPEN shares its bit with FILE_ATTRIBUTE_EA; both are unsafe here.
const BIT_RECALL_ON_OPEN: u32 = 0x0004_0000;
const BIT_PINNED: u32 = 0x0008_0000;
const BIT_UNPINNED: u32 = 0x0010_0000;
const BIT_RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;

fn expect_ok<T>(result: Result<T>, what: &str) -> T {
    result.unwrap_or_else(|error| panic!("{what}: {error:?}"))
}

fn expect_err<T>(result: Result<T>, what: &str) -> FsError {
    match result {
        Ok(_) => panic!("expected {what} to be rejected"),
        Err(error) => error,
    }
}

/// Prefix an absolute fixture path for `std::fs` fixture construction only.
/// All product-under-test opens stay handle-relative from a shallow root.
fn extended_path(path: &Path) -> PathBuf {
    let mut raw = OsString::from(r"\\?\");
    raw.push(path.as_os_str());
    PathBuf::from(raw)
}

// ---------------------------------------------------------------------------
// Process policy
// ---------------------------------------------------------------------------

/// Elevated-only fixture. Run explicitly in the elevated CI job; the normal
/// standard-account test run skips it via `#[ignore]`.
#[test]
#[ignore = "requires an elevated process; run explicitly in the elevated CI job"]
fn refuses_elevated_process() {
    assert!(matches!(ensure_standard_user(), Err(FsError::Elevated)));
    assert!(matches!(
        WindowsHome::open_current(),
        Err(FsError::Elevated)
    ));
}

/// Standard-account fixture. Performs only a read-only open of the trusted
/// current home: no real-home enumeration and no writes outside the tempdir.
#[test]
fn allows_standard_process() {
    expect_ok(
        ensure_standard_user(),
        "ensure_standard_user in a standard process",
    );
    let _home = expect_ok(
        WindowsHome::open_current(),
        "WindowsHome::open_current in a standard process",
    );
    let executable = std::env::current_exe().unwrap();
    if executable.file_name() == Some(OsStr::new("ccs-core-tests.exe")) {
        let expected = std::fs::read_to_string(executable.with_file_name("account.sid")).unwrap();
        let whoami = PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32")
            .join("whoami.exe");
        let output = std::process::Command::new(whoami)
            .args(["/user", "/fo", "csv", "/nh"])
            .output()
            .expect("query the hosted test account SID");
        assert!(output.status.success());
        let identity = String::from_utf8_lossy(&output.stdout);
        let (_, sid) = identity
            .trim_end()
            .rsplit_once(',')
            .expect("whoami CSV SID");
        assert_eq!(sid.trim_matches('"'), expected.trim(), "fixture token SID");

        let profile = std::fs::canonicalize(PathBuf::from(sys::profile().unwrap())).unwrap();
        let temp = std::fs::canonicalize(std::env::temp_dir()).unwrap();
        assert!(
            temp.starts_with(profile),
            "the test account must use its own profile for temporary files"
        );
    }
}

// ---------------------------------------------------------------------------
// Name policy on a fixture root
// ---------------------------------------------------------------------------

#[test]
fn fixture_rejects_dangerous_component_names() {
    let temp = tempfile::tempdir().expect("isolated fixture tempdir");
    std::fs::create_dir(temp.path().join("child")).expect("create child directory");
    std::fs::write(temp.path().join("target.txt"), b"data").expect("create target file");

    let home = expect_ok(WindowsHome::open_fixture(temp.path()), "open_fixture");
    let root = home.root();

    // Relative components, UNC/device paths, drive-relative paths and ADS.
    const BAD: &[&str] = &[
        "",
        ".",
        "..",
        "./child",
        ".\\child",
        "../child",
        "..\\child",
        "child/target.txt",
        "child\\target.txt",
        r"\\server\share\target.txt",
        r"\\?\C:\Windows",
        r"\\.\C:\Windows",
        "C:target.txt",
        "C:\\Windows\\System32",
        "d:relative",
        "target.txt:stream",
        "target.txt::$DATA",
        "nul.txt",
        "NUL .txt",
        "Com\u{b2}.txt",
    ];

    for &name in BAD {
        expect_err(
            root.open_dir(OsStr::new(name)),
            &format!("open_dir({name:?})"),
        );
        expect_err(
            root.inspect(OsStr::new(name)),
            &format!("inspect({name:?})"),
        );
    }
}

#[test]
fn fixture_rejects_suffix_aliases() {
    let temp = tempfile::tempdir().expect("isolated fixture tempdir");
    std::fs::write(temp.path().join("target"), b"data").expect("create target file");

    let home = expect_ok(WindowsHome::open_fixture(temp.path()), "open_fixture");
    let root = home.root();

    // Win32 strips trailing dots/spaces and interprets ':' as an alternate
    // data stream. Each alias must be refused before it reaches the OS.
    for alias in [
        "target.",
        "target..",
        "target ",
        "target  ",
        "target. ",
        "target .",
        "target::$DATA",
        "target:alt",
    ] {
        expect_err(
            root.inspect(OsStr::new(alias)),
            &format!("inspect({alias:?})"),
        );
        expect_err(
            root.open_dir(OsStr::new(alias)),
            &format!("open_dir({alias:?})"),
        );
    }

    // The plain name resolves, so the rejections above are a policy decision
    // and not simply a missing entry.
    expect_ok(root.inspect(OsStr::new("target")), "inspect(target)");
}

#[test]
fn fixture_accepts_plain_names_and_root_mut() {
    let temp = tempfile::tempdir().expect("isolated fixture tempdir");
    std::fs::create_dir(temp.path().join("子フォルダ")).expect("create child directory");
    std::fs::write(temp.path().join("メモ.txt"), b"data").expect("create file");

    let mut home = expect_ok(WindowsHome::open_fixture(temp.path()), "open_fixture");
    {
        let root = home.root_mut();
        expect_ok(
            root.open_dir(OsStr::new("子フォルダ")),
            "open_dir through root_mut",
        );
        expect_ok(
            root.inspect(OsStr::new("メモ.txt")),
            "inspect through root_mut",
        );
    }

    let root = home.root();
    expect_err(
        root.inspect(OsStr::new("存在しない")),
        "inspect missing entry",
    );
}

// ---------------------------------------------------------------------------
// Deep / Unicode / enumeration coverage
// ---------------------------------------------------------------------------

#[test]
fn fixture_traverses_deep_unicode_names_beyond_max_path() {
    let temp = tempfile::tempdir().expect("isolated fixture tempdir");

    // Build the deep tree with the extended-length prefix. This is the only
    // use of `\\?\`, strictly for fixture construction via std::fs; the code
    // under test only ever receives the shallow root and handle-relative
    // child names.
    let mut nested = extended_path(temp.path());
    let components: Vec<String> = (0..32).map(|i| format!("深い_階層_{i:02}")).collect();
    for component in &components {
        nested.push(component);
    }
    assert!(nested.as_os_str().encode_wide().count() > 260);
    std::fs::create_dir_all(&nested).expect("create deep unicode fixture");

    let leaf = "日本語_ファイル.txt";
    std::fs::write(nested.join(leaf), b"payload").expect("create deep unicode file");

    let home = expect_ok(WindowsHome::open_fixture(temp.path()), "open_fixture");
    let mut dir = expect_ok(
        home.root().open_dir(OsStr::new(&components[0])),
        "open first nested directory",
    );
    for component in &components[1..] {
        dir = expect_ok(
            dir.open_dir(OsStr::new(component)),
            "open handle-relative nested directory",
        );
    }

    expect_ok(dir.inspect(OsStr::new(leaf)), "inspect deep unicode leaf");

    let mut names = HashSet::new();
    for item in dir.entries().expect("start deep directory scan") {
        names.insert(expect_ok(item, "deep directory entry").name);
    }
    assert!(
        names.contains(&OsString::from(leaf)),
        "deep unicode leaf was not enumerated: {names:?}"
    );
}

#[test]
fn fixture_enumerates_many_unicode_entries_across_buffers() {
    let temp = tempfile::tempdir().expect("isolated fixture tempdir");
    let holder = temp.path().join("多数");
    std::fs::create_dir(&holder).expect("create holder directory");

    let mut expected = HashSet::new();
    for i in 0..512u32 {
        let name = format!("資料_{i:03}_终端.dat");
        std::fs::write(holder.join(&name), b"x").expect("create enumerated file");
        expected.insert(OsString::from(name));
    }

    let home = expect_ok(WindowsHome::open_fixture(temp.path()), "open_fixture");
    let mut dir = expect_ok(home.root().open_dir(OsStr::new("多数")), "open holder");

    let mut names = HashSet::new();
    for item in dir.entries().expect("start multi-buffer scan") {
        names.insert(expect_ok(item, "enumerated entry").name);
    }

    for name in &expected {
        assert!(names.contains(name), "missing enumerated entry {name:?}");
    }
    assert!(
        names.len() >= expected.len(),
        "enumeration stopped early: {} of {} entries",
        names.len(),
        expected.len()
    );
}

#[test]
fn case_alias_resolves_to_the_verified_identity() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("MiXeD.txt");
    std::fs::write(&path, b"fixture").unwrap();
    let home = WindowsHome::open_fixture(temp.path()).unwrap();
    let exact = home.root().inspect(OsStr::new("MiXeD.txt")).unwrap();
    let alias = home.root().inspect(OsStr::new("mixed.TXT")).unwrap();
    assert_eq!(exact, alias);
    home.root()
        .delete_verified(OsStr::new("MIXED.txt"), exact.identity)
        .unwrap();
    assert!(!path.exists());
}

// ---------------------------------------------------------------------------
// Synthetic placeholder attributes (no cloud provider involved)
// ---------------------------------------------------------------------------

#[test]
fn unsafe_attributes_flags_placeholder_policy_bits() {
    let benign = BIT_READONLY
        | BIT_HIDDEN
        | BIT_SYSTEM
        | BIT_DIRECTORY
        | BIT_ARCHIVE
        | BIT_DEVICE
        | BIT_NORMAL
        | BIT_TEMPORARY
        | BIT_SPARSE_FILE
        | BIT_COMPRESSED
        | BIT_NOT_CONTENT_INDEXED
        | BIT_ENCRYPTED
        | BIT_INTEGRITY_STREAM
        | BIT_VIRTUAL
        | BIT_NO_SCRUB_DATA
        | BIT_PINNED
        | BIT_UNPINNED;

    assert!(!is_unsafe_attributes(0), "zero attributes must be safe");
    assert!(
        !is_unsafe_attributes(benign),
        "ordinary attributes must be safe"
    );

    for bit in [
        BIT_REPARSE_POINT,
        BIT_OFFLINE,
        BIT_RECALL_ON_OPEN,
        BIT_RECALL_ON_DATA_ACCESS,
    ] {
        assert!(is_unsafe_attributes(bit), "bit {bit:#010x} must be unsafe");
        assert!(
            is_unsafe_attributes(benign | bit),
            "benign attributes plus {bit:#010x} must be unsafe"
        );
    }

    assert!(is_unsafe_attributes(
        BIT_REPARSE_POINT | BIT_OFFLINE | BIT_RECALL_ON_OPEN | BIT_RECALL_ON_DATA_ACCESS
    ));
}
