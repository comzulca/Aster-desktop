//! Dormant creation-journal privacy, not a filesystem/ACL IPC capability.
//! Only new exact app objects receive descriptors; existing ACLs are never repaired.

#![allow(dead_code)]

use crate::{new_file::Identity, workspace::WorkspaceError};
use std::{
    ffi::c_void,
    fs::File,
    mem::MaybeUninit,
    os::windows::{ffi::OsStrExt, io::AsRawHandle, io::FromRawHandle},
    path::Path,
    ptr,
};

const MAX_DESCRIPTOR: usize = 16 * 1024;
const MAX_TOKEN: usize = 4096;
const FILE_ALL_ACCESS: u32 = 0x001f_01ff;
const PROTECTED_DACL: u16 = 0x1000;
const DACL_PRESENT: u16 = 0x0004;
const SELF_RELATIVE: u16 = 0x8000;
const OPEN_REPARSE: u32 = 0x0020_0000;
const BACKUP_SEMANTICS: u32 = 0x0200_0000;

#[derive(Clone, Copy)]
pub(crate) enum ObjectKind {
    Directory,
    Record,
}

impl ObjectKind {
    fn ace_flags(self) -> u8 {
        match self {
            Self::Directory => 3, // OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE
            Self::Record => 0,
        }
    }
}

fn word(bytes: &[u8], offset: usize) -> Result<u16, WorkspaceError> {
    let value = bytes
        .get(offset..offset + 2)
        .ok_or(WorkspaceError::Denied)?;
    Ok(u16::from_le_bytes([value[0], value[1]]))
}

fn dword(bytes: &[u8], offset: usize) -> Result<u32, WorkspaceError> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or(WorkspaceError::Denied)?;
    Ok(u32::from_le_bytes(
        value.try_into().map_err(|_| WorkspaceError::Denied)?,
    ))
}

fn sid_length(bytes: &[u8]) -> Result<usize, WorkspaceError> {
    let header = bytes.get(..8).ok_or(WorkspaceError::Denied)?;
    if header[0] != 1 || header[1] == 0 || header[1] > 15 {
        return Err(WorkspaceError::Denied);
    }
    let size = 8 + 4 * header[1] as usize;
    bytes.get(..size).ok_or(WorkspaceError::Denied)?;
    Ok(size)
}

fn sid_at(bytes: &[u8], offset: usize) -> Result<&[u8], WorkspaceError> {
    if offset < 20 || !offset.is_multiple_of(4) {
        return Err(WorkspaceError::Denied);
    }
    let suffix = bytes.get(offset..).ok_or(WorkspaceError::Denied)?;
    Ok(&suffix[..sid_length(suffix)?])
}

/// Pure, bounded decoding; never dereference untrusted SID/ACL/ACE pointers.
fn validate_descriptor(
    bytes: &[u8],
    expected_sid: &[u8],
    kind: ObjectKind,
) -> Result<(), WorkspaceError> {
    if bytes.len() < 20
        || bytes.len() > MAX_DESCRIPTOR
        || sid_length(expected_sid)? != expected_sid.len()
        || bytes[0] != 1
        || bytes[1] != 0
    {
        return Err(WorkspaceError::Denied);
    }
    let control = word(bytes, 2)?;
    if control & (PROTECTED_DACL | DACL_PRESENT | SELF_RELATIVE)
        != PROTECTED_DACL | DACL_PRESENT | SELF_RELATIVE
    {
        return Err(WorkspaceError::Denied);
    }
    let owner_offset = dword(bytes, 4)? as usize;
    let owner = sid_at(bytes, owner_offset)?;
    if owner != expected_sid {
        return Err(WorkspaceError::Denied);
    }
    // Group has no authority here, but its layout must still be valid.
    let group_offset = dword(bytes, 8)? as usize;
    if group_offset != 0 {
        // Only OWNER|DACL information is used; an unexpected group layout
        // is not accepted as a permissive alternate descriptor representation.
        return Err(WorkspaceError::Denied);
    }
    // OWNER|DACL is the only requested information; no SACL is admitted.
    if dword(bytes, 12)? != 0 {
        return Err(WorkspaceError::Denied);
    }
    let acl_offset = dword(bytes, 16)? as usize;
    if acl_offset < 20 || !acl_offset.is_multiple_of(4) {
        return Err(WorkspaceError::Denied);
    }
    let suffix = bytes.get(acl_offset..).ok_or(WorkspaceError::Denied)?;
    let acl_size = word(suffix, 2)? as usize;
    let acl = suffix.get(..acl_size).ok_or(WorkspaceError::Denied)?;
    if acl.len() < 8 || acl[0] != 2 || acl[1] != 0 || word(acl, 4)? != 1 || word(acl, 6)? != 0 {
        return Err(WorkspaceError::Denied);
    }
    if owner_offset < acl_offset + acl_size && acl_offset < owner_offset + owner.len() {
        return Err(WorkspaceError::Denied);
    }
    let ace = &acl[8..];
    if ace.len() < 16
        || ace[0] != 0
        || ace[1] != kind.ace_flags()
        || word(ace, 2)? as usize != ace.len()
        || dword(ace, 4)? != FILE_ALL_ACCESS
        || sid_length(&ace[8..])? != ace.len() - 8
        || &ace[8..] != expected_sid
    {
        return Err(WorkspaceError::Denied);
    }
    Ok(())
}

