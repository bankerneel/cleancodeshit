//! Handle-level Windows filesystem safety fixtures (Milestone 2).
//!
//! Every mutation below is confined to a private `tempfile` tree. The fake
//! home is a directory *inside* that tree; the sentinel used by the junction
//! case is a sibling of the fake home, so crossing the approved-home boundary
//! is observable without ever going near a real profile. Nothing here needs
//! or attempts elevation.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::windows::fs::{OpenOptionsExt, symlink_file};
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

use super::*;

const SENTINEL_BYTES: &[u8] = b"outside-the-approved-home";

fn fixture_ok<T>(result: Result<T>, what: &str) -> T {
    result.unwrap_or_else(|error| panic!("{what}: {error:?}"))
}

/// Panic on success without requiring `FsError: Debug`.
fn fixture_err<T>(result: Result<T>, what: &str) -> FsError {
    match result {
        Ok(_) => panic!("{what} unexpectedly succeeded"),
        Err(err) => err,
    }
}

/// A fresh, exclusive temporary tree with a fake home inside it. The returned
/// `TempDir` must stay alive for the whole test.
fn fixture() -> (TempDir, PathBuf) {
    let temp = TempDir::new().expect("create exclusive fixture root");
    let home = temp.path().join("fake-home");
    fs::create_dir(&home).expect("create fake home");
    (temp, home)
}

fn write_fixture(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes)
        .unwrap_or_else(|err| panic!("fixture write at {} failed: {err}", path.display()));
}

