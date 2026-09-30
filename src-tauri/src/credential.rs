//! The NVIDIA credential exists only in the trusted native host and the
//! current user's Windows Credential Manager. Never put it in a command DTO.

use zeroize::Zeroizing;

const TARGET: &str = "comzulca/aster-desktop/nvidia/integrate.api.nvidia.com";

#[cfg(any(windows, test))]
fn valid_key(key: &str) -> bool {
    (8..=256).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_graphic())
}

/// Generic Credential blobs are application-defined. Accept our legacy ASCII
/// format and native UTF-16LE ASCII passwords, never trim or repair a secret.
#[cfg(any(windows, test))]
fn decode_blob(blob: &[u8]) -> Result<Zeroizing<String>, CredentialError> {
    if (8..=256).contains(&blob.len()) && blob.iter().all(u8::is_ascii_graphic) {
        let key = std::str::from_utf8(blob).map_err(|_| CredentialError::Rejected)?;
        return Ok(Zeroizing::new(key.to_owned()));
    }
    if !(16..=512).contains(&blob.len()) || !blob.len().is_multiple_of(2) {
        return Err(CredentialError::Rejected);
    }
    let mut key = Zeroizing::new(String::with_capacity(blob.len() / 2));
    for pair in blob.chunks_exact(2) {
        if pair[1] != 0 || !pair[0].is_ascii_graphic() {
            return Err(CredentialError::Rejected);
        }
        key.push(pair[0] as char);
    }
    if !valid_key(&key) {
        return Err(CredentialError::Rejected);
    }
    Ok(key)
}

#[cfg(any(windows, test))]
fn decode_native_password(password: &[u16]) -> Result<Zeroizing<String>, CredentialError> {
    // Validate before creating any String: a fallible UTF-16 conversion can
    // otherwise discard a partially decoded, non-zeroizing secret allocation.
    if !(8..=256).contains(&password.len())
        || !password.iter().all(|unit| (0x21..=0x7e).contains(unit))
    {
        return Err(CredentialError::Rejected);
    }
    let mut key = Zeroizing::new(String::with_capacity(password.len()));
    for unit in password {
        key.push(*unit as u8 as char);
    }
    Ok(key)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CredentialError {
    Cancelled,
    #[cfg(not(windows))]
    Unavailable,
    Rejected,
    System,
}

impl CredentialError {
    pub fn code(self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            #[cfg(not(windows))]
            Self::Unavailable => "windows_only",
            Self::Rejected => "credential_rejected",
            Self::System => "credential_error",
        }
    }
}

#[derive(Default)]
pub struct CredentialStore;

impl CredentialStore {
    pub fn configured(&self) -> Result<bool, CredentialError> {
        native::configured()
    }

    pub fn enter_with_native_dialog(&self) -> Result<bool, CredentialError> {
        native::prompt_and_save()
    }

    pub fn read(&self) -> Result<Zeroizing<String>, CredentialError> {
        native::read()
    }

    pub fn delete(&self) -> Result<bool, CredentialError> {
        native::delete()
    }
}

#[cfg(not(windows))]
mod native {
    use super::{CredentialError, Zeroizing};

    pub fn configured() -> Result<bool, CredentialError> {
        Err(CredentialError::Unavailable)
    }

    pub fn prompt_and_save() -> Result<bool, CredentialError> {
        Err(CredentialError::Unavailable)
    }

    pub fn read() -> Result<Zeroizing<String>, CredentialError> {
        Err(CredentialError::Unavailable)
    }

    pub fn delete() -> Result<bool, CredentialError> {
        Err(CredentialError::Unavailable)
    }
}

