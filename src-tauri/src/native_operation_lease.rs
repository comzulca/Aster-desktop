//! Shared native-operation exclusion only; this lease is never consent.
//! Both the async command and a queued/running blocking worker retain an owner.
//! The final owner releases the busy flag after return, cancellation or unwind.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Default)]
pub(crate) struct NativeOperationGate {
    busy: Arc<AtomicBool>,
}

impl NativeOperationGate {
    pub(crate) fn acquire(
        &self,
        mut is_closing: impl FnMut() -> bool,
    ) -> Result<NativeOperationLease, &'static str> {
        if is_closing() {
            return Err("window_closing");
        }
        self.busy
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| "native_dialog_busy")?;
        let lease = NativeOperationLease {
            owner: Arc::new(LeaseOwner {
                busy: Arc::clone(&self.busy),
            }),
        };
        // The local owner releases a successful CAS if closing raced acquisition.
        if is_closing() {
            return Err("window_closing");
        }
        Ok(lease)
    }

    /// Existing close admission reads the same shared flag; it cannot clear it.
    pub(crate) fn load(&self, ordering: Ordering) -> bool {
        self.busy.load(ordering)
    }
}

struct LeaseOwner {
    busy: Arc<AtomicBool>,
}

impl Drop for LeaseOwner {
    fn drop(&mut self) {
        self.busy.store(false, Ordering::Release);
    }
}

/// Not serializable, not an approval, and not a reference to the complete Host.
pub(crate) struct NativeOperationLease {
    owner: Arc<LeaseOwner>,
}

impl NativeOperationLease {
    /// Only fixed host worker plumbing shares ownership, before the worker queues.
    pub(crate) fn worker_lease(&self) -> Self {
        Self {
            owner: Arc::clone(&self.owner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn acquisition_is_exclusive_and_final_owner_releases() {
        let gate = NativeOperationGate::default();
        assert!(!gate.load(Ordering::Acquire));
        let caller = gate.acquire(|| false).unwrap();
        assert!(gate.load(Ordering::Acquire));
        assert!(matches!(gate.acquire(|| false), Err("native_dialog_busy")));
        drop(caller);
        assert!(!gate.load(Ordering::Acquire));
        assert!(gate.acquire(|| false).is_ok());
    }

    #[test]
    fn dropped_caller_cannot_release_retained_worker() {
        let gate = NativeOperationGate::default();
        let caller = gate.acquire(|| false).unwrap();
        let worker = caller.worker_lease();
        drop(caller);
        assert!(gate.load(Ordering::Acquire));
        assert!(matches!(gate.acquire(|| false), Err("native_dialog_busy")));
        drop(worker);
        assert!(!gate.load(Ordering::Acquire));
    }

    #[test]
    fn worker_return_cannot_release_callers_whole_command_scope() {
        let gate = NativeOperationGate::default();
        let caller = gate.acquire(|| false).unwrap();
        let worker = caller.worker_lease();
        drop(worker);
        assert!(gate.load(Ordering::Acquire));
        assert!(matches!(gate.acquire(|| false), Err("native_dialog_busy")));
        drop(caller);
        assert!(!gate.load(Ordering::Acquire));
    }

    #[test]
    fn queued_closure_owns_lease_even_when_it_never_starts() {
        let gate = NativeOperationGate::default();
        let caller = gate.acquire(|| false).unwrap();
        let worker = caller.worker_lease();
        let queued = move || {
            let _worker_lease = worker;
        };
        drop(caller);
        assert!(matches!(gate.acquire(|| false), Err("native_dialog_busy")));
        drop(queued);
        assert!(!gate.load(Ordering::Acquire));
    }

    #[test]
    fn worker_unwind_releases_final_owner_but_not_a_live_caller() {
        let gate = NativeOperationGate::default();
        let caller = gate.acquire(|| false).unwrap();
        let worker = caller.worker_lease();
        assert!(catch_unwind(AssertUnwindSafe(move || {
            let _worker_lease = worker;
            panic!("synthetic worker unwind");
        }))
        .is_err());
        assert!(gate.load(Ordering::Acquire));
        let worker = caller.worker_lease();
        drop(caller);
        assert!(catch_unwind(AssertUnwindSafe(move || {
            let _worker_lease = worker;
            panic!("synthetic final owner unwind");
        }))
        .is_err());
        assert!(!gate.load(Ordering::Acquire));
    }

    #[test]
    fn caller_unwind_does_not_release_an_independent_worker() {
        let gate = NativeOperationGate::default();
        let caller = gate.acquire(|| false).unwrap();
        let worker = caller.worker_lease();
        assert!(catch_unwind(AssertUnwindSafe(move || {
            let _caller_lease = caller;
            panic!("synthetic waiter unwind");
        }))
        .is_err());
        assert!(matches!(gate.acquire(|| false), Err("native_dialog_busy")));
        drop(worker);
        assert!(!gate.load(Ordering::Acquire));
    }

    #[test]
    fn initial_closing_denies_without_acquiring() {
        let gate = NativeOperationGate::default();
        let mut observations = 0;
        assert!(matches!(
            gate.acquire(|| {
                observations += 1;
                true
            }),
            Err("window_closing")
        ));
        assert_eq!(observations, 1);
        assert!(!gate.load(Ordering::Acquire));
    }

    #[test]
    fn closing_after_cas_denies_and_releases_its_owner() {
        let gate = NativeOperationGate::default();
        let mut observations = 0;
        assert!(matches!(
            gate.acquire(|| {
                observations += 1;
                observations == 2
            }),
            Err("window_closing")
        ));
        assert_eq!(observations, 2);
        assert!(!gate.load(Ordering::Acquire));
        assert!(gate.acquire(|| false).is_ok());
    }
}