/// Create a directory junction with the platform's `mklink /J`.
fn mk_junction(link: &Path, target: &Path) {
    let output = Command::new("cmd.exe")
        .args(["/C", "mklink", "/J"])
        .arg(link.as_os_str())
        .arg(target.as_os_str())
        .output()
        .expect("spawn cmd mklink /J");
    assert!(
        output.status.success(),
        "mklink /J failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Removing a Windows junction with `remove_dir` deletes only the reparse
/// point, never the entries in its target. Created before the fixture handles
/// and after the `TempDir`, so even a panic unwind drops the handles first and
/// removes the link before the temporary tree is torn down.
struct JunctionLink(PathBuf);

impl Drop for JunctionLink {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

#[test]
fn junction_escape_is_skipped_and_refused() {
    let (_temp, home) = fixture();
    let outside = home
        .parent()
        .expect("fake home has a parent")
        .join("sibling-home");
    fs::create_dir(&outside).expect("create sibling sentinel home");
    let sentinel = outside.join("sentinel.txt");
    write_fixture(&sentinel, SENTINEL_BYTES);
    write_fixture(&home.join("keep.txt"), b"inside-the-approved-home");

    let junction = home.join("escape");
    let _link = JunctionLink(junction.clone());
    mk_junction(&junction, &outside);

    let mut home_handle = fixture_ok(WindowsHome::open_fixture(&home), "open fixture");

    // The guarded walker must not surface the reparse point, but must still
    // report an ordinary entry in the pinned home.
    let names: Vec<OsString> = home_handle
        .root_mut()
        .entries()
        .expect("start root enumeration")
        .map(|entry| fixture_ok(entry, "enumerate root").name)
        .collect();
    let present = |name: &str| names.iter().any(|n| n.as_os_str() == OsStr::new(name));
    assert!(present("keep.txt"), "ordinary in-home entry disappeared");
    assert!(
        !present("escape"),
        "escaping junction leaked into enumeration"
    );

    // Opening the link directly must be refused.
    let err = fixture_err(
        home_handle.root().open_dir(OsStr::new("escape")),
        "open junction",
    );
    assert!(
        matches!(err, FsError::UnsafeEntry),
        "opening an escaping junction must be refused as an unsafe entry"
    );

    // Deleting the link directly must be refused, even when handed an
    // identity belonging to a real file inside the home.
    let identity = fixture_ok(
        home_handle.root().inspect(OsStr::new("keep.txt")),
        "inspect in-home file",
    )
    .identity;
    // The refusal may be reported as an unsafe entry or as an identity
    // mismatch; the required guarantee is that nothing outside the home is
    // removed, which the sentinel check below confirms.
    let result = home_handle
        .root()
        .delete_verified(OsStr::new("escape"), identity);
    assert!(
        result.is_err(),
        "deleting an escaping junction must be refused"
    );

    // The sentinel outside the approved home is untouched by every step.
    assert!(sentinel.is_file(), "sentinel disappeared");
    assert_eq!(
        fs::read(&sentinel).expect("read sentinel"),
        SENTINEL_BYTES,
        "sentinel outside the home was modified"
    );
}

#[test]
fn junction_inside_home_is_still_skipped() {
    let (_temp, home) = fixture();
    let directory = home.join("regular");
    fs::create_dir(&directory).expect("create directory");
    let junction = home.join("redirect");
    let _link = JunctionLink(junction.clone());
    mk_junction(&junction, &directory);

    let mut home = WindowsHome::open_fixture(&home).expect("pin fixture");
    assert!(matches!(
        home.root().open_dir(OsStr::new("redirect")),
        Err(FsError::UnsafeEntry)
    ));
    let entries = home
        .root_mut()
        .entries()
        .expect("enumerate fixture")
        .collect::<Result<Vec<_>>>()
        .expect("verify entries");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "regular");
}

#[test]
#[ignore = "requires the staged CI symlink or local Windows Developer Mode"]
fn file_symlink_is_skipped_and_refused() {
    let executable = std::env::current_exe().expect("locate test executable");
    let staged = executable.with_file_name("ccs-ci-links");
    let _local;
    let directory = if staged.is_dir() {
        staged
    } else {
        let (temp, directory) = fixture();
        write_fixture(&directory.join("target.txt"), b"symlink target");
        symlink_file(directory.join("target.txt"), directory.join("link.txt"))
            .expect("create fixture symlink with Windows Developer Mode");
        _local = temp;
        directory
    };
    let mut home = WindowsHome::open_fixture(&directory).expect("pin symlink fixture");
    let link = OsStr::new("link.txt");
    assert!(
        fs::symlink_metadata(directory.join(link))
            .unwrap()
            .is_symlink()
    );
    assert!(matches!(
        home.root().inspect(link),
        Err(FsError::UnsafeEntry)
    ));
    let identity = home
        .root()
        .inspect(OsStr::new("target.txt"))
        .unwrap()
        .identity;
    assert!(matches!(
        home.root().delete_verified(link, identity),
        Err(FsError::UnsafeEntry)
    ));
    let entries = home
        .root_mut()
        .entries()
        .unwrap()
        .collect::<Result<Vec<_>>>()
        .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "target.txt");
    assert!(
        fs::read(directory.join("target.txt"))
            .unwrap()
            .starts_with(b"symlink target")
    );
}

#[test]
fn offline_file_is_skipped_before_open() {
    let (_temp, home) = fixture();
    let file = home.join("offline.txt");
    let name = OsStr::new("offline.txt");
    write_fixture(&file, b"local fixture");
    let mut home = WindowsHome::open_fixture(&home).expect("pin fixture");
    let identity = home.root().inspect(name).unwrap().identity;

    let output = Command::new("attrib.exe")
        .arg("+O")
        .arg(&file)
        .output()
        .expect("set offline attribute");
    assert!(
        output.status.success(),
        "attrib +O failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(matches!(
        home.root().inspect(name),
        Err(FsError::UnsafeEntry)
    ));
    assert!(matches!(
        home.root().delete_verified(name, identity),
        Err(FsError::UnsafeEntry)
    ));
    assert!(home.root_mut().entries().unwrap().next().is_none());
    assert!(file.is_file());
}

