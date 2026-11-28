//! The Win32 and NT calls behind the safe filesystem adapter.

use std::ffi::{OsStr, OsString};
use std::io;
use std::mem::{offset_of, size_of, size_of_val};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::ptr::{null, null_mut};

use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
use windows_sys::Wdk::Storage::FileSystem::{
    FILE_OPEN, FILE_OPEN_NO_RECALL, FILE_OPEN_REPARSE_POINT, FILE_STAT_INFORMATION,
    FILE_SYNCHRONOUS_IO_NONALERT, FileStatInformation, NtCreateFile, NtQueryInformationByName,
};
use windows_sys::Win32::Foundation::{
    ERROR_NO_MORE_FILES, ERROR_NO_TOKEN, HANDLE, INVALID_HANDLE_VALUE, RtlNtStatusToDosError,
    STATUS_REPARSE_POINT_ENCOUNTERED, UNICODE_STRING,
};
use windows_sys::Win32::Security::{
    CheckTokenMembership, CreateWellKnownSid, GetTokenInformation, IsWellKnownSid,
    SECURITY_MAX_SID_SIZE, TOKEN_ELEVATION, TOKEN_QUERY, TOKEN_USER, TokenElevation,
    TokenElevationType, TokenElevationTypeFull, TokenUser, WinBuiltinAdministratorsSid,
    WinLocalSystemSid,
};
use windows_sys::Win32::Storage::FileSystem::*;
use windows_sys::Win32::System::Com::CoTaskMemFree;
use windows_sys::Win32::System::IO::IO_STATUS_BLOCK;
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentThread, OpenProcessToken, OpenThreadToken,
};
use windows_sys::Win32::System::WindowsProgramming::{DRIVE_FIXED, DRIVE_REMOVABLE};
use windows_sys::Win32::UI::Shell::{FOLDERID_Profile, SHGetKnownFolderPath};

use super::{FileIdentity, FsError, Result, is_unsafe_attributes};

const OBJ_CASE_INSENSITIVE: u32 = 0x40;
const OBJ_DONT_REPARSE: u32 = 0x1000;

pub(super) fn standard_user() -> Result<()> {
    let mut token = null_mut();
    // SAFETY: The pseudo-handles are valid for the current process/thread. Each
    // output is a writable HANDLE and a successful call gives us ownership.
    if unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut token) } != 0 {
        drop(owned(token)?);
        return Err(FsError::Impersonating);
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() != Some(ERROR_NO_TOKEN as i32) {
        return Err(error.into());
    }
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error().into());
    }
    let token = owned(token)?;
    let mut elevation = TOKEN_ELEVATION::default();
    let mut elevation_type = 0i32;
    let mut returned = 0u32;
    // SAFETY: The information classes specify these output types and the sizes
    // describe the entire live, initialized stack outputs.
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenElevation,
            (&mut elevation as *mut TOKEN_ELEVATION).cast(),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        ) == 0
            || GetTokenInformation(
                token.as_raw_handle(),
                TokenElevationType,
                (&mut elevation_type as *mut i32).cast(),
                size_of::<i32>() as u32,
                &mut returned,
            ) == 0
    } {
        return Err(io::Error::last_os_error().into());
    }
    if elevation.TokenIsElevated != 0 || elevation_type == TokenElevationTypeFull {
        return Err(FsError::Elevated);
    }

    let mut user = [0usize; 16];
    // SAFETY: The buffer is pointer-aligned and large enough for TOKEN_USER and
    // its embedded SID. The resulting SID points into the buffer during use.
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            user.as_mut_ptr().cast(),
            size_of_val(&user) as u32,
            &mut returned,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    if unsafe {
        IsWellKnownSid(
            (*user.as_ptr().cast::<TOKEN_USER>()).User.Sid,
            WinLocalSystemSid,
        )
    } != 0
    {
        return Err(FsError::Elevated);
    }

    let mut admin_sid = [0u32; SECURITY_MAX_SID_SIZE as usize / size_of::<u32>()];
    let mut sid_size = size_of_val(&admin_sid) as u32;
    let sid = admin_sid.as_mut_ptr().cast();
    let mut is_admin = 0;
    // SAFETY: sid describes writable aligned storage for a well-known SID.
    // With a null token, CheckTokenMembership checks the effective token.
    if unsafe {
        CreateWellKnownSid(WinBuiltinAdministratorsSid, null_mut(), sid, &mut sid_size) == 0
            || CheckTokenMembership(null_mut(), sid, &mut is_admin) == 0
    } {
        return Err(io::Error::last_os_error().into());
    }
    if is_admin != 0 {
        return Err(FsError::Elevated);
    }
    Ok(())
}

