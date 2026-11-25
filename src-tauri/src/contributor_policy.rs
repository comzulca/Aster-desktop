//! Historical Contributor windows and the versioned successor admission policy.
pub(crate) const TERMS_REVISION: &str = "2026-10-09";
pub(crate) const GEOGRAPHY_REVISION: &str = "2026-11-09";
pub(crate) const REVIEWED_ON: &str = "2026-11-25";
const LIFECYCLE_START: u64 = 1_795_564_800;
pub(crate) const REVIEWED_AT: u64 = 1_793_836_800;
pub(crate) const EXPIRES_AT: u64 = 1_796_515_200;
pub(crate) const LEGACY_REVISION: &str = "2026-10-02";
pub(crate) const LEGACY_START: u64 = 1_790_899_200;
pub(crate) const LEGACY_END: u64 = 1_793_577_600;

pub(crate) fn current(now: u64) -> bool {
    historical_current(now) || now >= LIFECYCLE_START
}

fn historical_current(now: u64) -> bool {
    (REVIEWED_AT..EXPIRES_AT).contains(&now)
}

pub(crate) fn record_version(now: u64) -> u32 {
    if now >= LIFECYCLE_START {
        2
    } else {
        1
    }
}

pub(crate) fn accepted_record_valid(version: u32, revision: &str, now: u64) -> bool {
    match (version, revision) {
        (1, LEGACY_REVISION) => (LEGACY_START..LEGACY_END).contains(&now),
        (1, TERMS_REVISION) => historical_current(now),
        (2, TERMS_REVISION) => now >= LIFECYCLE_START,
        _ => false,
    }
}

/// Replay uses the policy in force at the original event, never a retroactive
/// extension. In particular the unreviewed 2–5 November gap remains closed.
pub(crate) fn spending_deadline(version: u32, now: u64) -> Option<u64> {
    match version {
        1 if (LEGACY_START..LEGACY_END).contains(&now) => Some(LEGACY_END),
        1 if historical_current(now) => Some(EXPIRES_AT),
        2 if now >= LIFECYCLE_START => Some(u64::MAX),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn review_boundaries_and_unreviewed_gap_are_closed() {
        assert!(!current(REVIEWED_AT - 1));
        assert!(current(REVIEWED_AT));
        assert!(current(EXPIRES_AT - 1));
        assert!(current(EXPIRES_AT));
        assert!(current(EXPIRES_AT + 365 * 86_400));
        assert_eq!(spending_deadline(1, EXPIRES_AT), None);
        assert_eq!(spending_deadline(2, EXPIRES_AT), Some(u64::MAX));
        assert_eq!(EXPIRES_AT - REVIEWED_AT, 31 * 86_400);
        assert_eq!(spending_deadline(1, LEGACY_END - 1), Some(LEGACY_END));
        assert_eq!(spending_deadline(1, LEGACY_END), None);
        assert_eq!(spending_deadline(1, REVIEWED_AT - 1), None);
        assert_eq!(spending_deadline(1, REVIEWED_AT), Some(EXPIRES_AT));
        assert!(!accepted_record_valid(1, LEGACY_REVISION, REVIEWED_AT));
        assert!(!accepted_record_valid(1, "unknown", REVIEWED_AT));
    }
}
