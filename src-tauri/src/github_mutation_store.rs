//! Test-build adapter for the fixed protected mutation domain only. No runtime
//! startup, IPC, status, account, native approval or network dispatch route.
//! Backend bytes become ledger evidence only after the existing strict replay.
#![cfg(test)]

use super::github_mutation_ledger::RecordBackend;
use super::Error;
use crate::private_store::{Domain, PrivateStore};

pub(super) struct ProtectedBackend {
    store: PrivateStore,
}

impl ProtectedBackend {
    pub(super) fn memory() -> Self {
        Self {
            store: PrivateStore::memory(Domain::GithubMutations),
        }
    }

    /// Only the hosted Windows synthetic fixtures call this test-build seam.
    /// Their freshly created root is never an owner application-data path.
    #[cfg(windows)]
    pub(super) fn open_synthetic_root(base: &std::path::Path) -> Result<Self, Error> {
        PrivateStore::open(base, Domain::GithubMutations)
            .map(|store| Self { store })
            .map_err(|_| Error::Ledger)
    }

    /// Pure memory fixture only; the existing store refuses native injection.
    pub(super) fn fail_next_append_for_test(&mut self, after_write: bool) {
        self.store.fail_next_append_for_test(after_write);
    }
}

impl RecordBackend for ProtectedBackend {
    fn records(&self) -> Result<Vec<(String, Vec<u8>)>, Error> {
        self.store.records().map_err(|_| Error::Ledger)
    }

    fn append(&mut self, name: &str, bytes: &[u8]) -> Result<(), Error> {
        // No name/payload rewrite, capacity refund, error repair or retry.
        // A failed append can already have become durably visible.
        self.store.append(name, bytes).map_err(|_| Error::Ledger)
    }
}
