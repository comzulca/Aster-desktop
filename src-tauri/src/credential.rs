//! The NVIDIA credential exists only in the trusted native host and the
//! current user's Windows Credential Manager. Never put it in a command DTO.

use zeroize::Zeroizing;

const TARGET: &str = "comzulca/aster-desktop/nvidia/integrate.api.nvidia.com";

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
    use super::{CredentialError, Zeroizing, TARGET};
    use std::{ffi::c_void, iter, mem, ptr};
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_NOT_FOUND};
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredUIPromptForCredentialsW, CredWriteW, CREDENTIALW,
        CREDUI_FLAGS_ALWAYS_SHOW_UI, CREDUI_FLAGS_DO_NOT_PERSIST, CREDUI_FLAGS_GENERIC_CREDENTIALS,
        CREDUI_INFOW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
    };

    // API keys from this provider are printable ASCII bearer tokens. Refuse
    // a different format before allocating an HTTP Authorization header.
    fn valid_key(key: &str) -> bool {
        (8..=256).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_graphic())
    }

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(iter::once(0)).collect()
    }

    pub fn configured() -> Result<bool, CredentialError> {
        configured_at(TARGET)
    }

    fn configured_at(target_name: &str) -> Result<bool, CredentialError> {
        let target = wide(target_name);
        let mut pointer: *mut CREDENTIALW = ptr::null_mut();
        // SAFETY: target is NUL-terminated; CredReadW owns the returned
        // allocation until CredFree and is never exposed to the WebView.
        if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut pointer) } == 0 {
            let code = unsafe { GetLastError() };
            return if code == ERROR_NOT_FOUND {
                Ok(false)
            } else {
                Err(CredentialError::System)
            };
        }
        unsafe { CredFree(pointer.cast::<c_void>()) };
        Ok(true)
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
        let supplied = Zeroizing::new(
            String::from_utf16(&password[..end]).map_err(|_| CredentialError::Rejected)?,
        );
        if !valid_key(&supplied) {
            return Err(CredentialError::Rejected);
        }
        write_at(TARGET, &supplied)?;
        Ok(true)
    }

    fn write_at(target_name: &str, key: &str) -> Result<(), CredentialError> {
        if !valid_key(key) {
            return Err(CredentialError::Rejected);
        }
        // Windows Generic Credential blob is arbitrary bytes, not a UTF-16
        // password. This explicitly stores the NVIDIA ASCII key as UTF-8.
        let mut blob = Zeroizing::new(key.as_bytes().to_vec());
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
            if entry.CredentialBlob.is_null() || entry.CredentialBlobSize > 256 {
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
        unsafe { CredFree(pointer.cast::<c_void>()) };
        let bytes = bytes.ok_or(CredentialError::Rejected)?;
        let key = std::str::from_utf8(&bytes).map_err(|_| CredentialError::Rejected)?;
        if !valid_key(key) {
            return Err(CredentialError::Rejected);
        }
        Ok(Zeroizing::new(key.to_owned()))
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
            assert!(delete_at(&target).unwrap());
            assert!(!configured_at(&target).unwrap());
        }
    }
}
