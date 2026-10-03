//! Host-owned immutable, current-user private records. No renderer paths.
//! DPAPI and a protected owner DACL do not defend against same-user compromise
//! or undetectable rollback. Ambiguous/corrupt state is never reset or repaired.

use std::path::Path;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Domain {
    History,
    Budget,
    Terms,
}

impl Domain {
    fn name(self) -> &'static str {
        match self {
            Self::History => "history-v1",
            Self::Budget => "budget-v1",
            Self::Terms => "terms-v1",
        }
    }

    fn limits(self) -> (usize, usize, usize) {
        match self {
            Self::History => (4 * 1024 * 1024, 64 * 1024 * 1024, 8192),
            Self::Budget => (64 * 1024, 32 * 1024 * 1024, 32768),
            Self::Terms => (16 * 1024, 1024 * 1024, 512),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum StoreError {
    Unavailable,
    Denied,
    Conflict,
    Corrupt,
    Capacity,
    Io,
}

impl StoreError {
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::Unavailable => "private_store_unavailable",
            Self::Denied => "private_store_denied",
            Self::Conflict => "private_store_conflict",
            Self::Corrupt => "private_store_corrupt",
            Self::Capacity => "private_store_capacity",
            Self::Io => "private_store_io",
        }
    }
}

fn valid_name(name: &str) -> bool {
    name.len() == 32
        && name
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_payload(domain: Domain, bytes: &[u8]) -> Result<(), StoreError> {
    if bytes.is_empty() || bytes.len() > domain.limits().0 {
        return Err(StoreError::Capacity);
    }
    // Application domains use bounded JSON UTF-8, never arbitrary binary.
    std::str::from_utf8(bytes).map_err(|_| StoreError::Corrupt)?;
    Ok(())
}

pub(crate) struct PrivateStore {
    #[cfg(windows)]
    native: Option<windows::NativeStore>,
    #[cfg(test)]
    memory: Option<MemoryStore>,
}

impl PrivateStore {
    /// `base` comes only from the host's app-local path service. The child
    /// domain is fixed and no caller may supply a renderer/project path.
    pub(crate) fn open(base: &Path, domain: Domain) -> Result<Self, StoreError> {
        #[cfg(windows)]
        {
            Ok(Self {
                native: Some(windows::NativeStore::open(base, domain)?),
                #[cfg(test)]
                memory: None,
            })
        }
        #[cfg(not(windows))]
        {
            let _ = (base, domain);
            Err(StoreError::Unavailable)
        }
    }

    pub(crate) fn records(&self) -> Result<Vec<(String, Vec<u8>)>, StoreError> {
        #[cfg(test)]
        if let Some(memory) = &self.memory {
            memory.validate()?;
            return Ok(memory.records.clone());
        }
        #[cfg(windows)]
        if let Some(native) = &self.native {
            return native.records();
        }
        Err(StoreError::Unavailable)
    }
    pub(crate) fn has_capacity(
        &self,
        domain: Domain,
        bytes: usize,
        count: usize,
    ) -> Result<bool, StoreError> {
        let records = self.records()?;
        let (_, maximum, maximum_count) = domain.limits();
        let used = records.iter().try_fold(0usize, |sum, (_, body)| {
            sum.checked_add(body.len()).ok_or(StoreError::Capacity)
        })?;
        Ok(used.checked_add(bytes).is_some_and(|n| n <= maximum)
            && records
                .len()
                .checked_add(count)
                .is_some_and(|n| n <= maximum_count))
    }

    #[cfg(test)]
    pub(crate) fn read(&self, name: &str) -> Result<Vec<u8>, StoreError> {
        if !valid_name(name) {
            return Err(StoreError::Denied);
        }
        #[cfg(test)]
        if let Some(memory) = &self.memory {
            memory.validate()?;
            return memory
                .records
                .iter()
                .find(|(candidate, _)| candidate == name)
                .map(|(_, payload)| payload.clone())
                .ok_or(StoreError::Denied);
        }
        #[cfg(windows)]
        if let Some(native) = &self.native {
            return native.read(name);
        }
        Err(StoreError::Unavailable)
    }

    pub(crate) fn append(&mut self, name: &str, payload: &[u8]) -> Result<(), StoreError> {
        if !valid_name(name) {
            return Err(StoreError::Denied);
        }
        #[cfg(test)]
        if let Some(memory) = &mut self.memory {
            memory.validate()?;
            validate_payload(memory.domain, payload)?;
            if memory
                .records
                .iter()
                .any(|(candidate, _)| candidate == name)
            {
                return Err(StoreError::Conflict);
            }
            let (_, max_total, max_count) = memory.domain.limits();
            let used = memory
                .records
                .iter()
                .try_fold(0usize, |total, (_, bytes)| {
                    total.checked_add(bytes.len()).ok_or(StoreError::Capacity)
                })?;
            if memory.records.len() >= max_count
                || used
                    .checked_add(payload.len())
                    .is_none_or(|sum| sum > max_total)
            {
                return Err(StoreError::Capacity);
            }
            let failure = memory.fail_next.take();
            if failure == Some(false) {
                return Err(StoreError::Io);
            }
            memory.records.push((name.to_owned(), payload.to_vec()));
            if failure == Some(true) {
                return Err(StoreError::Io);
            }
            return Ok(());
        }
        #[cfg(windows)]
        if let Some(native) = &mut self.native {
            return native.append(name, payload);
        }
        Err(StoreError::Unavailable)
    }

    #[cfg(test)]
    pub(crate) fn memory(domain: Domain) -> Self {
        Self {
            #[cfg(windows)]
            native: None,
            memory: Some(MemoryStore {
                domain,
                records: Vec::new(),
                fail_next: None,
            }),
        }
    }

    /// Pure memory fixture only: emulate a failed immutable append before or
    /// after durable visibility, without touching an actual user store.
    #[cfg(test)]
    pub(crate) fn fail_next_append_for_test(&mut self, after_write: bool) {
        self.memory
            .as_mut()
            .expect("append failure injection requires an isolated memory fixture")
            .fail_next = Some(after_write);
    }
}

pub(crate) fn new_id() -> Result<String, StoreError> {
    #[cfg(windows)]
    {
        windows::random_id()
    }
    #[cfg(all(not(windows), test))]
    {
        // Pure fixture IDs are not an OS randomness or production proof.
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Ok(format!(
            "{:032x}",
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ))
    }
    #[cfg(all(not(windows), not(test)))]
    {
        Err(StoreError::Unavailable)
    }
}

#[cfg(test)]
struct MemoryStore {
    domain: Domain,
    records: Vec<(String, Vec<u8>)>,
    fail_next: Option<bool>,
}

#[cfg(test)]
impl MemoryStore {
    fn validate(&self) -> Result<(), StoreError> {
        let mut seen = std::collections::HashSet::new();
        let (_, max_total, max_count) = self.domain.limits();
        if self.records.len() > max_count {
            return Err(StoreError::Capacity);
        }
        let mut total = 0usize;
        for (name, bytes) in &self.records {
            if !valid_name(name) || !seen.insert(name) {
                return Err(StoreError::Corrupt);
            }
            validate_payload(self.domain, bytes)?;
            total = total.checked_add(bytes.len()).ok_or(StoreError::Capacity)?;
        }
        if total > max_total {
            return Err(StoreError::Capacity);
        }
        Ok(())
    }
}

#[cfg(windows)]
mod windows {
    use super::{valid_name, validate_payload, Domain, StoreError};
    use crate::journal_privacy::{self, ObjectKind, OwnerSid};
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};
    use std::{
        ffi::c_void,
        fs::{File, OpenOptions},
        io::{Read, Seek, SeekFrom, Write},
        os::windows::fs::OpenOptionsExt,
        path::{Path, PathBuf},
    };
    use zeroize::Zeroizing;

    const CONTAINER: &str = "com.comzulca.aster-desktop-private-v1";
    const LEASE: &str = "writer.lease";
    const OPEN_REPARSE: u32 = 0x0020_0000;
    const BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const MAX_CIPHERTEXT: usize = 32 * 1024 * 1024;

    #[repr(C)]
    struct DataBlob {
        bytes: u32,
        data: *mut u8,
    }

    #[allow(non_snake_case, clippy::too_many_arguments)]
    #[link(name = "crypt32")]
    unsafe extern "system" {
        fn CryptProtectData(
            input: *const DataBlob,
            description: *const u16,
            entropy: *const DataBlob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut DataBlob,
        ) -> i32;
        fn CryptUnprotectData(
            input: *const DataBlob,
            description: *mut *mut u16,
            entropy: *const DataBlob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut DataBlob,
        ) -> i32;
    }

    #[allow(non_snake_case, clippy::too_many_arguments)]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LocalFree(pointer: *mut c_void) -> *mut c_void;
        fn QueryDosDeviceW(name: *const u16, target: *mut u16, capacity: u32) -> u32;
        fn GetVolumeInformationW(
            root: *const u16,
            volume_name: *mut u16,
            volume_name_capacity: u32,
            serial: *mut u32,
            maximum_component_length: *mut u32,
            flags: *mut u32,
            file_system: *mut u16,
            file_system_capacity: u32,
        ) -> i32;
    }

    #[allow(non_snake_case)]
    #[link(name = "bcrypt")]
    unsafe extern "system" {
        fn BCryptGenRandom(algorithm: *mut c_void, buffer: *mut u8, bytes: u32, flags: u32) -> i32;
    }

    fn map_error(error: crate::workspace::WorkspaceError) -> StoreError {
        use crate::workspace::WorkspaceError;
        match error {
            WorkspaceError::Conflict => StoreError::Conflict,
            WorkspaceError::TooLarge => StoreError::Capacity,
            WorkspaceError::Io => StoreError::Io,
            WorkspaceError::Unavailable => StoreError::Unavailable,
            _ => StoreError::Denied,
        }
    }

    fn digest(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn crypt(
        domain: Domain,
        bytes: &[u8],
        decrypt: bool,
    ) -> Result<Zeroizing<Vec<u8>>, StoreError> {
        if bytes.is_empty() || bytes.len() > MAX_CIPHERTEXT {
            return Err(StoreError::Capacity);
        }
        let entropy = format!(
            "comzulca/aster-desktop/private-store/{}/schema-1",
            domain.name()
        );
        let source = DataBlob {
            bytes: bytes.len() as u32,
            data: bytes.as_ptr().cast_mut(),
        };
        let entropy_blob = DataBlob {
            bytes: entropy.len() as u32,
            data: entropy.as_ptr().cast_mut(),
        };
        let mut output = DataBlob {
            bytes: 0,
            data: std::ptr::null_mut(),
        };
        // Current-user protection only, no prompt/UI, no machine scope.
        let result = unsafe {
            if decrypt {
                CryptUnprotectData(
                    &source,
                    std::ptr::null_mut(),
                    &entropy_blob,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    1,
                    &mut output,
                )
            } else {
                CryptProtectData(
                    &source,
                    std::ptr::null(),
                    &entropy_blob,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    1,
                    &mut output,
                )
            }
        };
        if result == 0 || output.data.is_null() {
            return Err(StoreError::Corrupt);
        }
        let decoded = if output.bytes as usize > MAX_CIPHERTEXT {
            Err(StoreError::Capacity)
        } else {
            Ok(Zeroizing::new(unsafe {
                std::slice::from_raw_parts(output.data, output.bytes as usize).to_vec()
            }))
        };
        // Successful DPAPI allocations use LocalAlloc; erase then free.
        unsafe {
            for offset in 0..output.bytes as usize {
                std::ptr::write_volatile(output.data.add(offset), 0);
            }
            LocalFree(output.data.cast());
        }
        decoded
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Envelope {
        version: u32,
        domain: String,
        name: String,
        payload: String,
        payload_sha256: String,
    }

    struct Pin {
        path: PathBuf,
        file: File,
        identity: crate::new_file::Identity,
        private: bool,
    }

    impl Pin {
        fn open(path: PathBuf, private: bool, owner: &OwnerSid) -> Result<Self, StoreError> {
            // Restrictive live ancestor pins exclude write/delete handles,
            // including reparse mutations, for the lifetime of this lease.
            let file = OpenOptions::new()
                .read(true)
                .share_mode(1)
                .custom_flags(OPEN_REPARSE | BACKUP_SEMANTICS)
                .open(&path)
                .map_err(|_| StoreError::Denied)?;
            let identity = journal_privacy::object_identity(&file, ObjectKind::Directory)
                .map_err(map_error)?;
            if private {
                owner
                    .validate_handle(&file, ObjectKind::Directory)
                    .map_err(map_error)?;
            }
            Ok(Self {
                path,
                file,
                identity,
                private,
            })
        }

        fn validate(&self, owner: &OwnerSid) -> Result<(), StoreError> {
            if journal_privacy::object_identity(&self.file, ObjectKind::Directory)
                .map_err(map_error)?
                != self.identity
            {
                return Err(StoreError::Conflict);
            }
            let current = Pin::open(self.path.clone(), self.private, owner)?;
            if current.identity != self.identity {
                return Err(StoreError::Conflict);
            }
            if self.private {
                owner
                    .validate_handle(&self.file, ObjectKind::Directory)
                    .map_err(map_error)?;
            }
            Ok(())
        }
    }

    pub(super) struct NativeStore {
        domain: Domain,
        directory: PathBuf,
        pins: Vec<Pin>,
        // Pins retain ancestor identities, while this restrictive anchor
        // excludes write/delete handles against the admitted domain directory.
        anchor: File,
        anchor_identity: crate::new_file::Identity,
        lease: File,
        lease_identity: crate::new_file::Identity,
        owner: OwnerSid,
    }

    fn components(path: &Path) -> Result<Vec<PathBuf>, StoreError> {
        let text = path.to_str().ok_or(StoreError::Denied)?;
        if text.len() < 4
            || text.len() > 196
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
            return Err(StoreError::Denied);
        }
        let mut paths = vec![PathBuf::from(&text[..3])];
        for component in text[3..].split('\\') {
            paths.push(paths.last().ok_or(StoreError::Denied)?.join(component));
        }
        Ok(paths)
    }

    fn native_volume(base: &Path) -> Result<u32, StoreError> {
        let text = base.to_str().ok_or(StoreError::Denied)?;
        let bytes = text.as_bytes();
        if bytes.len() < 3 || !bytes[0].is_ascii_alphabetic() || &bytes[1..3] != b":\\" {
            return Err(StoreError::Denied);
        }
        let drive = [bytes[0] as u16, b':' as u16, 0];
        let mut target = [0u16; 512];
        let count =
            unsafe { QueryDosDeviceW(drive.as_ptr(), target.as_mut_ptr(), target.len() as u32) }
                as usize;
        if count == 0 || count > target.len() {
            return Err(StoreError::Denied);
        }
        let end = target[..count]
            .iter()
            .position(|unit| *unit == 0)
            .ok_or(StoreError::Denied)?;
        let mapping = String::from_utf16(&target[..end]).map_err(|_| StoreError::Denied)?;
        let suffix = mapping
            .strip_prefix(r"\Device\HarddiskVolume")
            .ok_or(StoreError::Denied)?;
        if suffix.is_empty() || !suffix.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(StoreError::Denied);
        }
        let root = [bytes[0] as u16, b':' as u16, b'\\' as u16, 0];
        let mut serial = 0u32;
        if unsafe {
            GetVolumeInformationW(
                root.as_ptr(),
                std::ptr::null_mut(),
                0,
                &mut serial,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
            )
        } == 0
        {
            return Err(StoreError::Unavailable);
        }
        Ok(serial)
    }

    fn private_child(parent: &Path, name: &str, owner: &OwnerSid) -> Result<Pin, StoreError> {
        let child = parent.join(name);
        match std::fs::symlink_metadata(&child) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                journal_privacy::create_directory_exact(&child).map_err(map_error)?;
            }
            Ok(_) => (),
            Err(_) => return Err(StoreError::Io),
        }
        // Never repair/adopt weak ACLs or foreign/reparse objects.
        Pin::open(child, true, owner)
    }

    impl NativeStore {
        pub(super) fn open(base: &Path, domain: Domain) -> Result<Self, StoreError> {
            journal_privacy::eligible_ntfs(base).map_err(map_error)?;
            let volume = native_volume(base)?;
            let owner = OwnerSid::current_process().map_err(map_error)?;
            let mut pins = components(base)?
                .into_iter()
                .map(|path| Pin::open(path, false, &owner))
                .collect::<Result<Vec<_>, _>>()?;
            if pins.last().ok_or(StoreError::Denied)?.identity.volume != volume {
                return Err(StoreError::Denied);
            }
            let container = private_child(base, CONTAINER, &owner)?;
            let child = private_child(&container.path, domain.name(), &owner)?;
            let directory = child.path.clone();
            let anchor_identity = child.identity;
            pins.push(container);
            pins.push(child);
            for pin in &pins {
                pin.validate(&owner)?;
            }
            let anchor = journal_privacy::open_directory(&directory).map_err(map_error)?;
            owner
                .validate_handle(&anchor, ObjectKind::Directory)
                .map_err(map_error)?;
            if journal_privacy::object_identity(&anchor, ObjectKind::Directory)
                .map_err(map_error)?
                != anchor_identity
            {
                return Err(StoreError::Conflict);
            }
            let lease_path = directory.join(LEASE);
            let lease = match std::fs::symlink_metadata(&lease_path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    let file = journal_privacy::create_record_exact(&lease_path, &owner)
                        .map_err(map_error)?;
                    file.sync_all().map_err(|_| StoreError::Io)?;
                    file
                }
                Ok(_) => OpenOptions::new()
                    .read(true)
                    .write(true)
                    .share_mode(0)
                    .custom_flags(OPEN_REPARSE)
                    .open(&lease_path)
                    .map_err(|_| StoreError::Conflict)?,
                Err(_) => return Err(StoreError::Io),
            };
            let lease_identity =
                journal_privacy::object_identity(&lease, ObjectKind::Record).map_err(map_error)?;
            owner
                .validate_handle(&lease, ObjectKind::Record)
                .map_err(map_error)?;
            if lease.metadata().map_err(|_| StoreError::Io)?.len() != 0 {
                return Err(StoreError::Corrupt);
            }
            let store = Self {
                domain,
                directory,
                pins,
                anchor,
                anchor_identity,
                lease,
                lease_identity,
                owner,
            };
            store.records()?;
            Ok(store)
        }

        fn validate(&self) -> Result<(), StoreError> {
            for pin in &self.pins {
                pin.validate(&self.owner)?;
            }
            if journal_privacy::object_identity(&self.anchor, ObjectKind::Directory)
                .map_err(map_error)?
                != self.anchor_identity
                || journal_privacy::object_identity(&self.lease, ObjectKind::Record)
                    .map_err(map_error)?
                    != self.lease_identity
            {
                return Err(StoreError::Conflict);
            }
            self.owner
                .validate_handle(&self.anchor, ObjectKind::Directory)
                .map_err(map_error)?;
            self.owner
                .validate_handle(&self.lease, ObjectKind::Record)
                .map_err(map_error)?;
            if self.lease.metadata().map_err(|_| StoreError::Io)?.len() != 0 {
                return Err(StoreError::Corrupt);
            }
            Ok(())
        }

        fn open_record(&self, name: &str) -> Result<File, StoreError> {
            if !valid_name(name) {
                return Err(StoreError::Denied);
            }
            let file = OpenOptions::new()
                .read(true)
                .share_mode(1)
                .custom_flags(OPEN_REPARSE)
                .open(self.directory.join(format!("{name}.dpapi")))
                .map_err(|_| StoreError::Denied)?;
            journal_privacy::object_identity(&file, ObjectKind::Record).map_err(map_error)?;
            self.owner
                .validate_handle(&file, ObjectKind::Record)
                .map_err(map_error)?;
            Ok(file)
        }

        fn read_file(&self, name: &str, file: &mut File) -> Result<Vec<u8>, StoreError> {
            let identity =
                journal_privacy::object_identity(file, ObjectKind::Record).map_err(map_error)?;
            self.owner
                .validate_handle(file, ObjectKind::Record)
                .map_err(map_error)?;
            if file.metadata().map_err(|_| StoreError::Io)?.len() > MAX_CIPHERTEXT as u64 {
                return Err(StoreError::Capacity);
            }
            let mut encrypted = Zeroizing::new(Vec::new());
            file.seek(SeekFrom::Start(0)).map_err(|_| StoreError::Io)?;
            (&mut *file)
                .take(MAX_CIPHERTEXT as u64 + 1)
                .read_to_end(&mut encrypted)
                .map_err(|_| StoreError::Io)?;
            if encrypted.len() > MAX_CIPHERTEXT {
                return Err(StoreError::Capacity);
            }
            let plaintext = crypt(self.domain, &encrypted, true)?;
            let envelope: Envelope =
                serde_json::from_slice(&plaintext).map_err(|_| StoreError::Corrupt)?;
            let payload = Zeroizing::new(envelope.payload);
            if envelope.version != 1
                || envelope.domain != self.domain.name()
                || envelope.name != name
                || digest(payload.as_bytes()) != envelope.payload_sha256
            {
                return Err(StoreError::Corrupt);
            }
            validate_payload(self.domain, payload.as_bytes())?;
            self.owner
                .validate_handle(file, ObjectKind::Record)
                .map_err(map_error)?;
            if journal_privacy::object_identity(file, ObjectKind::Record).map_err(map_error)?
                != identity
            {
                return Err(StoreError::Conflict);
            }
            Ok(payload.as_bytes().to_vec())
        }

        #[cfg(test)]
        pub(super) fn read(&self, name: &str) -> Result<Vec<u8>, StoreError> {
            self.validate()?;
            let mut file = self.open_record(name)?;
            let payload = self.read_file(name, &mut file)?;
            self.validate()?;
            Ok(payload)
        }

        pub(super) fn records(&self) -> Result<Vec<(String, Vec<u8>)>, StoreError> {
            self.validate()?;
            let (_, max_total, max_count) = self.domain.limits();
            let mut total = 0usize;
            let mut names = std::collections::BTreeSet::new();
            let mut records = Vec::new();
            // Retain admitted handles until all records and ancestry pass.
            let mut handles = Vec::new();
            let mut lease_seen = false;
            for entry in std::fs::read_dir(&self.directory).map_err(|_| StoreError::Io)? {
                let entry = entry.map_err(|_| StoreError::Io)?;
                let name = entry.file_name();
                let name = name.to_str().ok_or(StoreError::Corrupt)?;
                if name == LEASE {
                    if lease_seen {
                        return Err(StoreError::Corrupt);
                    }
                    lease_seen = true;
                    continue;
                }
                let id = name.strip_suffix(".dpapi").ok_or(StoreError::Corrupt)?;
                if !valid_name(id) || !names.insert(id.to_owned()) || names.len() > max_count {
                    return Err(StoreError::Corrupt);
                }
                let mut file = self.open_record(id)?;
                let payload = self.read_file(id, &mut file)?;
                total = total
                    .checked_add(payload.len())
                    .ok_or(StoreError::Capacity)?;
                if total > max_total {
                    return Err(StoreError::Capacity);
                }
                records.push((id.to_owned(), payload));
                handles.push(file);
            }
            if !lease_seen {
                return Err(StoreError::Corrupt);
            }
            self.validate()?;
            for file in handles {
                self.owner
                    .validate_handle(&file, ObjectKind::Record)
                    .map_err(map_error)?;
                journal_privacy::object_identity(&file, ObjectKind::Record).map_err(map_error)?;
            }
            records.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(records)
        }

        pub(super) fn append(&mut self, name: &str, payload: &[u8]) -> Result<(), StoreError> {
            validate_payload(self.domain, payload)?;
            let prior = self.records()?;
            if prior.iter().any(|(candidate, _)| candidate == name) {
                return Err(StoreError::Conflict);
            }
            let (_, max_total, max_count) = self.domain.limits();
            let used = prior.iter().try_fold(0usize, |sum, (_, bytes)| {
                sum.checked_add(bytes.len()).ok_or(StoreError::Capacity)
            })?;
            if prior.len() >= max_count
                || used
                    .checked_add(payload.len())
                    .is_none_or(|sum| sum > max_total)
            {
                return Err(StoreError::Capacity);
            }
            let envelope = Envelope {
                version: 1,
                domain: self.domain.name().to_owned(),
                name: name.to_owned(),
                payload: std::str::from_utf8(payload)
                    .map_err(|_| StoreError::Corrupt)?
                    .to_owned(),
                payload_sha256: digest(payload),
            };
            let encoded_result = serde_json::to_vec(&envelope);
            let _clear = Zeroizing::new(envelope.payload);
            let encoded = Zeroizing::new(encoded_result.map_err(|_| StoreError::Corrupt)?);
            let encrypted = crypt(self.domain, &encoded, false)?;
            self.validate()?;
            let path = self.directory.join(format!("{name}.dpapi"));
            let mut file =
                journal_privacy::create_record_exact(&path, &self.owner).map_err(map_error)?;
            let identity =
                journal_privacy::object_identity(&file, ObjectKind::Record).map_err(map_error)?;
            self.validate()?;
            file.write_all(&encrypted).map_err(|_| StoreError::Io)?;
            file.sync_all().map_err(|_| StoreError::Io)?;
            if journal_privacy::object_identity(&file, ObjectKind::Record).map_err(map_error)?
                != identity
            {
                return Err(StoreError::Conflict);
            }
            if self.read_file(name, &mut file)? != payload {
                return Err(StoreError::Corrupt);
            }
            self.validate()?;
            // Any failed partial immutable object remains for attention.
            Ok(())
        }
    }

    pub(super) fn random_id() -> Result<String, StoreError> {
        let mut bytes = [0u8; 16];
        let status = unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                bytes.as_mut_ptr(),
                bytes.len() as u32,
                2,
            )
        };
        if status != 0 {
            return Err(StoreError::Unavailable);
        }
        Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_names_and_payloads_are_closed_and_collisions_never_overwrite() {
        let mut store = PrivateStore::memory(Domain::Terms);
        let name = "a".repeat(32);
        store.append(&name, b"{\"synthetic\":true}").unwrap();
        assert_eq!(store.append(&name, b"{}"), Err(StoreError::Conflict));
        for hostile in ["../x", "", "A", "budget-v1", "a:stream"] {
            assert_eq!(store.append(hostile, b"{}"), Err(StoreError::Denied));
        }
        assert!(store.append(&"b".repeat(32), &[0xff]).is_err());
        assert_eq!(store.read(&name).unwrap(), b"{\"synthetic\":true}");
    }

    #[test]
    fn pure_fixture_capacity_is_checked_before_append() {
        let mut store = PrivateStore::memory(Domain::Terms);
        let bytes = vec![b'x'; 16 * 1024];
        for id in 0..64 {
            store.append(&format!("{id:032x}"), &bytes).unwrap();
        }
        assert_eq!(
            store.append(&format!("{:032x}", 64), b"x"),
            Err(StoreError::Capacity)
        );
        assert_eq!(store.records().unwrap().len(), 64);
    }

    #[test]
    fn pure_append_failures_distinguish_no_write_from_committed_but_unacknowledged() {
        let mut store = PrivateStore::memory(Domain::Budget);
        let before = new_id().unwrap();
        store.fail_next_append_for_test(false);
        assert_eq!(store.append(&before, b"{}"), Err(StoreError::Io));
        assert!(store.records().unwrap().is_empty());
        let after = new_id().unwrap();
        store.fail_next_append_for_test(true);
        assert_eq!(store.append(&after, b"{}"), Err(StoreError::Io));
        assert_eq!(store.read(&after).unwrap(), b"{}");
        assert_eq!(store.records().unwrap().len(), 1);
    }

    #[cfg(windows)]
    #[test]
    fn isolated_native_store_roundtrip_lock_and_ciphertext_corruption_deny() {
        let base =
            std::env::temp_dir().join(format!("aster-private-fixture-{}", new_id().unwrap()));
        std::fs::create_dir(&base).unwrap();
        let name = new_id().unwrap();
        let record_path = base
            .join("com.comzulca.aster-desktop-private-v1")
            .join("history-v1")
            .join(format!("{name}.dpapi"));
        {
            let mut store = PrivateStore::open(&base, Domain::History).unwrap();
            store
                .append(&name, b"{\"synthetic\":\"fixture only\"}")
                .unwrap();
            assert!(PrivateStore::open(&base, Domain::History).is_err());
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "private_store::tests::isolated_native_child_lease_is_denied",
                    "--nocapture",
                ])
                .env("ASTER_SYNTHETIC_PRIVATE_STORE_CHILD", &base)
                .status()
                .unwrap();
            assert!(child.success());
            assert_eq!(
                store.read(&name).unwrap(),
                b"{\"synthetic\":\"fixture only\"}"
            );
            let disk = std::fs::read(&record_path).unwrap();
            assert!(!disk.windows(12).any(|part| part == b"fixture only"));
        }
        {
            let store = PrivateStore::open(&base, Domain::History).unwrap();
            assert_eq!(store.records().unwrap().len(), 1);
        }
        std::fs::write(&record_path, b"not encrypted").unwrap();
        assert!(PrivateStore::open(&base, Domain::History).is_err());
        // Cleanup targets only the freshly generated isolated fixture root.
        std::fs::remove_dir_all(&base).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn isolated_native_child_lease_is_denied() {
        let Some(base) = std::env::var_os("ASTER_SYNTHETIC_PRIVATE_STORE_CHILD") else {
            return;
        };
        let base = std::path::PathBuf::from(base);
        assert_eq!(base.parent(), Some(std::env::temp_dir().as_path()));
        let name = base.file_name().unwrap().to_str().unwrap();
        let opaque = name.strip_prefix("aster-private-fixture-").unwrap();
        assert!(valid_name(opaque));
        assert!(PrivateStore::open(&base, Domain::History).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn isolated_native_foreign_acl_and_cross_domain_records_are_preserved_and_denied() {
        use crate::journal_privacy::{self, ObjectKind};
        let base =
            std::env::temp_dir().join(format!("aster-private-fixture-{}", new_id().unwrap()));
        std::fs::create_dir(&base).unwrap();
        let name = new_id().unwrap();
        let directory = base
            .join("com.comzulca.aster-desktop-private-v1")
            .join("history-v1");
        let path = directory.join(format!("{name}.dpapi"));
        {
            let mut store = PrivateStore::open(&base, Domain::History).unwrap();
            store.append(&name, b"{\"synthetic\":true}").unwrap();
            let acl = journal_privacy::fixture_acl_handle(&path, ObjectKind::Record).unwrap();
            journal_privacy::fixture_set_private(&acl, ObjectKind::Record, false).unwrap();
            assert!(store.records().is_err());
            assert!(store.append(&new_id().unwrap(), b"{}").is_err());
            // The fixture restores its own disposable object only. Runtime
            // code has no repair route and never makes this mutation.
            journal_privacy::fixture_set_private(&acl, ObjectKind::Record, true).unwrap();
        }
        let original = std::fs::read(&path).unwrap();
        {
            let _terms = PrivateStore::open(&base, Domain::Terms).unwrap();
        }
        let foreign = base
            .join("com.comzulca.aster-desktop-private-v1")
            .join("terms-v1")
            .join(format!("{name}.dpapi"));
        let owner = journal_privacy::OwnerSid::current_process().unwrap();
        {
            use std::io::Write;
            let mut record = journal_privacy::create_record_exact(&foreign, &owner).unwrap();
            record.write_all(&original).unwrap();
            record.sync_all().unwrap();
        }
        assert!(PrivateStore::open(&base, Domain::Terms).is_err());
        assert_eq!(std::fs::read(&foreign).unwrap(), original);
        {
            let store = PrivateStore::open(&base, Domain::History).unwrap();
            let unexpected = directory.join("foreign-sidecar");
            std::fs::write(&unexpected, b"fixture only").unwrap();
            assert!(store.records().is_err());
            assert_eq!(std::fs::read(&unexpected).unwrap(), b"fixture only");
        }
        std::fs::remove_dir_all(&base).unwrap();
    }
}