#[repr(C)]
struct SecurityAttributes {
    length: u32,
    descriptor: *mut c_void,
    inherit_handle: i32,
}

#[repr(C)]
struct TokenUser {
    sid: *mut c_void,
    attributes: u32,
}

#[repr(C)]
struct FileTime {
    low: u32,
    high: u32,
}

#[repr(C)]
struct FileInfo {
    attributes: u32,
    created: FileTime,
    accessed: FileTime,
    modified: FileTime,
    volume: u32,
    size_high: u32,
    size_low: u32,
    links: u32,
    index_high: u32,
    index_low: u32,
}

#[allow(non_snake_case, clippy::too_many_arguments)]
#[link(name = "advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(process: *mut c_void, access: u32, token: *mut *mut c_void) -> i32;
    fn GetTokenInformation(
        token: *mut c_void,
        class: u32,
        buffer: *mut c_void,
        size: u32,
        returned: *mut u32,
    ) -> i32;
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        text: *const u16,
        revision: u32,
        descriptor: *mut *mut c_void,
        size: *mut u32,
    ) -> i32;
    fn GetSecurityInfo(
        handle: *mut c_void,
        object_type: u32,
        information: u32,
        owner: *mut *mut c_void,
        group: *mut *mut c_void,
        dacl: *mut *mut c_void,
        sacl: *mut *mut c_void,
        descriptor: *mut *mut c_void,
    ) -> u32;
    fn GetSecurityDescriptorLength(descriptor: *const c_void) -> u32;
    fn GetSecurityDescriptorControl(
        descriptor: *const c_void,
        control: *mut u16,
        revision: *mut u32,
    ) -> i32;
    #[cfg(test)]
    fn GetSecurityDescriptorDacl(
        descriptor: *const c_void,
        present: *mut i32,
        acl: *mut *mut c_void,
        defaulted: *mut i32,
    ) -> i32;
    #[cfg(test)]
    fn SetSecurityInfo(
        handle: *mut c_void,
        object_type: u32,
        information: u32,
        owner: *mut c_void,
        group: *mut c_void,
        dacl: *mut c_void,
        sacl: *mut c_void,
    ) -> u32;
}

#[allow(non_snake_case, clippy::too_many_arguments)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
    fn CloseHandle(handle: *mut c_void) -> i32;
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
    fn CreateDirectoryW(path: *const u16, attributes: *const SecurityAttributes) -> i32;
    fn CreateFileW(
        path: *const u16,
        access: u32,
        share: u32,
        attributes: *const SecurityAttributes,
        disposition: u32,
        flags: u32,
        template: *mut c_void,
    ) -> *mut c_void;
    fn GetFileInformationByHandle(handle: *mut c_void, information: *mut FileInfo) -> i32;
    fn GetDriveTypeW(root: *const u16) -> u32;
    fn GetVolumeInformationW(
        root: *const u16,
        name: *mut u16,
        name_len: u32,
        serial: *mut u32,
        component_len: *mut u32,
        flags: *mut u32,
        filesystem: *mut u16,
        filesystem_len: u32,
    ) -> i32;
}

struct LocalDescriptor(*mut c_void);
impl Drop for LocalDescriptor {
    fn drop(&mut self) {
        // SAFETY: only successful OS LocalAlloc-family outputs enter this RAII owner.
        unsafe {
            LocalFree(self.0);
        }
    }
}

pub(crate) struct OwnerSid {
    words: Vec<u32>,
}