#[test]
fn another_volume_identity_is_refused() {
    let (_temp, home) = fixture();
    let file = home.join("regular.txt");
    write_fixture(&file, b"local fixture");
    let home = WindowsHome::open_fixture(&home).unwrap();
    let name = OsStr::new("regular.txt");
    let mut identity = home.root().inspect(name).unwrap().identity;
    identity.volume ^= 1;
    assert!(matches!(
        home.root().delete_verified(name, identity),
        Err(FsError::Changed)
    ));
    assert!(matches!(
        read_metadata_on_volume(&home.root().pinned.handle, identity.volume),
        Err(FsError::UnsupportedFilesystem)
    ));
    assert!(file.is_file());
}

#[test]
fn held_directory_cannot_be_renamed() {
    let (_temp, home) = fixture();
    let stable = home.join("stable");
    fs::create_dir(&stable).expect("create stable directory");
    write_fixture(&stable.join("file.txt"), b"payload");

    let home_handle = fixture_ok(WindowsHome::open_fixture(&home), "open fixture");
    let held = fixture_ok(
        home_handle.root().open_dir(OsStr::new("stable")),
        "open stable directory",
    );

    // The open directory is held without delete sharing, so a concurrent
    // rename from outside must fail and the original must stay in place.
    let renamed = home.join("stable-renamed");
    let rename = fs::rename(&stable, &renamed);
    assert!(rename.is_err(), "held directory rename must fail");
    assert!(stable.is_dir(), "held directory was renamed away");
    assert!(!renamed.exists(), "rename target appeared despite refusal");

    // Releasing the handle removes the block.
    drop(held);
    drop(home_handle);
    fs::rename(&stable, &renamed).expect("rename succeeds after handle release");
    assert!(renamed.is_dir());
}

#[test]
fn stale_identity_refuses_swapped_file() {
    let (_temp, home) = fixture();
    let leaf = home.join("leaf.txt");
    let spare = home.join("spare.txt");
    write_fixture(&leaf, b"original");
    write_fixture(&spare, b"replacement");

    let home_handle = fixture_ok(WindowsHome::open_fixture(&home), "open fixture");
    let identity = fixture_ok(
        home_handle.root().inspect(OsStr::new("leaf.txt")),
        "inspect leaf",
    )
    .identity;

    // Both files existed at once, so their identities are distinct. Swapping
    // the spare into the leaf's name rules out file-id reuse as a false pass.
    fs::rename(&leaf, home.join("leaf-retired.txt")).expect("retire old leaf");
    fs::rename(&spare, &leaf).expect("swap in replacement leaf");

    let err = fixture_err(
        home_handle
            .root()
            .delete_verified(OsStr::new("leaf.txt"), identity),
        "delete with stale leaf identity",
    );
    assert!(
        matches!(err, FsError::Changed),
        "a swapped leaf must be reported as Changed"
    );
    assert_eq!(
        fs::read(&leaf).expect("read replacement leaf"),
        b"replacement",
        "stale-identity refusal must not delete the replacement"
    );
}

#[test]
fn stale_identity_refuses_swapped_directory() {
    let (_temp, home) = fixture();
    let holder = home.join("holder");
    fs::create_dir(&holder).expect("create holder directory");

    let home_handle = fixture_ok(WindowsHome::open_fixture(&home), "open fixture");
    let metadata = fixture_ok(
        home_handle.root().inspect(OsStr::new("holder")),
        "inspect holder",
    );
    assert!(matches!(metadata.kind, EntryKind::Directory));

    // Replace the whole directory with a fresh empty one at the same name.
    fs::rename(&holder, home.join("holder-retired")).expect("retire old holder");
    fs::create_dir(&holder).expect("create replacement holder");

    let err = fixture_err(
        home_handle
            .root()
            .delete_verified(OsStr::new("holder"), metadata.identity),
        "delete with stale parent identity",
    );
    assert!(
        matches!(err, FsError::Changed),
        "a swapped parent directory must be reported as Changed"
    );
    assert!(
        holder.is_dir(),
        "replacement directory must survive the refusal"
    );
}

