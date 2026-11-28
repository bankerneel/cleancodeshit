//! Handle-relative access to one pinned Windows profile on local NTFS.
//!
//! This adapter handles one entry at a time. The planner must authorize each
//! name and retain its identity before requesting deletion.

mod entries;
#[allow(unsafe_code)]
mod sys;

#[cfg(test)]
mod tests_handles;
#[cfg(test)]
mod tests_policy;

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::OwnedHandle;
use std::sync::Arc;

use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_OFFLINE, FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS,
    FILE_ATTRIBUTE_RECALL_ON_OPEN, FILE_ATTRIBUTE_REPARSE_POINT,
};

pub use entries::Entries;

const BACKSLASH: u16 = b'\\' as u16;
const COLON: u16 = b':' as u16;
const DOT: u16 = b'.' as u16;
const SPACE: u16 = b' ' as u16;

pub type Result<T> = std::result::Result<T, FsError>;

#[derive(Debug)]
pub enum FsError {
    Elevated,
    Impersonating,
    InvalidProfile,
    UnsupportedFilesystem,
    InvalidName,
    UnsafeEntry,
    Changed,
    InvalidDirectoryRecord,
    Io(io::Error),
}

impl fmt::Display for FsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Elevated => "elevated Windows tokens cannot access the filesystem adapter",
            Self::Impersonating => "thread impersonation cannot access the filesystem adapter",
            Self::InvalidProfile => "the Windows profile is not an absolute local drive path",
            Self::UnsupportedFilesystem => "the entry must be on the profile's local NTFS volume",
            Self::InvalidName => "expected one unambiguous Windows filename component",
            Self::UnsafeEntry => "the entry is a reparse point or may recall offline data",
            Self::Changed => "the opened entry no longer matches its recorded identity",
            Self::InvalidDirectoryRecord => "Windows returned an invalid directory record",
            Self::Io(error) => return error.fmt(formatter),
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for FsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for FsError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileIdentity {
    volume: u64,
    id: [u8; 16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    File,
    Directory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntryMetadata {
    pub identity: FileIdentity,
    pub kind: EntryKind,
}

#[derive(Debug)]
pub struct DirectoryEntry {
    pub name: OsString,
    pub identity: FileIdentity,
    pub kind: EntryKind,
}

pub fn ensure_standard_user() -> Result<()> {
    sys::standard_user()
}

pub struct WindowsHome {
    root: Directory,
}

impl WindowsHome {
    /// Resolve the profile from Windows and pin its directory chain.
    pub fn open_current() -> Result<Self> {
        ensure_standard_user()?;
        Self::pin(&sys::profile()?)
    }

    #[cfg(test)]
    fn open_fixture(path: &std::path::Path) -> Result<Self> {
        ensure_standard_user()?;
        Self::pin(path.as_os_str())
    }

    fn pin(path: &OsStr) -> Result<Self> {
        let path: Vec<_> = path.encode_wide().collect();
        let [drive, COLON, BACKSLASH, remaining @ ..] = path.as_slice() else {
            return Err(FsError::InvalidProfile);
        };
        let drive = u8::try_from(*drive)
            .ok()
            .filter(u8::is_ascii_alphabetic)
            .ok_or(FsError::InvalidProfile)?;
        let components = remaining.split(|&unit| unit == BACKSLASH);
        for component in components.clone() {
            validate_name(component).map_err(|_| FsError::InvalidProfile)?;
        }

        let drive_root = sys::volume_root(drive)?;
        let metadata = read_metadata(&drive_root)?;
        if metadata.kind != EntryKind::Directory {
            return Err(FsError::InvalidProfile);
        }
        let mut current = Arc::new(PinnedDirectory {
            handle: drive_root,
            identity: metadata.identity,
            parent: None,
        });
        for component in components {
            current = PinnedDirectory::open(&current, component)?;
        }
        current.verify_chain()?;
        Ok(Self {
            root: Directory { pinned: current },
        })
    }

    pub fn root(&self) -> &Directory {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut Directory {
        &mut self.root
    }
}

struct PinnedDirectory {
    handle: OwnedHandle,
    identity: FileIdentity,
    parent: Option<Parent>,
}

struct Parent {
    directory: Arc<PinnedDirectory>,
    child_name: Box<[u16]>,
}

impl PinnedDirectory {
    fn open(parent: &Arc<Self>, name: &[u16]) -> Result<Arc<Self>> {
        let expected = parent.lookup(name)?;
        if expected.kind != EntryKind::Directory {
            return Err(FsError::Changed);
        }
        let handle = parent.open_verified(name, sys::Open::Directory, expected)?;
        Ok(Arc::new(Self {
            handle,
            identity: expected.identity,
            parent: Some(Parent {
                directory: Arc::clone(parent),
                child_name: name.into(),
            }),
        }))
    }

    fn lookup(&self, name: &[u16]) -> Result<EntryMetadata> {
        entries::lookup(&self.handle, name, self.identity.volume)
    }

    fn open_verified(
        &self,
        name: &[u16],
        access: sys::Open,
        expected: EntryMetadata,
    ) -> Result<OwnedHandle> {
        let handle = sys::open_child(&self.handle, name, access)?;
        if read_metadata_on_volume(&handle, self.identity.volume)? != expected {
            return Err(FsError::Changed);
        }
        Ok(handle)
    }

    fn verify_chain(&self) -> Result<()> {
        let mut directory = self;
        let mut child = None;
        loop {
            let actual = read_metadata(&directory.handle)?;
            if actual.identity != directory.identity || actual.kind != EntryKind::Directory {
                return Err(FsError::Changed);
            }
            if let Some((name, expected)) = child {
                if directory.lookup(name)? != expected {
                    return Err(FsError::Changed);
                }
            }
            let Some(parent) = &directory.parent else {
                return Ok(());
            };
            child = Some((parent.child_name.as_ref(), actual));
            directory = &parent.directory;
        }
    }
}

/// A directory handle which keeps its ancestors open without delete sharing.
pub struct Directory {
    pinned: Arc<PinnedDirectory>,
}

impl Directory {
    pub fn identity(&self) -> FileIdentity {
        self.pinned.identity
    }

    pub fn open_dir(&self, name: &OsStr) -> Result<Self> {
        let name = filename(name)?;
        ensure_standard_user()?;
        self.pinned.verify_chain()?;
        let pinned = PinnedDirectory::open(&self.pinned, &name)?;
        Ok(Self { pinned })
    }

    pub fn inspect(&self, name: &OsStr) -> Result<EntryMetadata> {
        let name = filename(name)?;
        ensure_standard_user()?;
        self.pinned.verify_chain()?;
        self.inspect_component(&name)
    }

    /// Mark the verified child handle for deletion. The OS decides when its
    /// filesystem disposition takes effect and refuses a nonempty directory.
    pub fn delete_verified(&self, name: &OsStr, expected: FileIdentity) -> Result<()> {
        let name = filename(name)?;
        ensure_standard_user()?;
        self.pinned.verify_chain()?;
        let preflight = self.pinned.lookup(&name)?;
        if preflight.identity != expected {
            return Err(FsError::Changed);
        }
        let handle = self
            .pinned
            .open_verified(&name, sys::Open::Delete, preflight)?;
        self.pinned.verify_chain()?;
        if self.pinned.lookup(&name)? != preflight {
            return Err(FsError::Changed);
        }
        // Attributes can change without a new file ID. Check again on the
        // verified handle immediately before setting its disposition.
        checked_attributes(&handle)?;
        sys::disposition(&handle)
    }

    /// Enumerate names and identities from the pinned directory. Unsafe entries
    /// are omitted. An error means the caller must treat this scan as incomplete.
    pub fn entries(&mut self) -> Result<Entries<'_>> {
        ensure_standard_user()?;
        self.pinned.verify_chain()?;
        Ok(Entries::new(self))
    }

    fn inspect_component(&self, name: &[u16]) -> Result<EntryMetadata> {
        let expected = self.pinned.lookup(name)?;
        self.pinned
            .open_verified(name, sys::Open::Inspect, expected)?;
        Ok(expected)
    }
}

fn read_metadata(handle: &OwnedHandle) -> Result<EntryMetadata> {
    let attributes = checked_attributes(handle)?;
    Ok(EntryMetadata {
        identity: sys::identity(handle)?,
        kind: entry_kind(attributes),
    })
}

fn checked_attributes(handle: &OwnedHandle) -> Result<u32> {
    let attributes = sys::attributes(handle)?;
    if is_unsafe_attributes(attributes) {
        return Err(FsError::UnsafeEntry);
    }
    Ok(attributes)
}

fn read_metadata_on_volume(handle: &OwnedHandle, volume: u64) -> Result<EntryMetadata> {
    let metadata = read_metadata(handle)?;
    if metadata.identity.volume != volume {
        return Err(FsError::UnsupportedFilesystem);
    }
    Ok(metadata)
}

fn entry_kind(attributes: u32) -> EntryKind {
    if attributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
        EntryKind::File
    } else {
        EntryKind::Directory
    }
}

fn is_unsafe_attributes(attributes: u32) -> bool {
    attributes
        & (FILE_ATTRIBUTE_REPARSE_POINT
            | FILE_ATTRIBUTE_OFFLINE
            | FILE_ATTRIBUTE_RECALL_ON_OPEN
            | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS)
        != 0
}

fn filename(name: &OsStr) -> Result<Vec<u16>> {
    let wide = name.encode_wide().collect::<Vec<_>>();
    validate_name(&wide)?;
    Ok(wide)
}

fn validate_name(name: &[u16]) -> Result<()> {
    if name.is_empty()
        || name.len() > 255
        || matches!(name.last(), Some(&SPACE | &DOT))
        || name.iter().any(|&unit| {
            unit < SPACE || u8::try_from(unit).is_ok_and(|byte| b"\"*/:<>?\\|".contains(&byte))
        })
        || is_dos_device(name)
    {
        return Err(FsError::InvalidName);
    }
    Ok(())
}

fn is_dos_device(name: &[u16]) -> bool {
    let stem = name.split(|&unit| unit == DOT).next().unwrap_or(name);
    let end = stem
        .iter()
        .rposition(|&unit| unit != SPACE)
        .map_or(0, |i| i + 1);
    let stem = &stem[..end];
    let ascii_eq = |prefix: &[u16], ascii: &[u8]| {
        prefix.len() == ascii.len()
            && prefix.iter().zip(ascii).all(|(&unit, &byte)| {
                u8::try_from(unit).is_ok_and(|unit| unit.eq_ignore_ascii_case(&byte))
            })
    };
    [
        b"CON".as_slice(),
        b"PRN",
        b"AUX",
        b"NUL",
        b"CONIN$",
        b"CONOUT$",
    ]
    .iter()
    .any(|device| ascii_eq(stem, device))
        || matches!(stem, [prefix @ .., 49..=57 | 0x00b9 | 0x00b2 | 0x00b3]
            if ascii_eq(prefix, b"COM") || ascii_eq(prefix, b"LPT"))
}