impl OwnerSid {
    pub(crate) fn current_process() -> Result<Self, WorkspaceError> {
        let mut token = ptr::null_mut();
        // SAFETY: pseudo process handle, TOKEN_QUERY only, valid output pointer.
        if unsafe { OpenProcessToken(GetCurrentProcess(), 8, &mut token) } == 0 {
            return Err(WorkspaceError::Unavailable);
        }
        struct Token(*mut c_void);
        impl Drop for Token {
            fn drop(&mut self) {
                unsafe {
                    CloseHandle(self.0);
                }
            }
        }
        let token = Token(token);
        let mut needed = 0;
        unsafe {
            GetTokenInformation(token.0, 1, ptr::null_mut(), 0, &mut needed);
        }
        if needed as usize > MAX_TOKEN || (needed as usize) < std::mem::size_of::<TokenUser>() {
            return Err(WorkspaceError::Denied);
        }
        // usize storage aligns TOKEN_USER; bound the OS size before allocation.
        let mut buffer = vec![0usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
        let capacity = buffer.len() * std::mem::size_of::<usize>();
        if unsafe {
            GetTokenInformation(
                token.0,
                1,
                buffer.as_mut_ptr().cast(),
                capacity as u32,
                &mut needed,
            )
        } == 0
            || needed as usize > capacity
            || (needed as usize) < std::mem::size_of::<TokenUser>()
        {
            return Err(WorkspaceError::Denied);
        }
        // SAFETY: OS success and checked buffer/header size and alignment.
        let user = unsafe { &*buffer.as_ptr().cast::<TokenUser>() };
        let offset = (user.sid as usize)
            .checked_sub(buffer.as_ptr() as usize)
            .ok_or(WorkspaceError::Denied)?;
        if offset < std::mem::size_of::<TokenUser>() || !offset.is_multiple_of(4) {
            return Err(WorkspaceError::Denied);
        }
        // SAFETY: initialized aligned owned buffer, limited to returned bytes.
        let bytes =
            unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), needed as usize) };
        let sid = bytes.get(offset..).ok_or(WorkspaceError::Denied)?;
        let sid = &sid[..sid_length(sid)?];
        Ok(Self {
            words: sid
                .chunks_exact(4)
                .map(|part| u32::from_le_bytes(part.try_into().unwrap()))
                .collect(),
        })
    }

    fn bytes(&self) -> &[u8] {
        // SAFETY: initialized u32 storage, exact owned byte extent.
        unsafe { std::slice::from_raw_parts(self.words.as_ptr().cast(), self.words.len() * 4) }
    }

    fn descriptor(&self, kind: ObjectKind) -> Result<LocalDescriptor, WorkspaceError> {
        let bytes = self.bytes();
        if sid_length(bytes)? != bytes.len() {
            return Err(WorkspaceError::Denied);
        }
        let authority = bytes[2..8]
            .iter()
            .fold(0u64, |value, byte| (value << 8) | *byte as u64);
        let mut sid = format!("S-1-{authority}");
        for part in bytes[8..].chunks_exact(4) {
            use std::fmt::Write;
            write!(
                &mut sid,
                "-{}",
                u32::from_le_bytes(part.try_into().unwrap())
            )
            .map_err(|_| WorkspaceError::Unavailable)?;
        }
        let flags = if matches!(kind, ObjectKind::Directory) {
            "OICI"
        } else {
            ""
        };
        let text: Vec<u16> = format!("O:{sid}D:P(A;{flags};FA;;;{sid})")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mut raw = ptr::null_mut();
        let mut size = 0;
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut raw,
                &mut size,
            )
        } == 0
            || raw.is_null()
        {
            return Err(WorkspaceError::Unavailable);
        }
        let owned = LocalDescriptor(raw);
        if size as usize > MAX_DESCRIPTOR || size < 20 {
            return Err(WorkspaceError::Denied);
        }
        let data = unsafe { std::slice::from_raw_parts(raw.cast::<u8>(), size as usize) };
        validate_descriptor(data, bytes, kind)?;
        Ok(owned)
    }

    pub(crate) fn validate_handle(
        &self,
        file: &File,
        kind: ObjectKind,
    ) -> Result<(), WorkspaceError> {
        validate_descriptor(&descriptor_bytes(file)?, self.bytes(), kind)
    }
}

