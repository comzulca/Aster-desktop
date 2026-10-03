//! The Meta Model API credential exists only in the trusted native host and the
//! current user's Windows Credential Manager. Never put it in a command DTO.

use zeroize::Zeroizing;

const TARGET: &str = "AsterDesktop/MetaModelAPI/v1";

#[cfg(any(windows, test))]
fn valid_key(key: &str) -> bool {
    if !(8..=256).contains(&key.len()) || !key.bytes().all(|b| b.is_ascii_graphic()) {
        return false;
    }
    // The direct API also issues opaque tokens. Do not derive an account,
    // translate separators, trim or repair either native representation.
    if let Some(suffix) = key.strip_prefix("LLM_") {
        return suffix.len() >= 12
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    }
    let mut parts = key.split('|');
    matches!((parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("LLM"), Some(id), Some(secret), None)
        if !id.is_empty() && id.len() <= 32 && id.bytes().all(|b| b.is_ascii_digit()) && !secret.is_empty())
}

/// Generic Credential blobs are application-defined. Accept bounded ASCII
/// format and native UTF-16LE ASCII passwords, never trim or repair a secret.
#[cfg(any(windows, test))]
fn decode_blob(blob: &[u8]) -> Result<Zeroizing<String>, CredentialError> {
    if (8..=256).contains(&blob.len()) && blob.iter().all(u8::is_ascii_graphic) {
        let key = std::str::from_utf8(blob).map_err(|_| CredentialError::Rejected)?;
        if !valid_key(key) {
            return Err(CredentialError::Rejected);
        }
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
    if !valid_key(&key) {
        return Err(CredentialError::Rejected);
    }
    Ok(key)
}

#[cfg(any(windows, test))]
fn admit_fixture_password(supplied: Zeroizing<String>) -> Result<(), CredentialError> {
    // Consume the zeroizing buffer even on rejection. A diagnostic never
    // returns supplied bytes and can admit only this known non-secret value.
    if !valid_key(&supplied) || supplied.as_str() != crate::credential_fixture::SYNTHETIC_KEY {
        return Err(CredentialError::Rejected);
    }
    Ok(())
}

/// A validated prompted key stays in this zeroising allocation. Revocation is
/// host-only and must finish durably before the explicit fixed-target writer.
/// No key is returned to the caller or supplied to the revocation callback.
#[cfg(any(windows, test))]
fn save_prompt_after<B, W>(
    supplied: Result<Zeroizing<String>, CredentialError>,
    before_write: B,
    write: W,
) -> Result<bool, CredentialError>
where
    B: FnOnce() -> Result<(), CredentialError>,
    W: FnOnce(&str) -> Result<(), CredentialError>,
{
    let supplied = supplied?;
    if !valid_key(&supplied) {
        return Err(CredentialError::Rejected);
    }
    before_write()?;
    write(&supplied)?;
    Ok(true)
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

    pub fn enter_with_native_dialog_after<F>(
        &self,
        owner: usize,
        before_write: F,
    ) -> Result<bool, CredentialError>
    where
        F: FnOnce() -> Result<(), CredentialError>,
    {
        native::prompt_and_save_after(owner, before_write)
    }

    pub fn read(&self) -> Result<Zeroizing<String>, CredentialError> {
        native::read()
    }

    pub fn delete(&self) -> Result<bool, CredentialError> {
        native::delete()
    }
}

// These diagnostic-only entry points accept an opaque generated target, never
// a string supplied by argv, environment, a renderer, or a provider. The
// diagnostic cannot call the production store through this surface.
#[cfg(windows)]
pub(crate) fn fixture_present(
    target: &crate::credential_fixture::FixtureTarget,
) -> Result<bool, CredentialError> {
    native::fixture_present(target)
}

#[cfg(windows)]
pub(crate) fn fixture_snapshot(
    target: &crate::credential_fixture::FixtureTarget,
) -> Result<crate::credential_fixture::FixtureSnapshot, CredentialError> {
    native::fixture_snapshot(target)
}

#[cfg(windows)]
pub(crate) fn fixture_prompt(
    target: &crate::credential_fixture::FixtureTarget,
) -> Result<bool, CredentialError> {
    native::fixture_prompt(target)
}

#[cfg(windows)]
pub(crate) fn fixture_write(
    target: &crate::credential_fixture::FixtureTarget,
) -> Result<(), CredentialError> {
    native::fixture_write(target)
}

#[cfg(windows)]
pub(crate) fn fixture_delete(
    target: &crate::credential_fixture::FixtureTarget,
    expected: &crate::credential_fixture::FixtureSnapshot,
) -> Result<(), CredentialError> {
    native::fixture_delete(target, expected)
}

#[cfg(not(windows))]
mod native {
    use super::{CredentialError, Zeroizing};

    pub fn configured() -> Result<bool, CredentialError> {
        Err(CredentialError::Unavailable)
    }

    pub fn prompt_and_save_after<F>(
        _owner: usize,
        _before_write: F,
    ) -> Result<bool, CredentialError>
    where
        F: FnOnce() -> Result<(), CredentialError>,
    {
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
        admit_fixture_password, decode_blob, decode_native_password, save_prompt_after, valid_key,
        CredentialError, Zeroizing, TARGET,
    };
    use std::{ffi::c_void, iter, mem, ptr};
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_NOT_FOUND};
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredUIPromptForCredentialsW, CredWriteW, CREDENTIALW,
        CREDUI_FLAGS_ALWAYS_SHOW_UI, CREDUI_FLAGS_DO_NOT_PERSIST, CREDUI_FLAGS_GENERIC_CREDENTIALS,
        CREDUI_INFOW, CRED_PERSIST_ENTERPRISE, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
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

    pub fn prompt_and_save_after<F>(owner: usize, before_write: F) -> Result<bool, CredentialError>
    where
        F: FnOnce() -> Result<(), CredentialError>,
    {
        crate::native_review::validate_owner_handle(owner)
            .map_err(|_| CredentialError::Rejected)?;
        let supplied = prompt_at_owned(
            TARGET,
            "meta-model-api",
            "Enter a direct Meta Model API key authorised for Aster. It will be saved in Windows Credential Manager for this Windows account. Do not use a Muse Code subscription credential.",
            "Aster Desktop: Meta Model API key",
            owner,
        )?;
        save_prompt_after(Ok(supplied), before_write, |key| {
            // A lost/foreign native owner cannot complete the write. A prior
            // successful revocation remains conservative if this check fails.
            crate::native_review::validate_owner_handle(owner)
                .map_err(|_| CredentialError::Rejected)?;
            write_at(TARGET, key)
        })
    }

    // The native UI never persists implicitly. A caller must independently
    // admit the result before its explicit, target-bound write.
    fn prompt_at(
        target_name: &str,
        default_username: &str,
        message_text: &str,
        caption_text: &str,
    ) -> Result<Zeroizing<String>, CredentialError> {
        prompt_at_owned(target_name, default_username, message_text, caption_text, 0)
    }

    fn prompt_at_owned(
        target_name: &str,
        default_username: &str,
        message_text: &str,
        caption_text: &str,
        owner: usize,
    ) -> Result<Zeroizing<String>, CredentialError> {
        let target = wide(target_name);
        let message = wide(message_text);
        let caption = wide(caption_text);
        let info = CREDUI_INFOW {
            cbSize: mem::size_of::<CREDUI_INFOW>() as u32,
            hwndParent: owner as *mut c_void,
            pszMessageText: message.as_ptr(),
            pszCaptionText: caption.as_ptr(),
            hbmBanner: ptr::null_mut(),
        };
        let mut username = Zeroizing::new(vec![0_u16; 513]);
        let default_user = wide(default_username);
        username[..default_user.len()].copy_from_slice(&default_user);
        let mut password = Zeroizing::new(vec![0_u16; 257]);
        let mut save = 0;
        let flags = CREDUI_FLAGS_GENERIC_CREDENTIALS
            | CREDUI_FLAGS_DO_NOT_PERSIST
            | CREDUI_FLAGS_ALWAYS_SHOW_UI;
        // DO_NOT_PERSIST keeps the OS prompt from implicitly storing anything.
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
        decode_native_password(&password[..end])
    }

    fn write_at(target_name: &str, key: &str) -> Result<(), CredentialError> {
        write_at_user(target_name, key, "meta-model-api")
    }

    fn write_at_user(target_name: &str, key: &str, username: &str) -> Result<(), CredentialError> {
        if !valid_key(key) {
            return Err(CredentialError::Rejected);
        }
        // Use UTF-16LE without a terminator for native credential-UI
        // interoperability; the reader also accepts the previous ASCII form.
        let mut blob = Zeroizing::new(Vec::with_capacity(key.len() * 2));
        for byte in key.bytes() {
            blob.extend_from_slice(&[byte, 0]);
        }
        write_blob_at_user(target_name, &mut blob, username)
    }

    #[cfg(test)]
    fn write_blob_at(target_name: &str, blob: &mut [u8]) -> Result<(), CredentialError> {
        write_blob_at_user(target_name, blob, "meta-model-api")
    }

    fn write_blob_at_user(
        target_name: &str,
        blob: &mut [u8],
        username: &str,
    ) -> Result<(), CredentialError> {
        // Shared with disposable synthetic tests, never accepts an IPC key.
        decode_blob(blob)?;
        let target = wide(target_name);
        let user = wide(username);
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

    fn fixture_name(
        target: &crate::credential_fixture::FixtureTarget,
    ) -> Result<&str, CredentialError> {
        target.validated_name().ok_or(CredentialError::Rejected)
    }

    struct FixtureAllocation(*mut CREDENTIALW);

    impl Drop for FixtureAllocation {
        fn drop(&mut self) {
            // This frees and scrubs only an OS read allocation. It NEVER
            // deletes a persisted credential on Drop, error or interruption.
            // SAFETY: the allocation is owned by CredReadW until CredFree.
            unsafe {
                let entry = &*self.0;
                if !entry.CredentialBlob.is_null() && entry.CredentialBlobSize <= 5 * 512 {
                    std::slice::from_raw_parts_mut(
                        entry.CredentialBlob,
                        entry.CredentialBlobSize as usize,
                    )
                    .zeroize();
                }
                CredFree(self.0.cast::<c_void>());
            }
        }
    }

    fn fixture_allocation(
        target: &crate::credential_fixture::FixtureTarget,
    ) -> Result<Option<FixtureAllocation>, CredentialError> {
        let target = wide(fixture_name(target)?);
        let mut pointer: *mut CREDENTIALW = ptr::null_mut();
        // Raw presence is deliberately independent of decoding. A malformed
        // existing entry is still present and must block a diagnostic write.
        // SAFETY: validated fixture namespace and a live output pointer.
        if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut pointer) } == 0 {
            return if unsafe { GetLastError() } == ERROR_NOT_FOUND {
                Ok(None)
            } else {
                Err(CredentialError::System)
            };
        }
        if pointer.is_null() {
            return Err(CredentialError::System);
        }
        Ok(Some(FixtureAllocation(pointer)))
    }

    pub(super) fn fixture_present(
        target: &crate::credential_fixture::FixtureTarget,
    ) -> Result<bool, CredentialError> {
        // Never copy, decode or output a password during raw presence checks.
        Ok(fixture_allocation(target)?.is_some())
    }

    fn exact_wide(pointer: *const u16, expected: &str) -> bool {
        if pointer.is_null() {
            return false;
        }
        // SAFETY: CredReadW supplies NUL-terminated metadata strings. Compare
        // only the bounded expected value and terminator, stopping on mismatch.
        // Test callers provide live, NUL-terminated fixture arrays.
        for (index, unit) in expected.encode_utf16().chain(iter::once(0)).enumerate() {
            if unsafe { *pointer.add(index) } != unit {
                return false;
            }
        }
        true
    }

    fn empty_wide(pointer: *const u16) -> bool {
        pointer.is_null() || exact_wide(pointer, "")
    }

    fn fixture_metadata_matches(entry: &CREDENTIALW, target_name: &str) -> bool {
        entry.Type == CRED_TYPE_GENERIC
            && entry.Flags == 0
            && matches!(
                entry.Persist,
                CRED_PERSIST_LOCAL_MACHINE | CRED_PERSIST_ENTERPRISE
            )
            && entry.AttributeCount == 0
            && empty_wide(entry.TargetAlias)
            && empty_wide(entry.Comment)
            && exact_wide(entry.TargetName, target_name)
            && exact_wide(
                entry.UserName,
                crate::credential_fixture::SYNTHETIC_USERNAME,
            )
    }

    pub(super) fn fixture_snapshot(
        target: &crate::credential_fixture::FixtureTarget,
    ) -> Result<crate::credential_fixture::FixtureSnapshot, CredentialError> {
        let target_name = fixture_name(target)?;
        let allocation = fixture_allocation(target)?.ok_or(CredentialError::Rejected)?;
        // SAFETY: allocation remains owned and live through this scope.
        let entry = unsafe { &*allocation.0 };
        if !fixture_metadata_matches(entry, target_name)
            || entry.CredentialBlob.is_null()
            || entry.CredentialBlobSize > 512
        {
            return Err(CredentialError::Rejected);
        }
        // SAFETY: the validated size refers to the live OS allocation.
        let bytes = Zeroizing::new(unsafe {
            std::slice::from_raw_parts(entry.CredentialBlob, entry.CredentialBlobSize as usize)
                .to_vec()
        });
        admit_fixture_password(decode_blob(&bytes)?)?;
        Ok(crate::credential_fixture::FixtureSnapshot {
            target_name: target_name.to_owned(),
            last_written: ((entry.LastWritten.dwHighDateTime as u64) << 32)
                | entry.LastWritten.dwLowDateTime as u64,
            persistence: entry.Persist,
        })
    }

    pub(super) fn fixture_prompt(
        target: &crate::credential_fixture::FixtureTarget,
    ) -> Result<bool, CredentialError> {
        let supplied = match prompt_at(
            fixture_name(target)?,
            crate::credential_fixture::SYNTHETIC_USERNAME,
            "OFFLINE SYNTHETIC TEST. Never enter a real key. Use only 'LLM|0|synthetic-fixture-key-123'. This dialogue does not save anything automatically.",
            "Aster: synthetic Meta credential test (offline)",
        ) {
            Ok(value) => value,
            Err(CredentialError::Cancelled) => return Ok(false),
            Err(error) => return Err(error),
        };
        // Wrong input is zeroized as supplied leaves this scope. No returned
        // key bytes and no write occur in the prompt, even after acceptance.
        admit_fixture_password(supplied)?;
        Ok(true)
    }

    pub(super) fn fixture_write(
        target: &crate::credential_fixture::FixtureTarget,
    ) -> Result<(), CredentialError> {
        if fixture_present(target)? {
            return Err(CredentialError::Rejected);
        }
        // A last raw-absence check is not atomic CAS against a malicious
        // same-user racer. The closed namespace still excludes production.
        write_at_user(
            fixture_name(target)?,
            crate::credential_fixture::SYNTHETIC_KEY,
            crate::credential_fixture::SYNTHETIC_USERNAME,
        )
    }

    pub(super) fn fixture_delete(
        target: &crate::credential_fixture::FixtureTarget,
        expected: &crate::credential_fixture::FixtureSnapshot,
    ) -> Result<(), CredentialError> {
        if &fixture_snapshot(target)? != expected {
            return Err(CredentialError::Rejected);
        }
        // No unconditional cleanup exists. This non-atomic revalidation is
        // allowed only after explicit native fixture consent.
        if !delete_at(fixture_name(target)?)? {
            return Err(CredentialError::Rejected);
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn fixture_metadata_admission_rejects_foreign_entries_with_known_fake_password() {
            // Pure stack-backed metadata fixtures: no Credential Manager API
            // or real target is named or called by this test.
            let target_name = "AsterDesktop/MetaModelAPI/credential-gui-fixture/00000001-00000000000000000000000000000002-0000000000000003";
            let mut target = wide(target_name);
            let mut username = wide(crate::credential_fixture::SYNTHETIC_USERNAME);
            let mut foreign_username = wide("other-synthetic-fixture");
            let mut foreign_target = wide("unrelated-synthetic-target");
            let mut annotation = wide("foreign-synthetic-metadata");
            let mut blob = b"LLM|0|synthetic-fixture-key-123".to_vec();
            let baseline = CREDENTIALW {
                Flags: 0,
                Type: CRED_TYPE_GENERIC,
                TargetName: target.as_mut_ptr(),
                Comment: ptr::null_mut(),
                LastWritten: unsafe { mem::zeroed() },
                CredentialBlobSize: blob.len() as u32,
                CredentialBlob: blob.as_mut_ptr(),
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                AttributeCount: 0,
                Attributes: ptr::null_mut(),
                TargetAlias: ptr::null_mut(),
                UserName: username.as_mut_ptr(),
            };
            assert!(fixture_metadata_matches(&baseline, target_name));
            let mut entry = baseline;
            entry.Persist = CRED_PERSIST_ENTERPRISE;
            assert!(fixture_metadata_matches(&entry, target_name));
            for field in 0..9 {
                let mut entry = baseline;
                match field {
                    0 => entry.UserName = foreign_username.as_mut_ptr(),
                    1 => entry.UserName = ptr::null_mut(),
                    2 => entry.TargetName = foreign_target.as_mut_ptr(),
                    3 => entry.TargetName = ptr::null_mut(),
                    4 => entry.Type = 2,
                    5 => entry.Flags = 1,
                    6 => entry.Persist = 1,
                    7 => entry.AttributeCount = 1,
                    8 => entry.TargetAlias = annotation.as_mut_ptr(),
                    _ => unreachable!(),
                }
                assert!(!fixture_metadata_matches(&entry, target_name));
            }
            let mut entry = baseline;
            entry.Comment = annotation.as_mut_ptr();
            assert!(!fixture_metadata_matches(&entry, target_name));
        }

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
            assert_eq!(TARGET, "AsterDesktop/MetaModelAPI/v1");
            assert!(!valid_key(""));
            assert!(!valid_key("a key with whitespace"));
            assert!(valid_key("LLM|0|synthetic-fixture-key-123"));
            assert!(!valid_key("nvapi-test-key"));
        }

        #[test]
        #[ignore = "requires separately authorised disposable native Credential Manager fixture"]
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
            write_at(&target, "LLM|0|synthetic-fixture-key-123").unwrap();
            assert!(configured_at(&target).unwrap());
            assert_eq!(
                read_at(&target).unwrap().as_str(),
                "LLM|0|synthetic-fixture-key-123"
            );
            // Legacy ASCII blobs remain readable after the representation
            // change. No real app target is modified by this fixture.
            let mut legacy = Zeroizing::new(b"LLM|0|synthetic-ascii-key-123".to_vec());
            write_blob_at(&target, &mut legacy).unwrap();
            assert_eq!(
                read_at(&target).unwrap().as_str(),
                "LLM|0|synthetic-ascii-key-123"
            );
            assert!(delete_at(&target).unwrap());
            assert!(!configured_at(&target).unwrap());
        }

        #[test]
        #[ignore = "requires separately authorised disposable native Credential Manager fixture"]
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

    const OPAQUE_SYNTHETIC_KEY: &str = "LLM_0_SYNTHETIC_NOT_A_SECRET";

    #[test]
    fn opaque_synthetic_key_decodes_exact_ascii_utf16_and_native_bytes() {
        let wide: Vec<u8> = OPAQUE_SYNTHETIC_KEY
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        let native: Vec<u16> = OPAQUE_SYNTHETIC_KEY.encode_utf16().collect();
        assert!(valid_key(OPAQUE_SYNTHETIC_KEY));
        assert_eq!(
            decode_blob(OPAQUE_SYNTHETIC_KEY.as_bytes())
                .unwrap()
                .as_str(),
            OPAQUE_SYNTHETIC_KEY
        );
        assert_eq!(decode_blob(&wide).unwrap().as_str(), OPAQUE_SYNTHETIC_KEY);
        assert_eq!(
            decode_native_password(&native).unwrap().as_str(),
            OPAQUE_SYNTHETIC_KEY
        );
        // The diagnostic still admits its original one known value only.
        assert_eq!(
            admit_fixture_password(Zeroizing::new(OPAQUE_SYNTHETIC_KEY.into())),
            Err(CredentialError::Rejected)
        );
    }

    #[test]
    fn opaque_syntax_length_alphabet_and_all_decoder_boundaries_are_closed() {
        // Generated bytes are synthetic boundary cases, never a real key.
        for suffix_length in [0, 11, 12, 252, 253] {
            let key = format!("LLM_{}", "x".repeat(suffix_length));
            let wide: Vec<u8> = key.encode_utf16().flat_map(u16::to_le_bytes).collect();
            let native: Vec<u16> = key.encode_utf16().collect();
            let accepted = (12..=252).contains(&suffix_length);
            assert_eq!(valid_key(&key), accepted);
            assert_eq!(decode_blob(key.as_bytes()).is_ok(), accepted);
            assert_eq!(decode_blob(&wide).is_ok(), accepted);
            assert_eq!(decode_native_password(&native).is_ok(), accepted);
        }
        let allowed = format!("LLM_{}", "xX09_-.".repeat(2));
        assert!(valid_key(&allowed));
        for bad in [
            ' ', '\t', '\r', '\n', '\0', '\u{007f}', 'é', '|', '/', '+', '=', ':', '!', '"',
        ] {
            let key = format!("{OPAQUE_SYNTHETIC_KEY}{bad}");
            let wide: Vec<u8> = key.encode_utf16().flat_map(u16::to_le_bytes).collect();
            let native: Vec<u16> = key.encode_utf16().collect();
            assert!(!valid_key(&key));
            assert!(decode_blob(key.as_bytes()).is_err());
            assert!(decode_blob(&wide).is_err());
            assert!(decode_native_password(&native).is_err());
        }
        for prefix in ["llm_", "LLM-", "LLM.", "Bearer ", "nvapi-"] {
            let key = format!("{prefix}{}", "x".repeat(20));
            assert!(!valid_key(&key));
            assert!(decode_blob(key.as_bytes()).is_err());
        }
        for key in [
            format!(" {OPAQUE_SYNTHETIC_KEY}"),
            format!("{OPAQUE_SYNTHETIC_KEY} "),
        ] {
            assert!(!valid_key(&key));
            assert!(decode_blob(key.as_bytes()).is_err());
        }
    }

    #[test]
    fn opaque_prompt_preserves_revocation_before_exact_writer_without_real_custody() {
        let revoked = std::cell::Cell::new(false);
        let writes = std::cell::Cell::new(0);
        assert_eq!(
            save_prompt_after(
                Ok(Zeroizing::new(OPAQUE_SYNTHETIC_KEY.into())),
                || {
                    revoked.set(true);
                    Ok(())
                },
                |key| {
                    assert!(revoked.get());
                    assert_eq!(key, OPAQUE_SYNTHETIC_KEY);
                    writes.set(writes.get() + 1);
                    Ok(())
                },
            ),
            Ok(true)
        );
        assert_eq!(writes.get(), 1);
    }

    #[test]
    fn prompted_key_revokes_identity_before_the_explicit_writer() {
        let events = std::cell::RefCell::new(Vec::new());
        let identity_valid = std::cell::Cell::new(true);
        assert_eq!(
            save_prompt_after(
                Ok(Zeroizing::new(
                    crate::credential_fixture::SYNTHETIC_KEY.into()
                )),
                || {
                    events.borrow_mut().push("durable_revocation");
                    identity_valid.set(false);
                    Ok(())
                },
                |key| {
                    assert!(!identity_valid.get());
                    assert_eq!(key, crate::credential_fixture::SYNTHETIC_KEY);
                    events.borrow_mut().push("fixed_target_write");
                    Ok(())
                },
            ),
            Ok(true),
        );
        assert_eq!(
            *events.borrow(),
            ["durable_revocation", "fixed_target_write"],
        );
    }

    #[test]
    fn cancelled_or_invalid_prompt_never_revokes_or_writes() {
        for (supplied, expected) in [
            (Err(CredentialError::Cancelled), CredentialError::Cancelled),
            (Err(CredentialError::System), CredentialError::System),
            (
                Ok(Zeroizing::new("invalid-prompted-value".into())),
                CredentialError::Rejected,
            ),
        ] {
            let revocations = std::cell::Cell::new(0);
            let writes = std::cell::Cell::new(0);
            assert_eq!(
                save_prompt_after(
                    supplied,
                    || {
                        revocations.set(revocations.get() + 1);
                        Ok(())
                    },
                    |_| {
                        writes.set(writes.get() + 1);
                        Ok(())
                    },
                ),
                Err(expected),
            );
            assert_eq!(revocations.get(), 0);
            assert_eq!(writes.get(), 0);
        }
    }

    #[test]
    fn failed_durable_revocation_never_writes_the_prompted_key() {
        let revocations = std::cell::Cell::new(0);
        let writes = std::cell::Cell::new(0);
        assert_eq!(
            save_prompt_after(
                Ok(Zeroizing::new(
                    crate::credential_fixture::SYNTHETIC_KEY.into()
                )),
                || {
                    revocations.set(revocations.get() + 1);
                    Err(CredentialError::System)
                },
                |_| {
                    writes.set(writes.get() + 1);
                    Ok(())
                },
            ),
            Err(CredentialError::System),
        );
        assert_eq!(revocations.get(), 1);
        assert_eq!(writes.get(), 0);
    }

    #[test]
    fn failed_key_write_does_not_restore_revoked_identity_or_retry() {
        let identity_valid = std::cell::Cell::new(true);
        let writes = std::cell::Cell::new(0);
        assert_eq!(
            save_prompt_after(
                Ok(Zeroizing::new(
                    crate::credential_fixture::SYNTHETIC_KEY.into()
                )),
                || {
                    identity_valid.set(false);
                    Ok(())
                },
                |_| {
                    writes.set(writes.get() + 1);
                    Err(CredentialError::System)
                },
            ),
            Err(CredentialError::System),
        );
        assert!(!identity_valid.get());
        assert_eq!(writes.get(), 1);
    }

    #[test]
    fn direct_meta_key_syntax_is_closed_not_an_entitlement_claim() {
        assert!(valid_key(crate::credential_fixture::SYNTHETIC_KEY));
        for key in [
            "nvapi-test-key",
            "synthetic-fixture-key-123",
            "LLM||fake",
            "LLM|abc|fake",
            "LLM|0|",
            "LLM|0|fake|extra",
            "llm|0|fake",
            " LLM|0|SYNTHETIC_NOT_A_SECRET",
        ] {
            assert!(!valid_key(key));
            assert!(decode_blob(key.as_bytes()).is_err());
        }
    }

    #[test]
    fn fixture_password_admission_consumes_only_the_known_synthetic_value() {
        assert_eq!(
            admit_fixture_password(Zeroizing::new(
                crate::credential_fixture::SYNTHETIC_KEY.to_owned(),
            )),
            Ok(()),
        );
        for value in ["", "synthetic-other-key-123", "synthetic key with spaces"] {
            assert_eq!(
                admit_fixture_password(Zeroizing::new(value.to_owned())),
                Err(CredentialError::Rejected),
            );
        }
    }

    #[test]
    fn fixture_native_admission_rejects_bounded_wrong_and_malformed_input() {
        let expected = Zeroizing::new(
            crate::credential_fixture::SYNTHETIC_KEY
                .encode_utf16()
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            decode_native_password(&expected).and_then(admit_fixture_password),
            Ok(()),
        );
        for input in [
            "synthetic-other-key-123".encode_utf16().collect(),
            vec![b'x' as u16; 7],
            vec![b'x' as u16; 8],
            vec![b'x' as u16; 256],
            vec![b'x' as u16; 257],
            "synthetic key with spaces".encode_utf16().collect(),
            "synthetic\0fixture-key".encode_utf16().collect(),
            vec![0xd800; 23],
        ] {
            let input = Zeroizing::new(input);
            assert_eq!(
                decode_native_password(&input).and_then(admit_fixture_password),
                Err(CredentialError::Rejected),
            );
        }
    }

    #[test]
    fn legacy_ascii_and_native_utf16_decode_identically() {
        let key = "LLM|0|synthetic-fixture-key-123";
        let wide: Vec<u8> = key.bytes().flat_map(|byte| [byte, 0]).collect();
        assert_eq!(decode_blob(key.as_bytes()).unwrap().as_str(), key);
        assert_eq!(decode_blob(&wide).unwrap().as_str(), key);
        let max_key = format!("{}{}", "LLM|0|", "x".repeat(250));
        let max_wide: Vec<u8> = max_key.bytes().flat_map(|byte| [byte, 0]).collect();
        assert!(decode_blob(&max_wide).is_ok());
    }

    #[test]
    fn ascii_and_utf16_password_length_boundaries_match() {
        for length in [7, 8, 256, 257] {
            let key = format!("{}{}", "LLM|0|", "x".repeat(length - 6));
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