pub(super) fn profile() -> Result<OsString> {
    let mut buffer = null_mut();
    // SAFETY: buffer is a writable out-pointer for the current user's known
    // profile. The allocation, if any, belongs to the caller.
    let status = unsafe { SHGetKnownFolderPath(&FOLDERID_Profile, 0, null_mut(), &mut buffer) };
    let path = if status < 0 {
        Err(io::Error::from_raw_os_error(status).into())
    } else if buffer.is_null() {
        Err(FsError::InvalidProfile)
    } else {
        // SAFETY: A successful call returns a NUL-terminated, allocated UTF-16
        // string, which we copy before freeing it.
        let units: Vec<_> = unsafe {
            (0..)
                .map(|i| *buffer.add(i))
                .take_while(|&unit| unit != 0)
                .collect()
        };
        Ok(OsString::from_wide(&units))
    };
    // SAFETY: SHGetKnownFolderPath uses the COM task allocator. Null is valid.
    unsafe { CoTaskMemFree(buffer.cast()) };
    path
}

pub(super) fn volume_root(letter: u8) -> Result<OwnedHandle> {
    let drive = wide_nul(OsStr::new(&format!("{}:\\", char::from(letter))));
    // SAFETY: drive is a NUL-terminated drive root.
    if !matches!(
        unsafe { GetDriveTypeW(drive.as_ptr()) },
        DRIVE_FIXED | DRIVE_REMOVABLE
    ) {
        return Err(FsError::UnsupportedFilesystem);
    }
    let root = wide_nul(OsStr::new(&format!("\\\\?\\{}:\\", char::from(letter))));
    // SAFETY: root is a NUL-terminated drive root. It is the only path-based
    // handle open. The actual handle is checked before any relative open.
    let handle = owned(unsafe {
        CreateFileW(
            root.as_ptr(),
            FILE_TRAVERSE | FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_OPEN_NO_RECALL,
            null_mut(),
        )
    })?;
    // A drive alias can name a subdirectory. Require the handle's name to
    // contain only the volume GUID and its root separator.
    let mut opened = [0u16; 64];
    // SAFETY: The root handle and bounded output storage are live.
    let length = unsafe {
        GetFinalPathNameByHandleW(
            handle.as_raw_handle(),
            opened.as_mut_ptr(),
            opened.len() as u32,
            VOLUME_NAME_GUID | FILE_NAME_OPENED,
        )
    } as usize;
    if length == 0 {
        return Err(io::Error::last_os_error().into());
    }
    let prefix = "\\\\?\\Volume{".encode_utf16().count();
    if length != prefix + 36 + 2
        || !opened[..prefix]
            .iter()
            .copied()
            .eq("\\\\?\\Volume{".encode_utf16())
        || !opened[length - 2..length]
            .iter()
            .copied()
            .eq("}\\".encode_utf16())
    {
        return Err(FsError::UnsupportedFilesystem);
    }
    let mut filesystem = [0u16; 32];
    // SAFETY: The root handle and bounded NUL-terminated output storage are live.
    if unsafe {
        GetVolumeInformationByHandleW(
            handle.as_raw_handle(),
            null_mut(),
            0,
            null_mut(),
            null_mut(),
            null_mut(),
            filesystem.as_mut_ptr(),
            filesystem.len() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    if filesystem[..5] != [b'N' as u16, b'T' as u16, b'F' as u16, b'S' as u16, 0] {
        return Err(FsError::UnsupportedFilesystem);
    }
    Ok(handle)
}

#[derive(Clone, Copy)]
pub(super) enum Open {
    Traverse,
    Directory,
    Inspect,
    Delete,
}

pub(super) fn open_child(
    parent: &OwnedHandle,
    name: &mut [u16],
    usage: Open,
) -> Result<OwnedHandle> {
    // Names have already been validated as a single component of <=255 units.
    let length = size_of_val(name) as u16;
    let name = UNICODE_STRING {
        Length: length,
        MaximumLength: length,
        Buffer: name.as_mut_ptr(),
    };
    let attributes = OBJECT_ATTRIBUTES {
        Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: parent.as_raw_handle(),
        ObjectName: &name,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE,
        ..Default::default()
    };
    // Query the attributes without opening the file. Enumeration also filters
    // them before reaching this point. The open still uses no recall and no
    // reparse flags and its actual handle is checked again after opening.
    let mut stat = FILE_STAT_INFORMATION::default();
    let mut status_block = IO_STATUS_BLOCK::default();
    // SAFETY: FileStatInformation specifies the live and writable stat type.
    // The parent, validated relative name and object attributes remain live.
    let status = unsafe {
        NtQueryInformationByName(
            &attributes,
            &mut status_block,
            (&mut stat as *mut FILE_STAT_INFORMATION).cast(),
            size_of_val(&stat) as u32,
            FileStatInformation,
        )
    };
    check_status(status)?;
    if is_unsafe_attributes(stat.FileAttributes) {
        return Err(FsError::UnsafeEntry);
    }
    let (access, sharing) = match usage {
        Open::Traverse => (FILE_TRAVERSE, FILE_SHARE_READ | FILE_SHARE_WRITE),
        Open::Directory => (
            FILE_TRAVERSE | FILE_LIST_DIRECTORY,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
        ),
        Open::Inspect => (0, FILE_SHARE_READ | FILE_SHARE_WRITE),
        Open::Delete => (DELETE, FILE_SHARE_READ),
    };
    let mut result = null_mut();
    // SAFETY: All pointers describe live inputs/outputs with the ABI layout
    // from windows-sys. The parent and name stay live for this synchronous
    // existing-file open. The object name is relative and reparsing is denied.
    let status = unsafe {
        NtCreateFile(
            &mut result,
            access | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            &attributes,
            &mut status_block,
            null(),
            0,
            sharing,
            FILE_OPEN,
            FILE_OPEN_NO_RECALL | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
            null(),
            0,
        )
    };
    check_status(status)?;
    owned(result)
}

fn check_status(status: i32) -> Result<()> {
    if status == STATUS_REPARSE_POINT_ENCOUNTERED {
        return Err(FsError::UnsafeEntry);
    }
    if status < 0 {
        // SAFETY: RtlNtStatusToDosError accepts any NTSTATUS value.
        return Err(
            io::Error::from_raw_os_error(unsafe { RtlNtStatusToDosError(status) } as i32).into(),
        );
    }
    Ok(())
}

pub(super) fn attributes(handle: &OwnedHandle) -> Result<u32> {
    let mut output = FILE_ATTRIBUTE_TAG_INFO::default();
    // SAFETY: The information class matches the live writable output structure.
    if unsafe {
        GetFileInformationByHandleEx(
            handle.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut output as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            size_of_val(&output) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    Ok(output.FileAttributes)
}

pub(super) fn identity(handle: &OwnedHandle) -> Result<FileIdentity> {
    let mut output = FILE_ID_INFO::default();
    // SAFETY: The information class matches the live writable output structure.
    if unsafe {
        GetFileInformationByHandleEx(
            handle.as_raw_handle(),
            FileIdInfo,
            (&mut output as *mut FILE_ID_INFO).cast(),
            size_of_val(&output) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    Ok(FileIdentity {
        volume: output.VolumeSerialNumber,
        id: output.FileId.Identifier,
    })
}

pub(super) fn disposition(handle: &OwnedHandle) -> Result<()> {
    let disposition = FILE_DISPOSITION_INFO { DeleteFile: true };
    // SAFETY: The verified handle has DELETE access and the information class
    // matches the immutable, live input. The OS refuses nonempty directories.
    if unsafe {
        SetFileInformationByHandle(
            handle.as_raw_handle(),
            FileDispositionInfo,
            (&disposition as *const FILE_DISPOSITION_INFO).cast(),
            size_of_val(&disposition) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error().into());
    }
    Ok(())
}

pub(super) const NEXT: usize = offset_of!(FILE_ID_EXTD_DIR_INFO, NextEntryOffset);
pub(super) const ATTRIBUTES: usize = offset_of!(FILE_ID_EXTD_DIR_INFO, FileAttributes);
pub(super) const NAME_LENGTH: usize = offset_of!(FILE_ID_EXTD_DIR_INFO, FileNameLength);
pub(super) const ID: usize = offset_of!(FILE_ID_EXTD_DIR_INFO, FileId);
pub(super) const NAME: usize = offset_of!(FILE_ID_EXTD_DIR_INFO, FileName);

#[repr(align(8))]
pub(super) struct DirectoryBuffer(pub [u8; 32 * 1024]);

pub(super) fn enumerate(
    handle: &OwnedHandle,
    buffer: &mut DirectoryBuffer,
    restart: bool,
) -> Result<bool> {
    let class = if restart {
        FileIdExtdDirectoryRestartInfo
    } else {
        FileIdExtdDirectoryInfo
    };
    // SAFETY: The live, mutable buffer has the alignment of the requested
    // record and the supplied length does not exceed its capacity.
    if unsafe {
        GetFileInformationByHandleEx(
            handle.as_raw_handle(),
            class,
            buffer.0.as_mut_ptr().cast(),
            buffer.0.len() as u32,
        )
    } != 0
    {
        return Ok(true);
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(ERROR_NO_MORE_FILES as i32) {
        return Ok(false);
    }
    Err(error.into())
}

fn owned(handle: HANDLE) -> Result<OwnedHandle> {
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error().into());
    }
    // SAFETY: Callers give us a freshly opened file or token handle. No other
    // owner calls CloseHandle and handle values that signal failure are rejected.
    Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
}

fn wide_nul(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain([0]).collect()
}
