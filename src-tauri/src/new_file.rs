//! Windows creation journal used by the native-approved new-file host path.
//!
//! Only metadata is journaled; project contents are never copied here. Each
//! immutable record is encrypted by Windows DPAPI for the current user,
//! created without overwriting and flushed before the next state transition.
//! Fixture evidence is separate from installed native-consent and physical
//! power-loss acceptance; those unexecuted checks remain explicit limitations.

#![allow(dead_code)] // Cross-platform builds and recovery proof helpers share this module.

#[cfg(windows)]
mod windows {
    use crate::journal_privacy::{self, ObjectKind, OwnerSid};
    use crate::workspace::WorkspaceError;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use sha2::{Digest, Sha256};
    use std::{
        ffi::c_void,
        fs::{File, OpenOptions},
        io::{Read, Write},
        mem::MaybeUninit,
        os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
        path::{Path, PathBuf},
    };
    use zeroize::Zeroizing;

    const MAX_METADATA_BYTES: usize = 16 * 1024;
    const MAX_CIPHERTEXT_BYTES: u64 = 64 * 1024;
    const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const CRYPTPROTECT_UI_FORBIDDEN: u32 = 1;
    const ENTROPY: &[u8] = b"comzulca/aster-desktop/creation-journal/v1";

    #[repr(C)]
    struct DataBlob {
        bytes: u32,
        data: *mut u8,
    }

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct FileInformation {
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

    #[allow(non_snake_case)]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LocalFree(memory: *mut c_void) -> *mut c_void;
        fn GetFileInformationByHandle(handle: *mut c_void, info: *mut FileInformation) -> i32;
    }

