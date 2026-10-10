//! Separate fixed current-user GitHub Credential Manager custody.
//! No renderer, model, package or caller-supplied credential target is accepted.
//! The host must serialize all operations, invalidate generations before local
//! delete/switch and confirm the authenticated account natively before save.
//! Restored custody is NOT restored account/repository/approval authority.

use crate::github_api::{AccountIdentity, TokenSet};
use serde::{Deserialize, Deserializer, Serialize};
use zeroize::Zeroizing;

pub(crate) const TARGET: &str = "AsterDesktop/GitHubDeviceFlow/v1";
const USERNAME: &str = "aster-github-device-flow";
const CONTRACT_VERSION: u32 = 1;
const MAX_BLOB_BYTES: usize = 2_560;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoreError {
    Missing,
    Rejected,
    Unavailable,
    System,
    RotationPending,
}

pub(crate) struct StoredConnection {
    pub(crate) account: AccountIdentity,
    pub(crate) tokens: TokenSet,
    pub(crate) rotation_pending: bool,
}

/// Move-only permit exists only after verified durable pending publication.
/// Drop after an uncertain request leaves custody pending and requires reconnect.
pub(crate) struct RotationPermit {
    old: StoredConnection,
    pending_blob: Zeroizing<Vec<u8>>,
}

/// Successful refresh still has no account authority. Independently retrieve and
/// compare /user before passing this move-only receipt to publish_rotation.
pub(crate) struct RefreshReceipt {
    account: AccountIdentity,
    tokens: TokenSet,
    pending_blob: Zeroizing<Vec<u8>>,
}

impl RotationPermit {
    pub(crate) fn tokens(&self) -> &TokenSet {
        &self.old.tokens
    }
    pub(crate) fn complete(self, tokens: TokenSet) -> RefreshReceipt {
        RefreshReceipt {
            account: self.old.account,
            tokens,
            pending_blob: self.pending_blob,
        }
    }
}

impl RefreshReceipt {
    pub(crate) fn tokens(&self) -> &TokenSet {
        &self.tokens
    }
    pub(crate) fn account(&self) -> &AccountIdentity {
        &self.account
    }
}

fn private_string<'de, D: Deserializer<'de>>(d: D) -> Result<Zeroizing<String>, D::Error> {
    String::deserialize(d).map(Zeroizing::new)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredWire {
    version: u32,
    app_id: u64,
    client_id: String,
    account_id: u64,
    account_login: String,
    #[serde(deserialize_with = "private_string")]
    access: Zeroizing<String>,
    #[serde(deserialize_with = "private_string")]
    refresh: Zeroizing<String>,
    issued_at: u64,
    access_expires_at: u64,
    refresh_expires_at: u64,
    rotation_pending: bool,
}

#[derive(Serialize)]
struct StoredRef<'a> {
    version: u32,
    app_id: u64,
    client_id: &'static str,
    account_id: u64,
    account_login: &'a str,
    access: &'a str,
    refresh: &'a str,
    issued_at: u64,
    access_expires_at: u64,
    refresh_expires_at: u64,
    rotation_pending: bool,
}