fn descriptor_bytes(file: &File) -> Result<Vec<u8>, WorkspaceError> {
    let mut raw = ptr::null_mut();
    // OS owns parsing of the original security object; never follow its SID pointers.
    let status = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            1,
            5,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut raw,
        )
    };
    if status != 0 || raw.is_null() {
        return Err(WorkspaceError::Denied);
    }
    let _owned = LocalDescriptor(raw);
    let mut control = 0;
    let mut revision = 0;
    if unsafe { GetSecurityDescriptorControl(raw, &mut control, &mut revision) } == 0
        || revision != 1
        || control & SELF_RELATIVE == 0
    {
        return Err(WorkspaceError::Denied);
    }
    // A successful OS descriptor is an OS-trusted allocation. Obtain its
    // extent through the API, cap it, then parse copied bytes only.
    let size = unsafe { GetSecurityDescriptorLength(raw) } as usize;
    if !(20..=MAX_DESCRIPTOR).contains(&size) {
        return Err(WorkspaceError::Denied);
    }
    let data = unsafe { std::slice::from_raw_parts(raw.cast::<u8>(), size) };
    Ok(data.to_vec())
}

fn path_wide(path: &Path) -> Result<Vec<u16>, WorkspaceError> {
    let text = path.to_str().ok_or(WorkspaceError::Denied)?;
    if text.len() < 4
        || text.len() > 260
        || !text.as_bytes()[0].is_ascii_alphabetic()
        || &text.as_bytes()[1..3] != b":\\"
        || text.contains('\0')
        || text[3..].split('\\').any(|part| {
            part.is_empty()
                || part == "."
                || part == ".."
                || part.ends_with(['.', ' '])
                || part.contains([':', '/', '*', '?', '<', '>', '|'])
        })
    {
        return Err(WorkspaceError::Denied);
    }
    Ok(path.as_os_str().encode_wide().chain(Some(0)).collect())
}

pub(crate) fn eligible_ntfs(path: &Path) -> Result<(), WorkspaceError> {
    path_wide(path)?;
    let prefix = &path.to_str().ok_or(WorkspaceError::Denied)?[..3];
    let root: Vec<u16> = prefix.encode_utf16().chain(Some(0)).collect();
    if unsafe { GetDriveTypeW(root.as_ptr()) } != 3 {
        return Err(WorkspaceError::Denied);
    }
    let mut flags = 0;
    let mut fs = [0u16; 16];
    if unsafe {
        GetVolumeInformationW(
            root.as_ptr(),
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut flags,
            fs.as_mut_ptr(),
            fs.len() as u32,
        )
    } == 0
        || flags & 8 == 0
        || fs[..5] != [b'N' as u16, b'T' as u16, b'F' as u16, b'S' as u16, 0]
    {
        return Err(WorkspaceError::Denied);
    }
    Ok(())
}

pub(crate) fn object_identity(file: &File, kind: ObjectKind) -> Result<Identity, WorkspaceError> {
    let mut info = MaybeUninit::<FileInfo>::uninit();
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
        return Err(WorkspaceError::Io);
    }
    let info = unsafe { info.assume_init() };
    if info.attributes & 0x400 != 0
        || (info.attributes & 0x10 != 0) != matches!(kind, ObjectKind::Directory)
        || matches!(kind, ObjectKind::Record) && info.links != 1
    {
        return Err(WorkspaceError::Denied);
    }
    Ok(Identity {
        volume: info.volume,
        index: ((info.index_high as u64) << 32) | info.index_low as u64,
    })
}

/// Caller retains the already-admitted local NTFS parent and ancestor pins.
/// Collision/failure never changes, repairs or deletes an existing object.
pub(crate) fn create_directory_exact(path: &Path) -> Result<(), WorkspaceError> {
    eligible_ntfs(path)?;
    let owner = OwnerSid::current_process()?;
    let descriptor = owner.descriptor(ObjectKind::Directory)?;
    let attributes = SecurityAttributes {
        length: std::mem::size_of::<SecurityAttributes>() as u32,
        descriptor: descriptor.0,
        inherit_handle: 0,
    };
    let path = path_wide(path)?;
    if unsafe { CreateDirectoryW(path.as_ptr(), &attributes) } == 0 {
        return Err(WorkspaceError::Conflict);
    }
    Ok(())
}

