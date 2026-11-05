//! Durable integer-only Contributor spending. No prompts, keys or reset API.
//! Unknown spend retains its reservation across restarts and month changes.
use crate::{
    admission::opaque_id,
    contributor_policy,
    private_store::{new_id, Domain, PrivateStore, StoreError},
    provider::TokenUsage,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub const LIMIT: u64 = 89_000_000_000;
const WARNING: u64 = 80_100_000_000;
const MAX_RESERVE_USD: u64 = 106_496_000;
const MAX_RECORDS: usize = 4096;
const MAX_PENDING: usize = 1024;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Configuration {
    pub account_id: String,
    pub gbp_per_usd_ppm: u64,
    pub margin_bps: u64,
    pub tax_bps: u64,
    pub payment_bps: u64,
    pub valid_until_unix: u64,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Quote {
    configuration: Configuration,
    period: String,
    amount: u64,
    dispatched: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ledger {
    configuration: Option<Configuration>,
    confirmed_period: Option<String>,
    spent: BTreeMap<String, u64>,
    pending: BTreeMap<String, Quote>,
    seen: BTreeSet<String>,
    maximum_time: u64,
    bound_exceeded: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Event {
    Configure {
        configuration: Configuration,
        reviewed_at: u64,
    },
    Reserve {
        request: String,
    },
    Dispatch {
        request: String,
    },
    Release {
        request: String,
    },
    ReleaseProvenUnsent {
        request: String,
    },
    Settle {
        request: String,
        usage: TokenUsage,
    },
    Reconcile {
        request: String,
        charge: u64,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: u32,
    sequence: u64,
    previous: String,
    timestamp: u64,
    event: Event,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingStatus {
    pub request_id: String,
    pub period: String,
    pub reservation_nano_gbp: String,
    pub dispatched: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetStatus {
    pub account_id: Option<String>,
    pub pending: Vec<PendingStatus>,
    pub configured: bool,
    pub period: Option<String>,
    pub limit_nano_gbp: String,
    pub spent_nano_gbp: String,
    pub reserved_nano_gbp: String,
    pub remaining_nano_gbp: String,
    pub warning: bool,
    pub requires_reconciliation: bool,
    pub configuration_expires_at: Option<u64>,
}

fn fail() -> StoreError {
    StoreError::Denied
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn ceil_ratio(value: u64, multiplier: u64, divisor: u64) -> Result<u64, StoreError> {
    let numerator = u128::from(value)
        .checked_mul(u128::from(multiplier))
        .ok_or_else(fail)?;
    u64::try_from(
        numerator
            .checked_add(u128::from(divisor - 1))
            .ok_or_else(fail)?
            / u128::from(divisor),
    )
    .map_err(|_| fail())
}
fn convert(usd: u64, config: &Configuration) -> Result<u64, StoreError> {
    let mut cost = ceil_ratio(usd, config.gbp_per_usd_ppm, 1_000_000)?;
    for basis in [config.margin_bps, config.tax_bps, config.payment_bps] {
        cost = ceil_ratio(cost, 10_000 + basis, 10_000)?;
    }
    Ok(cost)
}
fn configuration_valid(config: &Configuration, now: u64) -> bool {
    let Some(deadline) = contributor_policy::spending_deadline(now) else {
        return false;
    };
    config.account_id.len() == 32
        && opaque_id(&config.account_id)
        && (1..=10_000_000).contains(&config.gbp_per_usd_ppm)
        && (1..=10_000).contains(&config.margin_bps)
        && config.tax_bps <= 10_000
        && config.payment_bps <= 10_000
        && config.valid_until_unix > now
        && config.valid_until_unix <= deadline
        && config.valid_until_unix - now <= 31 * 86_400
}

/// Gregorian conversion and EU BST transitions used by Europe/London since
/// 1996 (IANA tzdb europe). The short dated pricing contract bounds live use.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    (y + i64::from(m <= 2), m, d)
}
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = y - i64::from(m <= 2);
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = m + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468
}
fn last_sunday(y: i64, month: i64) -> i64 {
    let day = days_from_civil(y, month, 31);
    day - (day + 4).rem_euclid(7)
}
pub fn period(now: u64) -> Result<String, StoreError> {
    let time = i64::try_from(now).map_err(|_| fail())?;
    let (year, _, _) = civil_from_days(time / 86400);
    if !(1996..=2099).contains(&year) {
        return Err(fail());
    }
    let start = last_sunday(year, 3) * 86400 + 3600;
    let end = last_sunday(year, 10) * 86400 + 3600;
    let (y, m, _) = civil_from_days(
        (time
            + if (start..end).contains(&time) {
                3600
            } else {
                0
            })
            / 86400,
    );
    Ok(format!("{y:04}-{m:02}"))
}
fn usage_cost(usage: &TokenUsage, config: &Configuration) -> Result<u64, StoreError> {
    if usage.prompt_tokens > 1_048_576
        || usage.completion_tokens > 8192
        || usage.cached_tokens > usage.prompt_tokens
        || usage.reasoning_tokens > usage.completion_tokens
        || usage.prompt_tokens.checked_add(usage.completion_tokens) != Some(usage.total_tokens)
    {
        return Err(fail());
    }
    let usd = (usage.prompt_tokens - usage.cached_tokens)
        .checked_mul(100)
        .and_then(|n| n.checked_add(usage.cached_tokens * 2))
        .and_then(|n| n.checked_add(usage.completion_tokens * 200))
        .ok_or_else(fail)?;
    // Reasoning is already included in completion_tokens, not charged twice.
    convert(usd, config)
}

impl Ledger {
    fn total_reserved(&self) -> Result<u64, StoreError> {
        self.pending
            .values()
            .try_fold(0u64, |sum, q| sum.checked_add(q.amount).ok_or_else(fail))
    }
    fn apply(&mut self, event: &Event, now: u64) -> Result<(), StoreError> {
        if now < self.maximum_time {
            return Err(fail());
        }
        let month = period(now)?;
        match event {
            Event::Configure {
                configuration,
                reviewed_at,
            } => {
                if !configuration_valid(configuration, now)
                    || now < *reviewed_at
                    || period(*reviewed_at)? != month
                {
                    return Err(fail());
                }
                self.configuration = Some(configuration.clone());
                self.confirmed_period = Some(month);
            }
            Event::Reserve { request } => {
                let config = self.configuration.as_ref().ok_or_else(fail)?;
                if !configuration_valid(config, now)
                    || self.confirmed_period.as_deref() != Some(&month)
                    || self.bound_exceeded
                    || !self.pending.is_empty()
                    || !opaque_id(request)
                    || self.seen.contains(request)
                    || self.pending.len() >= MAX_PENDING
                {
                    return Err(fail());
                }
                let amount = convert(MAX_RESERVE_USD, config)?;
                let used = self
                    .spent
                    .get(&month)
                    .copied()
                    .unwrap_or(0)
                    .checked_add(self.total_reserved()?)
                    .and_then(|n| n.checked_add(amount))
                    .ok_or_else(fail)?;
                if used > LIMIT {
                    return Err(fail());
                }
                self.seen.insert(request.clone());
                self.pending.insert(
                    request.clone(),
                    Quote {
                        configuration: config.clone(),
                        period: month,
                        amount,
                        dispatched: false,
                    },
                );
            }
            Event::Dispatch { request } => {
                let quote = self.pending.get_mut(request).ok_or_else(fail)?;
                if quote.dispatched
                    || !configuration_valid(&quote.configuration, now)
                    || quote.period != month
                    || self.confirmed_period.as_deref() != Some(&month)
                    || self.bound_exceeded
                {
                    return Err(fail());
                }
                quote.dispatched = true;
            }
            Event::Release { request } => {
                if self.pending.get(request).is_none_or(|q| q.dispatched) {
                    return Err(fail());
                }
                self.pending.remove(request);
            }
            Event::ReleaseProvenUnsent { request } => {
                // Only a host transport preflight proving zero send reaches
                // this transition. No renderer or failed-network refund API.
                if self.pending.get(request).is_none_or(|q| !q.dispatched) {
                    return Err(fail());
                }
                self.pending.remove(request);
            }
            Event::Settle { request, usage } => {
                let quote = self.pending.get(request).ok_or_else(fail)?;
                if !quote.dispatched {
                    return Err(fail());
                }
                let charge = usage_cost(usage, &quote.configuration)?;
                if charge > quote.amount {
                    return Err(fail());
                }
                let next = self
                    .spent
                    .get(&quote.period)
                    .copied()
                    .unwrap_or(0)
                    .checked_add(charge)
                    .ok_or_else(fail)?;
                self.spent.insert(quote.period.clone(), next);
                self.pending.remove(request);
            }
            Event::Reconcile { request, charge } => {
                let quote = self.pending.get(request).ok_or_else(fail)?;
                if *charge > LIMIT {
                    return Err(fail());
                }
                let next = self
                    .spent
                    .get(&quote.period)
                    .copied()
                    .unwrap_or(0)
                    .checked_add(*charge)
                    .ok_or_else(fail)?;
                self.bound_exceeded |= *charge > quote.amount || next > LIMIT;
                self.spent.insert(quote.period.clone(), next);
                self.pending.remove(request);
            }
        }
        self.maximum_time = now;
        Ok(())
    }
}

pub struct Budget {
    store: PrivateStore,
    ledger: Ledger,
    sequence: u64,
    head: String,
    fault: bool,
}
impl Budget {
    pub fn open(base: &Path) -> Result<Self, StoreError> {
        Self::restore(PrivateStore::open(base, Domain::Budget)?)
    }
    fn replay(store: &PrivateStore) -> Result<(Ledger, u64, String), StoreError> {
        let records = store.records()?;
        if records.len() > MAX_RECORDS {
            return Err(StoreError::Capacity);
        }
        let mut sorted = BTreeMap::new();
        for (_, bytes) in records {
            let record: Record = serde_json::from_slice(&bytes).map_err(|_| StoreError::Corrupt)?;
            if record.version != 1 || sorted.insert(record.sequence, (record, bytes)).is_some() {
                return Err(StoreError::Corrupt);
            }
        }
        let mut ledger = Ledger::default();
        let mut sequence = 0;
        let mut head = String::new();
        for (_, (record, bytes)) in sorted {
            if record.sequence != sequence + 1 || record.previous != head {
                return Err(StoreError::Corrupt);
            }
            ledger
                .apply(&record.event, record.timestamp)
                .map_err(|_| StoreError::Corrupt)?;
            sequence += 1;
            head = hash(&bytes);
        }
        Ok((ledger, sequence, head))
    }
    fn restore(store: PrivateStore) -> Result<Self, StoreError> {
        let (ledger, sequence, head) = Self::replay(&store)?;
        Ok(Self {
            store,
            ledger,
            sequence,
            head,
            fault: false,
        })
    }
    fn ensure_current(&self) -> Result<(), StoreError> {
        if self.fault {
            return Err(StoreError::Corrupt);
        }
        let (_, sequence, head) = Self::replay(&self.store)?;
        if sequence != self.sequence || head != self.head {
            return Err(StoreError::Corrupt);
        }
        Ok(())
    }
    fn commit(&mut self, event: Event, now: u64) -> Result<(), StoreError> {
        self.ensure_current()?;
        if self.sequence as usize >= MAX_RECORDS {
            return Err(StoreError::Capacity);
        }
        let mut candidate = self.ledger.clone();
        candidate.apply(&event, now)?;
        let record = Record {
            version: 1,
            sequence: self.sequence + 1,
            previous: self.head.clone(),
            timestamp: now,
            event,
        };
        let bytes = serde_json::to_vec(&record).map_err(|_| StoreError::Corrupt)?;
        if let Err(error) = self.store.append(&new_id()?, &bytes) {
            self.fault = true;
            return Err(error);
        }
        self.ledger = candidate;
        self.sequence += 1;
        self.head = hash(&bytes);
        Ok(())
    }
    pub fn validate_configuration_review(
        &self,
        config: &Configuration,
        reviewed_at: u64,
        now: u64,
    ) -> Result<(), StoreError> {
        self.ensure_current()?;
        let mut candidate = self.ledger.clone();
        candidate.apply(
            &Event::Configure {
                configuration: config.clone(),
                reviewed_at,
            },
            now,
        )
    }
    pub fn configure(
        &mut self,
        config: Configuration,
        reviewed_at: u64,
        now: u64,
    ) -> Result<(), StoreError> {
        self.commit(
            Event::Configure {
                configuration: config,
                reviewed_at,
            },
            now,
        )
    }
    pub fn reserve(&mut self, request: &str, now: u64) -> Result<(), StoreError> {
        self.commit(
            Event::Reserve {
                request: request.into(),
            },
            now,
        )
    }
    pub fn mark_dispatch(&mut self, request: &str, now: u64) -> Result<(), StoreError> {
        self.commit(
            Event::Dispatch {
                request: request.into(),
            },
            now,
        )
    }
    pub fn release_unsent(&mut self, request: &str, now: u64) -> Result<(), StoreError> {
        self.commit(
            Event::Release {
                request: request.into(),
            },
            now,
        )
    }
    pub fn release_proven_unsent(&mut self, request: &str, now: u64) -> Result<(), StoreError> {
        self.commit(
            Event::ReleaseProvenUnsent {
                request: request.into(),
            },
            now,
        )
    }
    pub fn settle(&mut self, request: &str, usage: TokenUsage, now: u64) -> Result<(), StoreError> {
        self.commit(
            Event::Settle {
                request: request.into(),
                usage,
            },
            now,
        )
    }
    pub fn reconcile(&mut self, request: &str, charge: u64, now: u64) -> Result<(), StoreError> {
        self.commit(
            Event::Reconcile {
                request: request.into(),
                charge,
            },
            now,
        )
    }
    pub fn status(&self, now: u64) -> BudgetStatus {
        let current = period(now).ok();
        let healthy = self.ensure_current().is_ok() && now >= self.ledger.maximum_time;
        let spent = current
            .as_ref()
            .and_then(|p| self.ledger.spent.get(p))
            .copied()
            .unwrap_or(0);
        let reserved = self.ledger.total_reserved().unwrap_or(LIMIT);
        let configured = healthy
            && !self.ledger.bound_exceeded
            && self
                .ledger
                .configuration
                .as_ref()
                .is_some_and(|c| configuration_valid(c, now))
            && self.ledger.confirmed_period == current;
        BudgetStatus {
            account_id: self
                .ledger
                .configuration
                .as_ref()
                .map(|c| c.account_id.clone()),
            pending: self
                .ledger
                .pending
                .iter()
                .map(|(id, q)| PendingStatus {
                    request_id: id.clone(),
                    period: q.period.clone(),
                    reservation_nano_gbp: q.amount.to_string(),
                    dispatched: q.dispatched,
                })
                .collect(),
            configured,
            period: current,
            limit_nano_gbp: LIMIT.to_string(),
            spent_nano_gbp: spent.to_string(),
            reserved_nano_gbp: reserved.to_string(),
            remaining_nano_gbp: LIMIT
                .saturating_sub(spent.saturating_add(reserved))
                .to_string(),
            warning: spent.saturating_add(reserved) >= WARNING,
            requires_reconciliation: !healthy
                || self.ledger.bound_exceeded
                || !self.ledger.pending.is_empty(),
            configuration_expires_at: self
                .ledger
                .configuration
                .as_ref()
                .map(|c| c.valid_until_unix),
        }
    }
    pub fn review_context(&self, request: Option<&str>) -> Result<serde_json::Value, StoreError> {
        self.ensure_current()?;
        Ok(
            serde_json::json!({"confirmedPeriod":self.ledger.confirmed_period,"clockHighWaterUnix":self.ledger.maximum_time,
            "originalQuote":request.and_then(|id|self.ledger.pending.get(id)),"boundExceeded":self.ledger.bound_exceeded}),
        )
    }
    pub fn available(&self, now: u64) -> bool {
        let status = self.status(now);
        status.configured
            && !status.requires_reconciliation
            && self.can_append_request()
            && self.ledger.configuration.as_ref().is_some_and(|config| {
                let spent = status
                    .period
                    .as_ref()
                    .and_then(|month| self.ledger.spent.get(month))
                    .copied()
                    .unwrap_or(0);
                convert(MAX_RESERVE_USD, config)
                    .ok()
                    .and_then(|amount| spent.checked_add(amount))
                    .is_some_and(|total| total <= LIMIT)
            })
    }
    pub fn can_append_request(&self) -> bool {
        self.ensure_current().is_ok()
            && self.sequence as usize + 4 <= MAX_RECORDS
            && self
                .store
                .has_capacity(Domain::Budget, 8192, 4)
                .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CONTRACT_END: u64 = contributor_policy::LEGACY_END;
    const NOW: u64 = 1_790_978_400;
    fn config() -> Configuration {
        Configuration {
            account_id: "a".repeat(32),
            gbp_per_usd_ppm: 800_000,
            margin_bps: 1000,
            tax_bps: 2000,
            payment_bps: 300,
            valid_until_unix: CONTRACT_END,
        }
    }
    fn budget() -> Budget {
        let mut b = Budget::restore(PrivateStore::memory(Domain::Budget)).unwrap();
        b.configure(config(), NOW, NOW).unwrap();
        b
    }
    fn usage() -> TokenUsage {
        TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 20,
            total_tokens: 120,
            cached_tokens: 30,
            reasoning_tokens: 10,
        }
    }
    #[test]
    fn integer_cost_cache_and_reasoning_not_double_counted() {
        assert_eq!(usage_cost(&usage(), &config()).unwrap(), 12031);
        assert_eq!(ceil_ratio(1, 1, 3).unwrap(), 1);
        let mut bad = usage();
        bad.total_tokens += 1;
        assert!(usage_cost(&bad, &config()).is_err());
    }
    #[test]
    fn policy_renewal_preserves_old_quotes_spend_and_unknown_reservations() {
        let mut b = budget();
        let paid = "7".repeat(32);
        b.reserve(&paid, NOW).unwrap();
        b.mark_dispatch(&paid, NOW).unwrap();
        b.settle(&paid, usage(), NOW).unwrap();
        let spent = b.ledger.spent.clone();
        let pending = "8".repeat(32);
        b.reserve(&pending, NOW).unwrap();
        b.mark_dispatch(&pending, NOW).unwrap();
        let original_quote = b.ledger.pending[&pending].clone();
        let now = contributor_policy::REVIEWED_AT;
        let mut b = Budget::restore(b.store).unwrap();
        assert!(!b.available(now));
        assert_eq!(b.ledger.spent, spent);
        let mut renewed = config();
        renewed.valid_until_unix = contributor_policy::EXPIRES_AT;
        b.configure(renewed.clone(), now, now).unwrap();
        assert!(!b.available(now));
        assert_eq!(
            b.ledger.pending[&pending].configuration,
            original_quote.configuration
        );
        assert_eq!(b.ledger.pending[&pending].amount, original_quote.amount);
        b.reconcile(&pending, 1, now).unwrap();
        let b = Budget::restore(b.store).unwrap();
        assert!(b.available(now));
        assert_eq!(b.ledger.spent["2026-10"], spent["2026-10"] + 1);
        assert!(!configuration_valid(
            &renewed,
            contributor_policy::EXPIRES_AT
        ));
        assert!(!configuration_valid(&config(), now));
        assert!(!configuration_valid(
            &renewed,
            contributor_policy::LEGACY_END
        ));
    }
    #[test]
    fn london_calendar_and_bst_boundaries() {
        assert_eq!(period(1_793_491_199).unwrap(), "2026-10");
        assert_eq!(period(1_793_491_200).unwrap(), "2026-11");
        assert_eq!(period(1_774_745_999).unwrap(), "2026-03");
        assert_eq!(period(1_774_746_000).unwrap(), "2026-03");
    }
    #[test]
    fn missing_usage_restart_cancel_and_duplicate_never_free_reservation() {
        let mut b = budget();
        let id = "b".repeat(32);
        b.reserve(&id, NOW).unwrap();
        b.mark_dispatch(&id, NOW).unwrap();
        assert!(b.release_unsent(&id, NOW).is_err());
        let mut restarted = Budget::restore(b.store).unwrap();
        assert_eq!(restarted.ledger.pending.len(), 1);
        assert!(restarted.status(NOW).requires_reconciliation);
        assert!(!restarted.available(NOW));
        assert!(restarted.reserve(&"9".repeat(32), NOW).is_err());
        assert!(restarted.reserve(&id, NOW).is_err());
        restarted.settle(&id, usage(), NOW).unwrap();
        assert!(restarted.settle(&id, usage(), NOW).is_err());
        assert!(restarted.ledger.pending.is_empty());
    }
    #[test]
    fn known_unsent_release_is_durable_but_replay_still_denied() {
        let mut b = budget();
        let id = "c".repeat(32);
        b.reserve(&id, NOW).unwrap();
        b.release_unsent(&id, NOW).unwrap();
        let mut b = Budget::restore(b.store).unwrap();
        assert!(b.ledger.pending.is_empty());
        assert!(b.reserve(&id, NOW).is_err());
    }
    #[test]
    fn account_alias_does_not_reset_spend_and_new_period_carries_unknown() {
        let mut b = budget();
        let id = "d".repeat(32);
        b.reserve(&id, NOW).unwrap();
        b.mark_dispatch(&id, NOW).unwrap();
        let amount = b.ledger.total_reserved().unwrap();
        let mut c = config();
        c.account_id = "e".repeat(32);
        b.configure(c, 1_793_491_200, 1_793_491_200).unwrap();
        assert_eq!(b.ledger.total_reserved().unwrap(), amount);
        assert_eq!(b.status(1_793_491_200).period.as_deref(), Some("2026-11"));
    }
    #[test]
    fn backwards_clock_and_expired_configuration_deny() {
        let mut b = budget();
        assert!(b.reserve(&"e".repeat(32), NOW - 1).is_err());
        assert!(!b.available(CONTRACT_END));
        assert!(!b.available(1_793_491_200));
        assert!(b.configure(config(), CONTRACT_END, CONTRACT_END).is_err());
    }
    #[test]
    fn configuration_review_cannot_activate_an_unreviewed_month() {
        let mut b = budget();
        let november = 1_793_491_200;
        let sequence = b.sequence;
        assert!(b
            .validate_configuration_review(&config(), november - 1, november)
            .is_err());
        assert!(b.configure(config(), november - 1, november).is_err());
        assert_eq!(b.sequence, sequence);
        assert_eq!(b.ledger.confirmed_period.as_deref(), Some("2026-10"));
        assert!(b.configure(config(), NOW + 1, NOW).is_err());
        b.configure(config(), november, november).unwrap();
        assert_eq!(
            Budget::restore(b.store)
                .unwrap()
                .ledger
                .confirmed_period
                .as_deref(),
            Some("2026-11")
        );
    }
    #[test]
    fn expiry_during_preparation_denies_dispatch_but_allows_unsent_cleanup() {
        let mut b = budget();
        let mut short = config();
        short.valid_until_unix = NOW + 1;
        b.configure(short, NOW, NOW).unwrap();
        let id = "6".repeat(32);
        b.reserve(&id, NOW).unwrap();
        assert!(b.mark_dispatch(&id, NOW + 1).is_err());
        assert!(!b.ledger.pending[&id].dispatched);
        b.release_unsent(&id, NOW + 1).unwrap();
        assert!(Budget::restore(b.store).unwrap().ledger.pending.is_empty());
    }
    #[test]
    fn unreviewed_period_and_contract_end_deny_dispatch() {
        for deadline in [1_793_491_200, CONTRACT_END] {
            let mut b = budget();
            let id = "5".repeat(32);
            b.reserve(&id, NOW).unwrap();
            assert!(b.mark_dispatch(&id, deadline).is_err());
            assert!(!b.ledger.pending[&id].dispatched);
            b.release_unsent(&id, deadline).unwrap();
        }
    }
    #[test]
    fn expired_quote_still_settles_or_reconciles_a_dispatched_request() {
        for reconcile in [false, true] {
            let mut b = budget();
            let mut short = config();
            short.valid_until_unix = NOW + 1;
            b.configure(short, NOW, NOW).unwrap();
            let id = "4".repeat(32);
            b.reserve(&id, NOW).unwrap();
            b.mark_dispatch(&id, NOW).unwrap();
            if reconcile {
                b.reconcile(&id, 1, CONTRACT_END).unwrap();
            } else {
                b.settle(&id, usage(), CONTRACT_END).unwrap();
            }
            assert!(Budget::restore(b.store).unwrap().ledger.pending.is_empty());
        }
    }
    #[test]
    fn statement_reconciliation_records_charge_above_quote_then_blocks() {
        let mut b = budget();
        let id = "f".repeat(32);
        b.reserve(&id, NOW).unwrap();
        b.mark_dispatch(&id, NOW).unwrap();
        let amount = b.ledger.pending[&id].amount;
        b.reconcile(&id, amount + 1, NOW).unwrap();
        assert!(!b.available(NOW));
        assert!(b.reserve(&"1".repeat(32), NOW).is_err());
    }
    #[test]
    fn exact_limit_allowed_and_one_nanolibra_above_denied() {
        let mut ledger = Ledger::default();
        ledger
            .apply(
                &Event::Configure {
                    configuration: config(),
                    reviewed_at: NOW,
                },
                NOW,
            )
            .unwrap();
        let amount = convert(MAX_RESERVE_USD, &config()).unwrap();
        ledger.spent.insert("2026-10".into(), LIMIT - amount);
        assert!(ledger
            .apply(
                &Event::Reserve {
                    request: "2".repeat(32)
                },
                NOW
            )
            .is_ok());
        let mut ledger = Ledger::default();
        ledger
            .apply(
                &Event::Configure {
                    configuration: config(),
                    reviewed_at: NOW,
                },
                NOW,
            )
            .unwrap();
        ledger.spent.insert("2026-10".into(), LIMIT - amount + 1);
        assert!(ledger
            .apply(
                &Event::Reserve {
                    request: "3".repeat(32)
                },
                NOW
            )
            .is_err());
    }
    #[test]
    fn availability_denies_insufficient_headroom_without_a_balance_reset() {
        let mut b = budget();
        let amount = convert(MAX_RESERVE_USD, &config()).unwrap();
        b.ledger.spent.insert("2026-10".into(), LIMIT - amount);
        assert!(b.available(NOW));
        b.ledger.spent.insert("2026-10".into(), LIMIT - amount + 1);
        assert!(b.status(NOW).configured);
        assert!(!b.available(NOW));
        assert_eq!(b.ledger.spent["2026-10"], LIMIT - amount + 1);
    }
    #[test]
    fn forged_sequence_and_missing_head_fail_closed() {
        let mut b = budget();
        let bogus = serde_json::to_vec(&Record {
            version: 1,
            sequence: 999,
            previous: "".into(),
            timestamp: NOW,
            event: Event::Configure {
                configuration: config(),
                reviewed_at: NOW,
            },
        })
        .unwrap();
        b.store.append(&new_id().unwrap(), &bogus).unwrap();
        assert!(b.ensure_current().is_err());
        assert!(!b.available(NOW));
    }
    #[test]
    fn preflight_proof_releases_possible_dispatch_without_replay() {
        let mut b = budget();
        let id = "7".repeat(32);
        b.reserve(&id, NOW).unwrap();
        b.mark_dispatch(&id, NOW).unwrap();
        b.release_proven_unsent(&id, NOW).unwrap();
        assert!(b.available(NOW));
        assert!(b.reserve(&id, NOW).is_err());
    }
    #[test]
    fn failed_append_before_or_after_write_latches_until_replay() {
        for after in [false, true] {
            let mut b = budget();
            b.store.fail_next_append_for_test(after);
            let id = "8".repeat(32);
            assert!(b.reserve(&id, NOW).is_err());
            assert!(!b.available(NOW));
            let restored = Budget::restore(b.store).unwrap();
            assert_eq!(restored.ledger.pending.contains_key(&id), after);
            assert_eq!(restored.available(NOW), !after);
        }
    }
    #[test]
    fn partial_financial_record_never_resets_on_restart() {
        let mut b = budget();
        b.store
            .append(&new_id().unwrap(), b"{\"version\":1")
            .unwrap();
        assert!(Budget::restore(b.store).is_err());
    }
}
