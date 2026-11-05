//! Dated primary-source review. These are Aster review deadlines, not Meta
//! account/key/subscription expiry. Historical records keep their own window.
pub(crate) const TERMS_REVISION: &str = "2026-10-09";
pub(crate) const GEOGRAPHY_REVISION: &str = "2026-10-12";
pub(crate) const REVIEWED_ON: &str = "2026-11-05";
pub(crate) const REVIEWED_AT: u64 = 1_793_836_800;
pub(crate) const EXPIRES_AT: u64 = 1_796_515_200;
pub(crate) const EXPIRES_ISO: &str = "2026-12-06T00:00:00Z";
pub(crate) const LEGACY_REVISION: &str = "2026-10-02";
pub(crate) const LEGACY_START: u64 = 1_790_899_200;
pub(crate) const LEGACY_END: u64 = 1_793_577_600;

pub(crate) fn current(now: u64) -> bool {
    (REVIEWED_AT..EXPIRES_AT).contains(&now)
}

pub(crate) fn accepted_record_valid(revision: &str, now: u64) -> bool {
    match revision {
        LEGACY_REVISION => (LEGACY_START..LEGACY_END).contains(&now),
        TERMS_REVISION => current(now),
        _ => false,
    }
}

/// Replay uses the policy in force at the original event, never a retroactive
/// extension. In particular the unreviewed 2–5 November gap remains closed.
pub(crate) fn spending_deadline(now: u64) -> Option<u64> {
    if (LEGACY_START..LEGACY_END).contains(&now) {
        Some(LEGACY_END)
    } else if current(now) {
        Some(EXPIRES_AT)
    } else {
        None
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
        assert!(!current(EXPIRES_AT));
        assert_eq!(EXPIRES_AT - REVIEWED_AT, 31 * 86_400);
        assert_eq!(spending_deadline(LEGACY_END - 1), Some(LEGACY_END));
        assert_eq!(spending_deadline(LEGACY_END), None);
        assert_eq!(spending_deadline(REVIEWED_AT - 1), None);
        assert_eq!(spending_deadline(REVIEWED_AT), Some(EXPIRES_AT));
        assert!(!accepted_record_valid(LEGACY_REVISION, REVIEWED_AT));
        assert!(!accepted_record_valid("unknown", REVIEWED_AT));
    }
}