#[cfg(windows)]
mod native {
    use super::{
        decode_blob, decode_native_password, valid_key, CredentialError, Zeroizing, TARGET,
    };
    use std::{ffi::c_void, iter, mem, ptr};
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_NOT_FOUND};
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredUIPromptForCredentialsW, CredWriteW, CREDENTIALW,
        CREDUI_FLAGS_ALWAYS_SHOW_UI, CREDUI_FLAGS_DO_NOT_PERSIST, CREDUI_FLAGS_GENERIC_CREDENTIALS,
        CREDUI_INFOW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
    };
    use zeroize::Zeroize;

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(iter::once(0)).collect()
    }

    pub fn configured() -> Result<bool, CredentialError> {
        configured_at(TARGET)
    }

    fn configured_at(target_name: &str) -> Result<bool, CredentialError> {
        // Mere presence must not report a malformed blob as a usable key.
        // The validated temporary secret is zeroized as it leaves this match.
        match read_at(target_name) {
            Ok(_) => Ok(true),
            Err(CredentialError::Rejected) => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub fn prompt_and_save() -> Result<bool, CredentialError> {
        let target = wide(TARGET);
        let message = wide("Introduce la clave NVIDIA. Se guardará en el Administrador de credenciales de esta cuenta de Windows.");
        let caption = wide("Aster Desktop: clave de NVIDIA");
        let info = CREDUI_INFOW {
            cbSize: mem::size_of::<CREDUI_INFOW>() as u32,
            hwndParent: ptr::null_mut(),
            pszMessageText: message.as_ptr(),
            pszCaptionText: caption.as_ptr(),
            hbmBanner: ptr::null_mut(),
        };
        let mut username = Zeroizing::new(vec![0_u16; 513]);
        let default_user = wide("nvidia-api");
        username[..default_user.len()].copy_from_slice(&default_user);
        let mut password = Zeroizing::new(vec![0_u16; 257]);
        let mut save = 0;
        let flags = CREDUI_FLAGS_GENERIC_CREDENTIALS
            | CREDUI_FLAGS_DO_NOT_PERSIST
            | CREDUI_FLAGS_ALWAYS_SHOW_UI;
        // DO_NOT_PERSIST keeps the OS prompt from implicitly storing any
        // credential. Our explicit CredWriteW uses exactly one fixed target.
        // SAFETY: all buffers are live and sized to the values passed.
        let result = unsafe {
            CredUIPromptForCredentialsW(
                &info,
                target.as_ptr(),
                ptr::null(),
                0,
                username.as_mut_ptr(),
                username.len() as u32,
                password.as_mut_ptr(),
                password.len() as u32,
                &mut save,
                flags,
            )
        };
        if result == 1223 {
            return Err(CredentialError::Cancelled);
        }
        if result != 0 {
            return Err(CredentialError::System);
        }
        let end = password
            .iter()
            .position(|c| *c == 0)
            .unwrap_or(password.len());
        let supplied = decode_native_password(&password[..end])?;
        write_at(TARGET, &supplied)?;
        Ok(true)
    }

    fn write_at(target_name: &str, key: &str) -> Result<(), CredentialError> {
        if !valid_key(key) {
            return Err(CredentialError::Rejected);
        }
        // Use UTF-16LE without a terminator for native credential-UI
        // interoperability; the reader also accepts the previous ASCII form.
        let mut blob = Zeroizing::new(Vec::with_capacity(key.len() * 2));
        for byte in key.bytes() {
            blob.extend_from_slice(&[byte, 0]);
        }
        write_blob_at(target_name, &mut blob)
    }

    fn write_blob_at(target_name: &str, blob: &mut [u8]) -> Result<(), CredentialError> {
        // Shared with disposable synthetic tests, never accepts an IPC key.
        decode_blob(blob)?;
        let target = wide(target_name);
        let user = wide("nvidia-api");
        let entry = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: target.as_ptr() as *mut u16,
            Comment: ptr::null_mut(),
            LastWritten: unsafe { mem::zeroed() },
            CredentialBlobSize: blob.len() as u32,
            CredentialBlob: blob.as_mut_ptr(),
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: ptr::null_mut(),
            TargetAlias: ptr::null_mut(),
            UserName: user.as_ptr() as *mut u16,
        };
        // SAFETY: pointers remain live throughout this synchronous call.
        if unsafe { CredWriteW(&entry, 0) } == 0 {
            return Err(CredentialError::System);
        }
        Ok(())
    }

    pub fn read() -> Result<Zeroizing<String>, CredentialError> {
        read_at(TARGET)
    }

    fn read_at(target_name: &str) -> Result<Zeroizing<String>, CredentialError> {
        let target = wide(target_name);
        let mut pointer: *mut CREDENTIALW = ptr::null_mut();
        // SAFETY: fixed NUL-terminated target and valid output pointer.
        if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut pointer) } == 0 {
            return Err(if unsafe { GetLastError() } == ERROR_NOT_FOUND {
                CredentialError::Rejected
            } else {
                CredentialError::System
            });
        }
        // Limit the blob before copying it out of the Credential Manager.
        // Always call CredFree, including for malformed or empty blobs.
        let bytes = unsafe {
            let entry = &*pointer;
            if entry.CredentialBlob.is_null() || entry.CredentialBlobSize > 512 {
                None
            } else {
                Some(Zeroizing::new(
                    std::slice::from_raw_parts(
                        entry.CredentialBlob,
                        entry.CredentialBlobSize as usize,
                    )
                    .to_vec(),
                ))
            }
        };
        // SAFETY: CredReadW returned a writable allocation owned until
        // CredFree. Bound to the documented maximum Generic Credential blob.
        unsafe {
            let entry = &*pointer;
            if !entry.CredentialBlob.is_null() && entry.CredentialBlobSize <= 5 * 512 {
                std::slice::from_raw_parts_mut(
                    entry.CredentialBlob,
                    entry.CredentialBlobSize as usize,
                )
                .zeroize();
            }
            CredFree(pointer.cast::<c_void>());
        }
        let bytes = bytes.ok_or(CredentialError::Rejected)?;
        decode_blob(&bytes)
    }

    pub fn delete() -> Result<bool, CredentialError> {
        delete_at(TARGET)
    }

    fn delete_at(target_name: &str) -> Result<bool, CredentialError> {
        let target = wide(target_name);
        // SAFETY: the target is NUL-terminated and we delete only our fixed
        // Generic Credential under the current Windows user profile.
        if unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) } != 0 {
            return Ok(true);
        }
        if unsafe { GetLastError() } == ERROR_NOT_FOUND {
            Ok(false)
        } else {
            Err(CredentialError::System)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // This deliberately bypasses admission only to exercise malformed
        // stored blobs. It cannot name the real app credential or cross IPC.
        fn write_malformed_fixture_blob(target_name: &str, blob: &mut [u8]) {
            assert!(target_name.starts_with(&format!("{TARGET}/synthetic-ci-")));
            assert!(blob.len() <= 5 * 512);
            let target = wide(target_name);
            let user = wide("synthetic-fixture");
            let entry = CREDENTIALW {
                Flags: 0,
                Type: CRED_TYPE_GENERIC,
                TargetName: target.as_ptr() as *mut u16,
                Comment: ptr::null_mut(),
                LastWritten: unsafe { mem::zeroed() },
                CredentialBlobSize: blob.len() as u32,
                CredentialBlob: blob.as_mut_ptr(),
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                AttributeCount: 0,
                Attributes: ptr::null_mut(),
                TargetAlias: ptr::null_mut(),
                UserName: user.as_ptr() as *mut u16,
            };
            // SAFETY: only a disposable synthetic target; all buffers remain
            // live for the call. No key is printed or read from a real target.
            assert_ne!(unsafe { CredWriteW(&entry, 0) }, 0);
        }

        #[test]
        fn fixed_target_and_key_admission() {
            assert_eq!(
                TARGET,
                "comzulca/aster-desktop/nvidia/integrate.api.nvidia.com"
            );
            assert!(!valid_key(""));
            assert!(!valid_key("a key with whitespace"));
            assert!(valid_key("nvapi-test-key"));
        }

        #[test]
        fn synthetic_credential_round_trip_uses_a_disposable_target() {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time available")
                .as_nanos();
            let target = format!("{TARGET}/synthetic-ci-{}-{unique}", std::process::id());
            struct Cleanup(String);
            impl Drop for Cleanup {
                fn drop(&mut self) {
                    let _ = delete_at(&self.0);
                }
            }
            let _cleanup = Cleanup(target.clone());
            assert!(!configured_at(&target).unwrap());
            write_at(&target, "synthetic-fixture-key-123").unwrap();
            assert!(configured_at(&target).unwrap());
            assert_eq!(
                read_at(&target).unwrap().as_str(),
                "synthetic-fixture-key-123"
            );
            // Legacy ASCII blobs remain readable after the representation
            // change. No real app target is modified by this fixture.
            let mut legacy = Zeroizing::new(b"synthetic-legacy-key-123".to_vec());
            write_blob_at(&target, &mut legacy).unwrap();
            assert_eq!(
                read_at(&target).unwrap().as_str(),
                "synthetic-legacy-key-123"
            );
            assert!(delete_at(&target).unwrap());
            assert!(!configured_at(&target).unwrap());
        }

        #[test]
        fn malformed_stored_blobs_are_not_configured_on_a_disposable_target() {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time available")
                .as_nanos();
            let target = format!(
                "{TARGET}/synthetic-ci-malformed-{}-{unique}",
                std::process::id()
            );
            struct Cleanup(String);
            impl Drop for Cleanup {
                fn drop(&mut self) {
                    let _ = delete_at(&self.0);
                }
            }
            let _cleanup = Cleanup(target.clone());
            assert!(!configured_at(&target).unwrap());
            let mut terminated: Vec<u8> = b"synthetic-key".iter().flat_map(|b| [*b, 0]).collect();
            terminated.extend_from_slice(&[0, 0]);
            for blob in [
                b"synthetic key with spaces".to_vec(),
                b"synthetic\0key".to_vec(),
                terminated,
                vec![b'x'; 514],
            ] {
                let mut blob = Zeroizing::new(blob);
                write_malformed_fixture_blob(&target, &mut blob);
                assert!(!configured_at(&target).unwrap());
                assert_eq!(read_at(&target).err(), Some(CredentialError::Rejected));
            }
            assert!(delete_at(&target).unwrap());
        }
    }
}

