use std::ffi::OsString;
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::io::OwnedHandle;

use super::{
    Directory, DirectoryEntry, EntryMetadata, FileIdentity, FsError, Result, ensure_standard_user,
    entry_kind, is_unsafe_attributes, sys, validate_name,
};

const BATCH_SIZE: usize = 32 * 1024;

/// A bounded iterator over entries verified against the open directory handle.
pub struct Entries<'a> {
    directory: &'a mut Directory,
    buffer: Box<sys::DirectoryBuffer<BATCH_SIZE>>,
    written: usize,
    offset: Option<usize>,
    restart: bool,
    finished: bool,
}

impl<'a> Entries<'a> {
    pub(super) fn new(directory: &'a mut Directory) -> Self {
        Self {
            directory,
            buffer: Box::new(sys::DirectoryBuffer([0; BATCH_SIZE])),
            written: 0,
            offset: None,
            restart: true,
            finished: false,
        }
    }

    fn next_entry(&mut self) -> Result<Option<DirectoryEntry>> {
        loop {
            let offset = match self.offset {
                Some(offset) => offset,
                None => {
                    let handle = &self.directory.pinned.handle;
                    let Some(written) = sys::enumerate(handle, &mut self.buffer, self.restart)?
                    else {
                        return Ok(None);
                    };
                    self.written = written;
                    self.restart = false;
                    0
                }
            };
            let record = Record::parse(&self.buffer.0[offset..self.written])?;
            self.offset = record.next.map(|next| offset + next);
            let Some(expected) = record.metadata(self.directory.identity().volume) else {
                continue;
            };
            let name: Vec<_> = record
                .name
                .chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect();
            if validate_name(&name).is_err() {
                continue;
            }
            let actual = match self.directory.inspect_component(&name) {
                Ok(metadata) => metadata,
                Err(FsError::UnsafeEntry) => continue,
                Err(error) => return Err(error),
            };
            if actual != expected {
                return Err(FsError::Changed);
            }
            return Ok(Some(DirectoryEntry {
                name: OsString::from_wide(&name),
                identity: actual.identity,
                kind: actual.kind,
            }));
        }
    }
}

impl Iterator for Entries<'_> {
    type Item = Result<DirectoryEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let result = ensure_standard_user()
            .and_then(|()| self.directory.pinned.verify_chain())
            .and_then(|()| self.next_entry());
        self.finished = !matches!(result, Ok(Some(_)));
        result.transpose()
    }
}

impl std::iter::FusedIterator for Entries<'_> {}

/// Recall-on-open is visible in directory records, including exact-name
/// queries. This lookup leaves the sequential enumeration cursor untouched.
pub(super) fn lookup(parent: &OwnedHandle, name: &[u16], volume: u64) -> Result<EntryMetadata> {
    let mut buffer = sys::DirectoryBuffer([0; 1024]);
    let length = sys::lookup(parent, &mut buffer, name)?
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
    let record = Record::parse(&buffer.0[..length])?;
    if record.next.is_some() {
        return Err(FsError::InvalidDirectoryRecord);
    }
    record.metadata(volume).ok_or(FsError::UnsafeEntry)
}

struct Record<'a> {
    next: Option<usize>,
    attributes: u32,
    id: [u8; 16],
    name: &'a [u8],
}

impl<'a> Record<'a> {
    fn metadata(&self, volume: u64) -> Option<EntryMetadata> {
        if is_unsafe_attributes(self.attributes) {
            return None;
        }
        Some(EntryMetadata {
            identity: FileIdentity {
                volume,
                id: self.id,
            },
            kind: entry_kind(self.attributes),
        })
    }

    fn parse(bytes: &'a [u8]) -> Result<Self> {
        let header = bytes
            .get(..sys::NAME)
            .ok_or(FsError::InvalidDirectoryRecord)?;
        let u32_at = |offset| u32::from_le_bytes(header[offset..offset + 4].try_into().unwrap());
        let name_len = u32_at(sys::NAME_LENGTH) as usize;
        if name_len == 0 || name_len > 255 * 2 || name_len % 2 != 0 {
            return Err(FsError::InvalidDirectoryRecord);
        }
        let name_end = sys::NAME + name_len;
        let name = bytes
            .get(sys::NAME..name_end)
            .ok_or(FsError::InvalidDirectoryRecord)?;
        let next = u32_at(sys::NEXT) as usize;
        if next != 0 && (next < name_end || next % 8 != 0 || next > bytes.len() - sys::NAME) {
            return Err(FsError::InvalidDirectoryRecord);
        }
        Ok(Self {
            next: (next != 0).then_some(next),
            attributes: u32_at(sys::ATTRIBUTES),
            id: header[sys::ID..sys::ID + 16].try_into().unwrap(),
            name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_filename_truncated_at_reported_length() {
        let mut bytes = [0u8; 1024];
        bytes[sys::NAME_LENGTH..sys::NAME_LENGTH + 4].copy_from_slice(&4u32.to_le_bytes());
        let short_batch = &bytes[..sys::NAME + 2];
        assert!(matches!(
            Record::parse(short_batch),
            Err(FsError::InvalidDirectoryRecord)
        ));
    }

    #[test]
    fn refuses_offset_outside_reported_length_or_overlapping_current_record() {
        let mut bytes = [0u8; 1024];
        bytes[sys::NAME_LENGTH..sys::NAME_LENGTH + 4].copy_from_slice(&4u32.to_le_bytes());
        for invalid in [8u32, 513, 1016] {
            bytes[sys::NEXT..sys::NEXT + 4].copy_from_slice(&invalid.to_le_bytes());
            assert!(matches!(
                Record::parse(&bytes[..512]),
                Err(FsError::InvalidDirectoryRecord)
            ));
        }
    }
}