pub(crate) fn create_record_exact(path: &Path, owner: &OwnerSid) -> Result<File, WorkspaceError> {
    let descriptor = owner.descriptor(ObjectKind::Record)?;
    let attributes = SecurityAttributes {
        length: std::mem::size_of::<SecurityAttributes>() as u32,
        descriptor: descriptor.0,
        inherit_handle: 0,
    };
    let path = path_wide(path)?;
    let raw = unsafe {
        CreateFileW(
            path.as_ptr(),
            0xc000_0000,
            0,
            &attributes,
            1,
            0x80 | OPEN_REPARSE,
            ptr::null_mut(),
        )
    };
    if raw as isize == -1 || raw.is_null() {
        return Err(WorkspaceError::Io);
    }
    // SAFETY: successful unique owned CreateFile handle, transferred once.
    let file = unsafe { File::from_raw_handle(raw) };
    object_identity(&file, ObjectKind::Record)?;
    owner.validate_handle(&file, ObjectKind::Record)?;
    Ok(file)
}

pub(crate) fn open_directory(path: &Path) -> Result<File, WorkspaceError> {
    let path = path_wide(path)?;
    let raw = unsafe {
        CreateFileW(
            path.as_ptr(),
            0x8000_0000,
            1,
            ptr::null(),
            3,
            BACKUP_SEMANTICS | OPEN_REPARSE,
            ptr::null_mut(),
        )
    };
    if raw as isize == -1 || raw.is_null() {
        return Err(WorkspaceError::Io);
    }
    Ok(unsafe { File::from_raw_handle(raw) })
}

/// Test-only ACL mutation; callers pass only objects freshly created by the
/// synthetic fixture. Production has no SetSecurityInfo/repair route.
#[cfg(test)]
pub(crate) fn fixture_acl_handle(path: &Path, kind: ObjectKind) -> Result<File, WorkspaceError> {
    let path = path_wide(path)?;
    let flags = OPEN_REPARSE
        | if matches!(kind, ObjectKind::Directory) {
            BACKUP_SEMANTICS
        } else {
            0
        };
    let raw = unsafe {
        CreateFileW(
            path.as_ptr(),
            0x0006_0080,
            3,
            ptr::null(),
            3,
            flags,
            ptr::null_mut(),
        )
    };
    if raw as isize == -1 || raw.is_null() {
        return Err(WorkspaceError::Io);
    }
    let file = unsafe { File::from_raw_handle(raw) };
    object_identity(&file, kind)?;
    Ok(file)
}

#[cfg(test)]
pub(crate) fn fixture_descriptor(file: &File) -> Result<Vec<u8>, WorkspaceError> {
    descriptor_bytes(file)
}

#[cfg(test)]
pub(crate) fn fixture_set_private(
    file: &File,
    kind: ObjectKind,
    private: bool,
) -> Result<(), WorkspaceError> {
    let owner = OwnerSid::current_process()?;
    let current = descriptor_bytes(file)?;
    if sid_at(&current, dword(&current, 4)? as usize)? != owner.bytes() {
        return Err(WorkspaceError::Denied);
    }
    let descriptor = if private {
        owner.descriptor(kind)?
    } else {
        // A broad full-control ACE only on the fixture's own disposable object.
        let text: Vec<u16> = "D:P(A;;FA;;;WD)".encode_utf16().chain(Some(0)).collect();
        let mut raw = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut raw,
                ptr::null_mut(),
            )
        } == 0
            || raw.is_null()
        {
            return Err(WorkspaceError::Unavailable);
        }
        LocalDescriptor(raw)
    };
    let mut present = 0;
    let mut defaulted = 0;
    let mut acl = ptr::null_mut();
    if unsafe { GetSecurityDescriptorDacl(descriptor.0, &mut present, &mut acl, &mut defaulted) }
        == 0
        || present == 0
        || acl.is_null()
    {
        return Err(WorkspaceError::Denied);
    }
    if unsafe {
        SetSecurityInfo(
            file.as_raw_handle(),
            1,
            0x8000_0004,
            ptr::null_mut(),
            ptr::null_mut(),
            acl,
            ptr::null_mut(),
        )
    } != 0
    {
        return Err(WorkspaceError::Denied);
    }
    Ok(())
}

#[cfg(test)]
mod pure_tests {
    use super::*;

    fn sid() -> Vec<u8> {
        vec![1, 2, 0, 0, 0, 0, 0, 5, 21, 0, 0, 0, 7, 0, 0, 0]
    }