fn validate_account(account: &AccountIdentity) -> bool {
    account.id != 0
        && !account.login.is_empty()
        && account.login.len() <= 39
        && !account.login.starts_with('-')
        && !account.login.ends_with('-')
        && account
            .login
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn encode(stored: &StoredConnection) -> Result<Zeroizing<Vec<u8>>, StoreError> {
    if !validate_account(&stored.account) || stored.tokens.validate_restored().is_err() {
        return Err(StoreError::Rejected);
    }
    let wire = StoredRef {
        version: CONTRACT_VERSION,
        app_id: crate::github_api::REGISTERED_APP_ID,
        client_id: crate::github_connection::REGISTERED_CLIENT_ID,
        account_id: stored.account.id,
        account_login: &stored.account.login,
        access: &stored.tokens.access,
        refresh: &stored.tokens.refresh,
        issued_at: stored.tokens.issued_at,
        access_expires_at: stored.tokens.access_expires_at,
        refresh_expires_at: stored.tokens.refresh_expires_at,
        rotation_pending: stored.rotation_pending,
    };
    let bytes = Zeroizing::new(serde_json::to_vec(&wire).map_err(|_| StoreError::Rejected)?);
    if bytes.len() > MAX_BLOB_BYTES {
        return Err(StoreError::Rejected);
    }
    Ok(bytes)
}

fn decode(bytes: &[u8]) -> Result<StoredConnection, StoreError> {
    if bytes.is_empty() || bytes.len() > MAX_BLOB_BYTES {
        return Err(StoreError::Rejected);
    }
    let wire: StoredWire = serde_json::from_slice(bytes).map_err(|_| StoreError::Rejected)?;
    if wire.version != CONTRACT_VERSION
        || wire.app_id != crate::github_api::REGISTERED_APP_ID
        || wire.client_id != crate::github_connection::REGISTERED_CLIENT_ID
    {
        return Err(StoreError::Rejected);
    }
    let stored = StoredConnection {
        account: AccountIdentity {
            id: wire.account_id,
            login: wire.account_login,
        },
        tokens: TokenSet {
            access: wire.access,
            refresh: wire.refresh,
            issued_at: wire.issued_at,
            access_expires_at: wire.access_expires_at,
            refresh_expires_at: wire.refresh_expires_at,
        },
        rotation_pending: wire.rotation_pending,
    };
    if !validate_account(&stored.account) || stored.tokens.validate_restored().is_err() {
        return Err(StoreError::Rejected);
    }
    Ok(stored)
}

#[derive(Default)]
pub(crate) struct GithubCredentialStore;

impl GithubCredentialStore {
    /// A pending rotation restored after interruption cannot be retried; reconnect.
    pub(crate) fn read(&self) -> Result<StoredConnection, StoreError> {
        let stored = decode(&native::read()?)?;
        if stored.rotation_pending {
            return Err(StoreError::RotationPending);
        }
        Ok(stored)
    }

    /// One fixed blob holds the complete token pair and binding. Verify after
    /// CredWrite; this is not a cross-process CAS or forensic deletion claim.
    pub(crate) fn save(&self, stored: &StoredConnection) -> Result<(), StoreError> {
        if stored.rotation_pending {
            return Err(StoreError::Rejected);
        }
        let bytes = encode(stored)?;
        native::write(&bytes)?;
        let reread = native::read()?;
        if reread.as_slice() != bytes.as_slice() {
            return Err(StoreError::System);
        }
        Ok(())
    }

    /// Caller must persist this state before a refresh request. Failures prevent
    /// dispatch. An interrupted/uncertain refresh never restores reusable rights.
    pub(crate) fn mark_rotation_pending(
        &self,
        stored: &StoredConnection,
    ) -> Result<RotationPermit, StoreError> {
        if stored.rotation_pending {
            return Err(StoreError::RotationPending);
        }
        let current = self.read()?;
        if encode(&current)?.as_slice() != encode(stored)?.as_slice() {
            return Err(StoreError::Rejected);
        }
        let pending = StoredRef {
            version: CONTRACT_VERSION,
            app_id: crate::github_api::REGISTERED_APP_ID,
            client_id: crate::github_connection::REGISTERED_CLIENT_ID,
            account_id: stored.account.id,
            account_login: &stored.account.login,
            access: &stored.tokens.access,
            refresh: &stored.tokens.refresh,
            issued_at: stored.tokens.issued_at,
            access_expires_at: stored.tokens.access_expires_at,
            refresh_expires_at: stored.tokens.refresh_expires_at,
            rotation_pending: true,
        };
        let bytes = Zeroizing::new(serde_json::to_vec(&pending).map_err(|_| StoreError::Rejected)?);
        if bytes.len() > MAX_BLOB_BYTES {
            return Err(StoreError::Rejected);
        }
        native::write(&bytes)?;
        if native::read()?.as_slice() != bytes.as_slice() {
            return Err(StoreError::System);
        }
        Ok(RotationPermit {
            old: current,
            pending_blob: bytes,
        })
    }

    /// Current-user custody must still equal the exact pending pair. A moved,
    /// replaced or already consumed receipt cannot publish a refreshed pair.
    pub(crate) fn publish_rotation(
        &self,
        receipt: RefreshReceipt,
        freshly_verified_account: &AccountIdentity,
    ) -> Result<(), StoreError> {
        if &receipt.account != freshly_verified_account
            || native::read()?.as_slice() != receipt.pending_blob.as_slice()
        {
            return Err(StoreError::Rejected);
        }
        let refreshed = StoredConnection {
            account: receipt.account,
            tokens: receipt.tokens,
            rotation_pending: false,
        };
        self.save(&refreshed)
    }

    /// Local forgetting only. This does not revoke GitHub authorization remotely.
    pub(crate) fn delete(&self) -> Result<bool, StoreError> {
        native::delete()
    }
}

#[cfg(not(windows))]
mod native {
    use super::*;
    pub(super) fn read() -> Result<Zeroizing<Vec<u8>>, StoreError> {
        Err(StoreError::Unavailable)
    }
    pub(super) fn write(_bytes: &[u8]) -> Result<(), StoreError> {
        Err(StoreError::Unavailable)
    }
    pub(super) fn delete() -> Result<bool, StoreError> {
        Err(StoreError::Unavailable)
    }
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::{ffi::c_void, mem, ptr};
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_NOT_FOUND};
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
        CRED_TYPE_GENERIC,
    };
    use zeroize::Zeroize;

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn exact_wide(pointer: *const u16, expected: &str) -> bool {
        if pointer.is_null() {
            return false;
        }
        for (i, unit) in expected
            .encode_utf16()
            .chain(std::iter::once(0))
            .enumerate()
        {
            // SAFETY: CredReadW metadata strings are NUL-terminated OS allocations.
            if unsafe { *pointer.add(i) } != unit {
                return false;
            }
        }
        true
    }

    struct Allocation(*mut CREDENTIALW);
    impl Drop for Allocation {
        fn drop(&mut self) {
            // SAFETY: the successful CredReadW allocation remains owned until free.
            unsafe {
                let entry = &*self.0;
                if !entry.CredentialBlob.is_null()
                    && entry.CredentialBlobSize <= MAX_BLOB_BYTES as u32
                {
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

    pub(super) fn read() -> Result<Zeroizing<Vec<u8>>, StoreError> {
        let target = wide(TARGET);
        let mut pointer = ptr::null_mut();
        // SAFETY: fixed target and live output pointer, no user target is accepted.
        if unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut pointer) } == 0 {
            return Err(if unsafe { GetLastError() } == ERROR_NOT_FOUND {
                StoreError::Missing
            } else {
                StoreError::System
            });
        }
        if pointer.is_null() {
            return Err(StoreError::System);
        }
        let allocation = Allocation(pointer);
        // SAFETY: allocation owns a valid CREDENTIALW until it leaves scope.
        let entry = unsafe { &*allocation.0 };
        if entry.Type != CRED_TYPE_GENERIC
            || entry.Persist != CRED_PERSIST_LOCAL_MACHINE
            || entry.Flags != 0
            || entry.AttributeCount != 0
            || !exact_wide(entry.TargetName, TARGET)
            || !exact_wide(entry.UserName, USERNAME)
            || entry.CredentialBlob.is_null()
            || entry.CredentialBlobSize == 0
            || entry.CredentialBlobSize > MAX_BLOB_BYTES as u32
        {
            return Err(StoreError::Rejected);
        }
        // SAFETY: OS blob length is checked before copying into a protected buffer.
        Ok(Zeroizing::new(
            unsafe {
                std::slice::from_raw_parts(entry.CredentialBlob, entry.CredentialBlobSize as usize)
            }
            .to_vec(),
        ))
    }

    pub(super) fn write(bytes: &[u8]) -> Result<(), StoreError> {
        decode(bytes)?;
        let target = wide(TARGET);
        let username = wide(USERNAME);
        let mut protected = Zeroizing::new(bytes.to_vec());
        let entry = CREDENTIALW {
            Flags: 0,
            Type: CRED_TYPE_GENERIC,
            TargetName: target.as_ptr() as *mut u16,
            Comment: ptr::null_mut(),
            LastWritten: unsafe { mem::zeroed() },
            CredentialBlobSize: protected.len() as u32,
            CredentialBlob: protected.as_mut_ptr(),
            Persist: CRED_PERSIST_LOCAL_MACHINE,
            AttributeCount: 0,
            Attributes: ptr::null_mut(),
            TargetAlias: ptr::null_mut(),
            UserName: username.as_ptr() as *mut u16,
        };
        // SAFETY: all fixed metadata and protected blob allocations stay live.
        if unsafe { CredWriteW(&entry, 0) } == 0 {
            return Err(StoreError::System);
        }
        Ok(())
    }

    pub(super) fn delete() -> Result<bool, StoreError> {
        let target = wide(TARGET);
        // SAFETY: delete only the fixed GitHub current-user Generic Credential.
        if unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) } != 0 {
            return Ok(true);
        }
        if unsafe { GetLastError() } == ERROR_NOT_FOUND {
            Ok(false)
        } else {
            Err(StoreError::System)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> StoredConnection {
        let tokens = TokenSet::parse_json(br#"{"access_token":"ghu_SYNTHETICNOTASECRET1234","refresh_token":"ghr_SYNTHETICNOTASECRET5678","expires_in":28800,"refresh_token_expires_in":15897600,"scope":"","token_type":"bearer"}"#,100).unwrap();
        StoredConnection {
            account: AccountIdentity {
                id: 7,
                login: "synthetic-owner".into(),
            },
            tokens,
            rotation_pending: false,
        }
    }

    #[test]
    fn fixed_domain_round_trip_does_not_access_os_or_restore_grants() {
        assert_eq!(TARGET, "AsterDesktop/GitHubDeviceFlow/v1");
        assert_ne!(TARGET, "AsterDesktop/MetaModelAPI/v1");
        let stored = fixture();
        let bytes = encode(&stored).unwrap();
        let parsed = decode(&bytes).unwrap();
        assert_eq!(parsed.account, stored.account);
        assert_eq!(parsed.tokens.access.as_str(), stored.tokens.access.as_str());
        assert!(!parsed.rotation_pending);
        let mut pending = fixture();
        pending.rotation_pending = true;
        assert!(decode(&encode(&pending).unwrap()).unwrap().rotation_pending);
    }

    #[test]
    fn corrupt_cross_app_cross_account_and_oversized_custody_denied_offline() {
        let value: serde_json::Value =
            serde_json::from_slice(&encode(&fixture()).unwrap()).unwrap();
        for (key, bad) in [
            ("version", serde_json::json!(99)),
            ("app_id", serde_json::json!(1)),
            ("client_id", serde_json::json!("different-public-id")),
            ("account_id", serde_json::json!(0)),
            ("account_login", serde_json::json!("../evil")),
            ("access_expires_at", serde_json::json!(101)),
            ("extra", serde_json::json!("not admitted")),
        ] {
            let mut altered = value.clone();
            altered[key] = bad;
            assert!(decode(&serde_json::to_vec(&altered).unwrap()).is_err());
        }
        assert!(decode(&vec![b'x'; MAX_BLOB_BYTES + 1]).is_err());
        assert!(decode(b"").is_err());
    }
}