#[cfg(test)]
mod representation_tests {
    use super::*;

    #[test]
    fn legacy_ascii_and_native_utf16_decode_identically() {
        let key = "synthetic-fixture-key-123";
        let wide: Vec<u8> = key.bytes().flat_map(|byte| [byte, 0]).collect();
        assert_eq!(decode_blob(key.as_bytes()).unwrap().as_str(), key);
        assert_eq!(decode_blob(&wide).unwrap().as_str(), key);
        let max_key = "x".repeat(256);
        let max_wide: Vec<u8> = max_key.bytes().flat_map(|byte| [byte, 0]).collect();
        assert!(decode_blob(&max_wide).is_ok());
    }

    #[test]
    fn ascii_and_utf16_password_length_boundaries_match() {
        for length in [7, 8, 256, 257] {
            let key = "x".repeat(length);
            let wide: Vec<u8> = key.bytes().flat_map(|byte| [byte, 0]).collect();
            let native: Vec<u16> = key.encode_utf16().collect();
            let accepted = (8..=256).contains(&length);
            assert_eq!(decode_blob(key.as_bytes()).is_ok(), accepted);
            assert_eq!(decode_blob(&wide).is_ok(), accepted);
            assert_eq!(decode_native_password(&native).is_ok(), accepted);
        }
    }