    fn fixture(kind: ObjectKind) -> Vec<u8> {
        let sid = sid();
        let acl_offset = 20 + sid.len();
        let ace_size = 8 + sid.len();
        let mut bytes = vec![0u8; acl_offset + 8 + ace_size];
        bytes[0] = 1;
        bytes[2..4].copy_from_slice(&(SELF_RELATIVE | PROTECTED_DACL | DACL_PRESENT).to_le_bytes());
        bytes[4..8].copy_from_slice(&20u32.to_le_bytes());
        bytes[16..20].copy_from_slice(&(acl_offset as u32).to_le_bytes());
        bytes[20..acl_offset].copy_from_slice(&sid);
        bytes[acl_offset] = 2;
        bytes[acl_offset + 2..acl_offset + 4]
            .copy_from_slice(&((8 + ace_size) as u16).to_le_bytes());
        bytes[acl_offset + 4..acl_offset + 6].copy_from_slice(&1u16.to_le_bytes());
        let ace = &mut bytes[acl_offset + 8..];
        ace[1] = kind.ace_flags();
        ace[2..4].copy_from_slice(&(ace_size as u16).to_le_bytes());
        ace[4..8].copy_from_slice(&FILE_ALL_ACCESS.to_le_bytes());
        ace[8..].copy_from_slice(&sid);
        bytes
    }

    #[test]
    fn pure_descriptor_accepts_only_exact_owner_and_object_kind() {
        for kind in [ObjectKind::Directory, ObjectKind::Record] {
            assert!(validate_descriptor(&fixture(kind), &sid(), kind).is_ok());
        }
        assert!(
            validate_descriptor(&fixture(ObjectKind::Directory), &sid(), ObjectKind::Record)
                .is_err()
        );
    }

    #[test]
    fn pure_descriptor_rejects_missing_null_empty_broad_and_foreign_acl() {
        let valid = fixture(ObjectKind::Directory);
        let acl = dword(&valid, 16).unwrap() as usize;
        for (offset, value) in [
            (2, 0),
            (3, 0x80),
            (16, 0),
            (acl + 4, 0),
            (acl + 4, 2),
            (20, 2),
            (28, 99),
            (acl + 8, 1),
            (acl + 8, 5),
            (acl + 8, 9),
            (acl + 9, 0x13),
            (acl + 12, 0),
            (acl + 24, 99),
        ] {
            let mut bad = valid.clone();
            bad[offset] = value;
            assert!(
                validate_descriptor(&bad, &sid(), ObjectKind::Directory).is_err(),
                "offset {offset}"
            );
        }
    }

    #[test]
    fn pure_descriptor_rejects_truncation_size_offsets_and_sid_layouts() {
        let valid = fixture(ObjectKind::Directory);
        for length in 0..valid.len() {
            assert!(validate_descriptor(&valid[..length], &sid(), ObjectKind::Directory).is_err());
        }
        for field in [4, 8, 12, 16] {
            let mut bad = valid.clone();
            bad[field..field + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(validate_descriptor(&bad, &sid(), ObjectKind::Directory).is_err());
        }
        let mut oversized = valid.clone();
        oversized.resize(MAX_DESCRIPTOR + 1, 0);
        assert!(validate_descriptor(&oversized, &sid(), ObjectKind::Directory).is_err());
        for count in [0, 3, 16, 255] {
            let mut bad = valid.clone();
            bad[21] = count;
            assert!(validate_descriptor(&bad, &sid(), ObjectKind::Directory).is_err());
        }
    }

    #[test]
    fn pure_descriptor_rejects_acl_ace_size_and_closed_flag_variants() {
        let valid = fixture(ObjectKind::Directory);
        let acl = dword(&valid, 16).unwrap() as usize;
        for offset in [acl + 1, acl + 6, acl + 7] {
            let mut bad = valid.clone();
            bad[offset] = 1;
            assert!(validate_descriptor(&bad, &sid(), ObjectKind::Directory).is_err());
        }
        for field in [acl + 2, acl + 10] {
            for size in [0u16, 1, 7, 65535] {
                let mut bad = valid.clone();
                bad[field..field + 2].copy_from_slice(&size.to_le_bytes());
                assert!(validate_descriptor(&bad, &sid(), ObjectKind::Directory).is_err());
            }
        }
        for flags in [0u8, 1, 2, 4, 8, 0x10, 0x13, 255] {
            let mut bad = valid.clone();
            bad[acl + 9] = flags;
            assert!(validate_descriptor(&bad, &sid(), ObjectKind::Directory).is_err());
        }
        let mut overlap = valid.clone();
        overlap[16..20].copy_from_slice(&20u32.to_le_bytes());
        assert!(validate_descriptor(&overlap, &sid(), ObjectKind::Directory).is_err());
        let mut missing_owner = valid.clone();
        missing_owner[4..8].fill(0);
        assert!(validate_descriptor(&missing_owner, &sid(), ObjectKind::Directory).is_err());
    }
}