#[test]
fn locked_leaf_is_safely_refused() {
    let (_temp, home) = fixture();
    let leaf = home.join("locked.txt");
    write_fixture(&leaf, b"in-use");

    let home_handle = fixture_ok(WindowsHome::open_fixture(&home), "open fixture");
    let identity = fixture_ok(
        home_handle.root().inspect(OsStr::new("locked.txt")),
        "inspect locked leaf",
    )
    .identity;

    // share_mode(0) is an exclusive Windows handle: no second opener,
    // including the adapter's verifying delete open, can succeed.
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true).share_mode(0);
    let held = options.open(&leaf).expect("take exclusive leaf handle");

    let err = fixture_err(
        home_handle
            .root()
            .delete_verified(OsStr::new("locked.txt"), identity),
        "delete locked leaf",
    );
    assert!(
        matches!(err, FsError::Io(ref io_err) if matches!(io_err.raw_os_error(), Some(5 | 32 | 33))),
        "an in-use leaf must surface a sharing/access Io error"
    );
    assert!(leaf.is_file(), "in-use leaf must not have been deleted");

    drop(held);

    // Positive control: once the lock is gone, verification succeeds.
    let identity = fixture_ok(
        home_handle.root().inspect(OsStr::new("locked.txt")),
        "re-inspect unlocked leaf",
    )
    .identity;
    fixture_ok(
        home_handle
            .root()
            .delete_verified(OsStr::new("locked.txt"), identity),
        "delete unlocked leaf",
    );
    assert!(!leaf.exists());
}

#[test]
fn verified_file_deletion_succeeds() {
    let (_temp, home) = fixture();
    let leaf = home.join("doomed.txt");
    write_fixture(&leaf, b"payload");

    let home_handle = fixture_ok(WindowsHome::open_fixture(&home), "open fixture");
    let metadata = fixture_ok(
        home_handle.root().inspect(OsStr::new("doomed.txt")),
        "inspect doomed file",
    );
    assert!(matches!(metadata.kind, EntryKind::File));
    fixture_ok(
        home_handle
            .root()
            .delete_verified(OsStr::new("doomed.txt"), metadata.identity),
        "delete verified file",
    );
    assert!(!leaf.exists(), "verified file was not deleted");
}

#[test]
fn verified_empty_directory_deletion_succeeds() {
    let (_temp, home) = fixture();
    let empty = home.join("empty-dir");
    fs::create_dir(&empty).expect("create empty directory");

    let home_handle = fixture_ok(WindowsHome::open_fixture(&home), "open fixture");
    let metadata = fixture_ok(
        home_handle.root().inspect(OsStr::new("empty-dir")),
        "inspect empty directory",
    );
    assert!(matches!(metadata.kind, EntryKind::Directory));
    fixture_ok(
        home_handle
            .root()
            .delete_verified(OsStr::new("empty-dir"), metadata.identity),
        "delete verified empty directory",
    );
    assert!(!empty.exists(), "verified empty directory was not deleted");
}

#[test]
fn nonempty_directory_deletion_is_refused() {
    let (_temp, home) = fixture();
    let nonempty = home.join("nonempty-dir");
    fs::create_dir(&nonempty).expect("create nonempty directory");
    write_fixture(&nonempty.join("child.txt"), b"child");

    let home_handle = fixture_ok(WindowsHome::open_fixture(&home), "open fixture");
    let metadata = fixture_ok(
        home_handle.root().inspect(OsStr::new("nonempty-dir")),
        "inspect nonempty directory",
    );
    assert!(matches!(metadata.kind, EntryKind::Directory));

    // The adapter may report this as an OS disposition error or as an unsafe
    // entry; the required guarantee is that nothing inside is removed.
    let result = home_handle
        .root()
        .delete_verified(OsStr::new("nonempty-dir"), metadata.identity);
    assert!(result.is_err(), "nonempty directory must be refused");
    assert!(
        nonempty.is_dir() && nonempty.join("child.txt").is_file(),
        "nonempty directory or its child was removed"
    );
}