    #[test]
    fn native_password_rejects_non_ascii_before_secret_string_allocation() {
        for unit in [0, 0x20, 0x7f, 0x80, 0xd800, 0xdc00] {
            let mut password = vec![b'x' as u16; 8];
            password[7] = unit;
            assert_eq!(
                decode_native_password(&password).err(),
                Some(CredentialError::Rejected)
            );
        }
    }

    #[test]
    fn malformed_and_oversized_password_representations_deny() {
        for blob in [
            vec![],
            b"short".to_vec(),
            b"spaces are rejected".to_vec(),
            b"embedded\0nul-key".to_vec(),
            vec![b'a'; 257],
            vec![0; 16],
            vec![b'a', 0, b'b'],
            vec![0xff; 16],
            vec![b'a'; 513],
        ] {
            assert!(decode_blob(&blob).is_err());
        }
        let mut terminated: Vec<u8> = b"synthetic-key".iter().flat_map(|b| [*b, 0]).collect();
        terminated.extend_from_slice(&[0, 0]);
        assert!(decode_blob(&terminated).is_err());
        let mut non_ascii: Vec<u8> = b"synthetic-key".iter().flat_map(|b| [*b, 0]).collect();
        non_ascii[1] = 1;
        assert!(decode_blob(&non_ascii).is_err());
    }
}