    #[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub(crate) struct Identity {
        pub volume: u32,
        pub index: u64,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub(crate) struct Intent {
        pub version: u32,
        pub nonce: String,
        pub conversation_id: String,
        pub request_id: String,
        pub root_epoch: u64,
        pub root_identity: Identity,
        pub parent_identity: Identity,
        pub relative_path: String,
        pub content_sha256: String,
        pub content_bytes: usize,
        pub review_binding_sha256: String,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub(crate) struct Created {
        pub version: u32,
        pub nonce: String,
        pub identity: Identity,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub(crate) struct Complete {
        pub version: u32,
        pub nonce: String,
        pub identity: Identity,
        pub content_sha256: String,
        pub content_bytes: usize,
    }

    pub(crate) struct Journal<'a> {
        directory: &'a Path,
        anchor: &'a File,
        ancestors: &'a [File],
        ancestor_identities: Vec<Identity>,
        expected_identity: Identity,
        owner: OwnerSid,
    }

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Envelope {
        version: u32,
        payload: String,
        payload_sha256: String,
    }

    fn digest(bytes: &[u8]) -> String {
        Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    fn decode_envelope(plaintext: &[u8]) -> Result<Zeroizing<String>, WorkspaceError> {
        let envelope: Envelope =
            serde_json::from_slice(plaintext).map_err(|_| WorkspaceError::Denied)?;
        let payload = Zeroizing::new(envelope.payload);
        if envelope.version != 1 {
            return Err(WorkspaceError::Denied);
        }
        if payload.len() > MAX_METADATA_BYTES {
            return Err(WorkspaceError::TooLarge);
        }
        if digest(payload.as_bytes()) != envelope.payload_sha256 {
            return Err(WorkspaceError::Denied);
        }
        Ok(payload)
    }

    impl<'a> Journal<'a> {
        /// The workspace service must pin/check this native host-selected
        /// directory and its ancestors throughout use. This constructor is
        /// never a renderer-facing path admission. Borrowed pins cannot be
        /// dropped before this journal's operations finish.
        pub fn in_pinned_directory(
            directory: &'a Path,
            anchor: &'a File,
            ancestors: &'a [File],
            expected_identity: Identity,
        ) -> Result<Self, WorkspaceError> {
            journal_privacy::eligible_ntfs(directory)?;
            let owner = OwnerSid::current_process()?;
            let ancestor_identities = ancestors
                .iter()
                .map(|file| journal_privacy::object_identity(file, ObjectKind::Directory))
                .collect::<Result<Vec<_>, _>>()?;
            let journal = Self {
                directory,
                anchor,
                ancestors,
                ancestor_identities,
                expected_identity,
                owner,
            };
            journal.revalidate()?;
            Ok(journal)
        }

        pub(crate) fn revalidate(&self) -> Result<(), WorkspaceError> {
            for (file, expected) in self.ancestors.iter().zip(&self.ancestor_identities) {
                if journal_privacy::object_identity(file, ObjectKind::Directory)? != *expected {
                    return Err(WorkspaceError::Conflict);
                }
            }
            if journal_privacy::object_identity(self.anchor, ObjectKind::Directory)?
                != self.expected_identity
            {
                return Err(WorkspaceError::Conflict);
            }
            self.owner
                .validate_handle(self.anchor, ObjectKind::Directory)?;
            let current = journal_privacy::open_directory(self.directory)?;
            if journal_privacy::object_identity(&current, ObjectKind::Directory)?
                != self.expected_identity
            {
                return Err(WorkspaceError::Conflict);
            }
            self.owner.validate_handle(&current, ObjectKind::Directory)
        }

        fn path(&self, nonce: &str, kind: &str) -> Result<PathBuf, WorkspaceError> {
            if nonce.len() != 32
                || !nonce
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                || !matches!(kind, "intent" | "created" | "complete")
            {
                return Err(WorkspaceError::Denied);
            }
            Ok(self.directory.join(format!("{nonce}.{kind}")))
        }

        pub fn persist<T: Serialize>(
            &self,
            nonce: &str,
            kind: &str,
            record: &T,
        ) -> Result<(), WorkspaceError> {
            self.revalidate()?;
            // Admit every already-existing immutable record before creating
            // another one. The new exclusive-share record does not exist yet.
            self.nonces()?;
            let plaintext = Zeroizing::new(
                serde_json::to_vec(record).map_err(|_| WorkspaceError::Unavailable)?,
            );
            if plaintext.len() > MAX_METADATA_BYTES {
                return Err(WorkspaceError::TooLarge);
            }
            // DPAPI authentication is supplemented by exact payload integrity
            // and a closed envelope: CryptUnprotectData can sometimes return
            // corrupted output successfully (Microsoft dpapi.h contract).
            let envelope = Envelope {
                version: 1,
                payload: std::str::from_utf8(&plaintext)
                    .map_err(|_| WorkspaceError::Denied)?
                    .to_owned(),
                payload_sha256: digest(&plaintext),
            };
            let encoded_result = serde_json::to_vec(&envelope);
            let _clear_payload = Zeroizing::new(envelope.payload);
            let encoded = Zeroizing::new(encoded_result.map_err(|_| WorkspaceError::Unavailable)?);
            let encrypted = crypt(&encoded, false)?;
            let mut file =
                journal_privacy::create_record_exact(&self.path(nonce, kind)?, &self.owner)?;
            let id = identity(&file)?;
            self.revalidate()?;
            self.owner.validate_handle(&file, ObjectKind::Record)?;
            file.write_all(&encrypted).map_err(|_| WorkspaceError::Io)?;
            file.sync_all().map_err(|_| WorkspaceError::Io)?;
            if identity(&file)? != id {
                return Err(WorkspaceError::Conflict);
            }
            self.owner.validate_handle(&file, ObjectKind::Record)?;
            self.revalidate()?;
            Ok(())
        }

        pub fn read<T: DeserializeOwned>(
            &self,
            nonce: &str,
            kind: &str,
        ) -> Result<T, WorkspaceError> {
            self.revalidate()?;
            let mut file = OpenOptions::new()
                .read(true)
                .share_mode(1) // Deny writers/deleters while reading metadata.
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(self.path(nonce, kind)?)
                .map_err(|_| WorkspaceError::Io)?;
            let id = identity(&file)?;
            self.owner.validate_handle(&file, ObjectKind::Record)?;
            if file.metadata().map_err(|_| WorkspaceError::Io)?.len() > MAX_CIPHERTEXT_BYTES {
                return Err(WorkspaceError::TooLarge);
            }
            let mut encrypted = Zeroizing::new(Vec::new());
            (&mut file)
                .take(MAX_CIPHERTEXT_BYTES + 1)
                .read_to_end(&mut encrypted)
                .map_err(|_| WorkspaceError::Io)?;
            if encrypted.len() as u64 > MAX_CIPHERTEXT_BYTES || identity(&file)? != id {
                return Err(WorkspaceError::Conflict);
            }
            self.owner.validate_handle(&file, ObjectKind::Record)?;
            self.revalidate()?;
            let plaintext = crypt(&encrypted, true)?;
            let payload = decode_envelope(&plaintext)?;
            let value = serde_json::from_str(&payload).map_err(|_| WorkspaceError::Denied)?;
            self.owner.validate_handle(&file, ObjectKind::Record)?;
            self.revalidate()?;
            if identity(&file)? != id {
                return Err(WorkspaceError::Conflict);
            }
            Ok(value)
        }

        /// Do not ignore sidecars or unfamiliar objects: an interrupted,
        /// malformed or foreign record blocks dependent candidate writes.
        pub fn nonces(&self) -> Result<Vec<String>, WorkspaceError> {
            self.revalidate()?;
            let mut nonces = std::collections::BTreeSet::new();
            let mut retained_records = Vec::new();
            let mut entries = 0;
            for entry in std::fs::read_dir(self.directory).map_err(|_| WorkspaceError::Io)? {
                entries += 1;
                if entries > 192 {
                    return Err(WorkspaceError::TooLarge);
                }
                let entry = entry.map_err(|_| WorkspaceError::Io)?;
                let name = entry.file_name();
                let name = name.to_str().ok_or(WorkspaceError::Denied)?;
                let (nonce, kind) = name.split_once('.').ok_or(WorkspaceError::Denied)?;
                self.path(nonce, kind)?;
                let record = OpenOptions::new()
                    .read(true)
                    .share_mode(1)
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                    .open(entry.path())
                    .map_err(|_| WorkspaceError::Io)?;
                let record_identity = identity(&record)?;
                self.owner.validate_handle(&record, ObjectKind::Record)?;
                retained_records.push((record, record_identity));
                nonces.insert(nonce.to_owned());
            }
            self.revalidate()?;
            for (record, expected) in retained_records {
                self.owner.validate_handle(&record, ObjectKind::Record)?;
                if identity(&record)? != expected {
                    return Err(WorkspaceError::Conflict);
                }
            }
            Ok(nonces.into_iter().collect())
        }
    }

    pub(crate) fn identity(file: &File) -> Result<Identity, WorkspaceError> {
        let mut info = MaybeUninit::<FileInformation>::uninit();
        // SAFETY: the live File handle and writable ABI-compatible buffer
        // remain valid until the synchronous Win32 call returns.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
            return Err(WorkspaceError::Io);
        }
        // SAFETY: successful GetFileInformationByHandle initialized the buffer.
        let info = unsafe { info.assume_init() };
        if info.attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0
            || info.links != 1
        {
            return Err(WorkspaceError::Denied);
        }
        Ok(Identity {
            volume: info.volume,
            index: ((info.index_high as u64) << 32) | info.index_low as u64,
        })
    }

    fn crypt(input: &[u8], decrypt: bool) -> Result<Zeroizing<Vec<u8>>, WorkspaceError> {
        if input.is_empty() || input.len() as u64 > MAX_CIPHERTEXT_BYTES {
            return Err(WorkspaceError::TooLarge);
        }
        let source = DataBlob {
            bytes: input.len() as u32,
            data: input.as_ptr().cast_mut(),
        };
        let entropy = DataBlob {
            bytes: ENTROPY.len() as u32,
            data: ENTROPY.as_ptr().cast_mut(),
        };
        let mut output = DataBlob {
            bytes: 0,
            data: std::ptr::null_mut(),
        };
        // SAFETY: input/entropy point to bounded live byte arrays; output is
        // writable. UI is forbidden, scope is the current user (not machine),
        // and all optional description/prompt pointers are null.
        let ok = unsafe {
            if decrypt {
                CryptUnprotectData(
                    &source,
                    std::ptr::null_mut(),
                    &entropy,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output,
                )
            } else {
                CryptProtectData(
                    &source,
                    std::ptr::null(),
                    &entropy,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output,
                )
            }
        };
        if ok == 0 || output.data.is_null() {
            return Err(WorkspaceError::Denied);
        }
        // DPAPI allocates with LocalAlloc. Bound the successful output before
        // copying, wipe its entire buffer and release with the matching API.
        let result = if output.bytes as u64 > MAX_CIPHERTEXT_BYTES {
            Err(WorkspaceError::TooLarge)
        } else {
            // SAFETY: successful DPAPI returns `bytes` initialized bytes.
            Ok(Zeroizing::new(unsafe {
                std::slice::from_raw_parts(output.data, output.bytes as usize).to_vec()
            }))
        };
        // SAFETY: successful DPAPI returned a writable LocalAlloc buffer. Use
        // volatile stores so the compiler cannot elide clearing plaintext.
        unsafe {
            for offset in 0..output.bytes as usize {
                std::ptr::write_volatile(output.data.add(offset), 0);
            }
            LocalFree(output.data.cast());
        }
        result
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn user_scoped_dpapi_roundtrip_is_not_plaintext_and_tampering_denies() {
            let original = b"synthetic-fixture-metadata-only";
            let encrypted = crypt(original, false).unwrap();
            assert!(!encrypted.windows(original.len()).any(|w| w == original));
            assert_eq!(&*crypt(&encrypted, true).unwrap(), original);
            let mut tampered = encrypted.to_vec();
            let end = tampered.len() - 1;
            tampered[end] ^= 0x80;
            assert!(crypt(&tampered, true).is_err());
            assert!(crypt(b"not-a-dpapi-blob", true).is_err());
        }

        #[test]
        fn decrypted_metadata_envelope_rejects_wrong_version_unknown_fields_and_changed_payload() {
            let payload = "{\"synthetic\":\"metadata\"}";
            let fixture = serde_json::json!({"version":1,"payload":payload,"payloadSha256":digest(payload.as_bytes())});
            assert_eq!(
                &*decode_envelope(fixture.to_string().as_bytes()).unwrap(),
                payload
            );
            for (key, value) in [
                ("version", serde_json::json!(2)),
                ("payload", serde_json::json!("{\"synthetic\":\"changed\"}")),
                ("extra", serde_json::json!(true)),
                ("payloadSha256", serde_json::json!("0".repeat(64))),
            ] {
                let mut changed = fixture.clone();
                changed.as_object_mut().unwrap().insert(key.into(), value);
                assert!(decode_envelope(changed.to_string().as_bytes()).is_err());
            }
            let oversized = "x".repeat(MAX_METADATA_BYTES + 1);
            let encoded = serde_json::json!({"version":1,"payload":oversized,"payloadSha256":digest(oversized.as_bytes())});
            assert_eq!(
                decode_envelope(encoded.to_string().as_bytes()).unwrap_err(),
                WorkspaceError::TooLarge
            );
        }
    }
}

#[cfg(windows)]
pub(crate) use windows::*;
