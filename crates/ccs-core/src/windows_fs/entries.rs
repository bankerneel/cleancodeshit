use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use super::{
    Directory, DirectoryEntry, FsError, Result, ensure_standard_user, entry_kind,
    is_unsafe_attributes, sys, validate_name,
};

/// A bounded iterator over entries verified against the open directory handle.
pub struct Entries<'a> {
    directory: &'a mut Directory,
    buffer: Box<sys::DirectoryBuffer>,
    offset: Option<usize>,
    restart: bool,
    finished: bool,
}

impl<'a> Entries<'a> {
    pub(super) fn new(directory: &'a mut Directory) -> Self {
        Self {
            directory,
            buffer: Box::new(sys::DirectoryBuffer([0; 32 * 1024])),
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
                    self.buffer.0.fill(0);
                    let handle = &self.directory.pinned.handle;
                    if !sys::enumerate(handle, &mut self.buffer, self.restart)? {
                        return Ok(None);
                    }
                    self.restart = false;
                    0
                }
            };
            let record = Record::parse(&self.buffer.0[offset..])?;
            self.offset = record.next.map(|next| offset + next);
            if is_unsafe_attributes(record.attributes) {
                continue;
            }
            let mut name: Vec<_> = record
                .name
                .chunks_exact(2)
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect();
            if validate_name(&name).is_err() {
                continue;
            }
            let actual = match self.directory.inspect_component(&mut name) {
                Ok(metadata) => metadata,
                Err(FsError::UnsafeEntry) => continue,
                Err(error) => return Err(error),
            };
            if actual.identity.id != record.id || actual.kind != entry_kind(record.attributes) {
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

struct Record<'a> {
    next: Option<usize>,
    attributes: u32,
    id: [u8; 16],
    name: &'a [u8],
}

impl<'a> Record<'a> {
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
