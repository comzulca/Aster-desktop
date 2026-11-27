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
const NANO_GBP_PER_PENNY: u64 = 10_000_000;
const MAX_LIMIT_PENCE: u64 = 1_000_000; // £10,000, no implicit cap on upgrade.
const DEFAULT_FIRST_REMAINING_BPS: u64 = 1_000;
const DEFAULT_SECOND_REMAINING_BPS: u64 = 500;
// UK standard VAT, not a claim about any individual invoice.
const DEFAULT_UK_VAT_BPS: u64 = 2_000;
const MAX_RESERVE_USD: u64 = 106_496_000;
const MAX_RECORDS: usize = 4096;
const MAX_PENDING: usize = 1024;
const MAX_CONFIGURATION_SECONDS: u64 = 31 * 86_400;

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FiscalIntent {
    account_id: String,
    gbp_per_usd_ppm: u64,
    margin_bps: u64,
    payment_bps: u64,
    valid_until_unix: u64,
}

impl From<FiscalIntent> for Configuration {
    fn from(intent: FiscalIntent) -> Self {
        Self {
            account_id: intent.account_id,
            gbp_per_usd_ppm: intent.gbp_per_usd_ppm,
            margin_bps: intent.margin_bps,
            tax_bps: DEFAULT_UK_VAT_BPS, // Legacy shape only. Recurring quotes use the separate VAT policy.
            payment_bps: intent.payment_bps,
            valid_until_unix: intent.valid_until_unix,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonthlyIntent {
    limit_pence: u64,
    first_remaining_bps: Option<u64>,
    second_remaining_bps: Option<u64>,
    tax_bps: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonthlyPolicy {
    limit_pence: u64,
    first_remaining_bps: u64,
    second_remaining_bps: u64,
    tax_bps: u64,
}

impl MonthlyIntent {
    pub fn validate(self) -> Result<MonthlyPolicy, &'static str> {
        let policy = MonthlyPolicy {
            limit_pence: self.limit_pence,
            first_remaining_bps: self
                .first_remaining_bps
                .unwrap_or(DEFAULT_FIRST_REMAINING_BPS),
            second_remaining_bps: self
                .second_remaining_bps
                .unwrap_or(DEFAULT_SECOND_REMAINING_BPS),
            tax_bps: self.tax_bps.unwrap_or(DEFAULT_UK_VAT_BPS),
        };
        if !policy.valid() {
            return Err("monthly_spending_invalid");
        }
        Ok(policy)
    }
}

impl MonthlyPolicy {
    fn valid(&self) -> bool {
        (1..=MAX_LIMIT_PENCE).contains(&self.limit_pence)
            && (1..=9_999).contains(&self.first_remaining_bps)
            && (1..self.first_remaining_bps).contains(&self.second_remaining_bps)
            && self.tax_bps <= 10_000
    }
    pub fn limit(&self) -> u64 {
        // The ledger validates this bounded value before it can be applied.
        self.limit_pence * NANO_GBP_PER_PENNY
    }
    fn warning_level(&self, spent: u64) -> u8 {
        let reached = |remaining_bps| {
            u128::from(spent) * 10_000
                >= u128::from(self.limit()) * u128::from(10_000 - remaining_bps)
        };
        if reached(self.second_remaining_bps) {
            2
        } else if reached(self.first_remaining_bps) {
            1
        } else {
            0
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Quote {
    configuration: Configuration,
    period: String,
    amount: u64,
    dispatched: bool,
    recurring: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ledger {
    configuration: Option<Configuration>,
    confirmed_period: Option<String>,
    monthly_policy: Option<MonthlyPolicy>,
    blocked_periods: BTreeSet<String>,
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
    SetMonthlyPolicy {
        policy: MonthlyPolicy,
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
    pub warning_level: u8,
    pub limit_reached: bool,
    pub month_resets_at: Option<u64>,
    pub monthly_policy: Option<MonthlyPolicy>,
    pub fiscal_profile_configured: bool,
    pub requires_reconciliation: bool,
    pub configuration_expires_at: Option<u64>,
    pub configuration_max_until: Option<u64>,
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
    configuration_valid_at_version(config, now, contributor_policy::record_version(now))
}

fn configuration_valid_at_version(config: &Configuration, now: u64, version: u32) -> bool {
    let Some(deadline) = contributor_policy::spending_deadline(version, now) else {
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
        && config.valid_until_unix - now <= MAX_CONFIGURATION_SECONDS
}

fn configuration_max_until(now: u64) -> Option<u64> {
    let deadline =
        contributor_policy::spending_deadline(contributor_policy::record_version(now), now)?;
    Some(deadline.min(now.checked_add(MAX_CONFIGURATION_SECONDS)?))
}

/// Actionable proposal errors; the durable ledger independently revalidates.
/// This never changes an expiry, approves a review or clears spending.
pub fn validate_configuration_proposal(
    config: &Configuration,
    now: u64,
) -> Result<(), &'static str> {
    let maximum = configuration_max_until(now).ok_or("contributor_policy_expired")?;
    if config.valid_until_unix <= now {
        return Err("budget_expiry_not_future");
    }
    if config.valid_until_unix > maximum {
        return Err("budget_expiry_out_of_range");
    }
    if !configuration_valid(config, now) {
        return Err("budget_configuration_invalid");
    }
    Ok(())
}

/// Gregorian conversion and EU BST transitions used by Europe/London since
/// 1996 (IANA tzdb europe). The year range bounds the supported calendar.
pub(crate) fn civil_from_days(days: i64) -> (i64, i64, i64) {
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
pub(crate) fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
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
fn month_resets_at(month: &str) -> Option<u64> {
    let (year, month) = month.split_once('-')?;
    let year: i64 = year.parse().ok()?;
    let month: i64 = month.parse().ok()?;
    let (year, month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    // London has switched to BST by every 1 April and switched back by
    // every 1 November in the same supported calendar range as period().
    let offset = if (4..=10).contains(&month) { 3600 } else { 0 };
    u64::try_from(days_from_civil(year, month, 1) * 86_400 - offset).ok()
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
    fn policy_limit(&self) -> u64 {
        self.monthly_policy
            .as_ref()
            .map_or(LIMIT, MonthlyPolicy::limit)
    }
    fn lock_if_spent(&mut self, month: &str, spent: u64) {
        if self.monthly_policy.is_some() && spent >= self.policy_limit() {
            self.blocked_periods.insert(month.to_owned());
        }
    }
    fn active_fiscal_configuration(
        &self,
        month: &str,
        now: u64,
        version: u32,
    ) -> Option<Configuration> {
        let mut config = self.configuration.clone()?;
        if !configuration_valid_at_version(&config, now, version) {
            return None;
        }
        if let Some(policy) = &self.monthly_policy {
            config.tax_bps = policy.tax_bps;
        } else if self.confirmed_period.as_deref() != Some(month) {
            return None;
        }
        Some(config)
    }
    fn total_reserved(&self) -> Result<u64, StoreError> {
        self.pending
            .values()
            .try_fold(0u64, |sum, q| sum.checked_add(q.amount).ok_or_else(fail))
    }
    fn apply(&mut self, event: &Event, now: u64) -> Result<(), StoreError> {
        self.apply_at_version(event, now, contributor_policy::record_version(now))
    }
    fn apply_at_version(
        &mut self,
        event: &Event,
        now: u64,
        version: u32,
    ) -> Result<(), StoreError> {
        if now < self.maximum_time {
            return Err(fail());
        }
        let month = period(now)?;
        match event {
            Event::Configure {
                configuration,
                reviewed_at,
            } => {
                if !configuration_valid_at_version(configuration, now, version)
                    || now < *reviewed_at
                    || period(*reviewed_at)? != month
                {
                    return Err(fail());
                }
                self.configuration = Some(configuration.clone());
                self.confirmed_period = Some(month);
            }
            Event::SetMonthlyPolicy {
                policy,
                reviewed_at,
            } => {
                if version != 2
                    || !policy.valid()
                    || now < *reviewed_at
                    || period(*reviewed_at)? != month
                {
                    return Err(fail());
                }
                let spent = self.spent.get(&month).copied().unwrap_or(0);
                // Reducing the limit to spent or lower closes the whole month.
                // Raising the limit later cannot undo a latch.
                self.lock_if_spent(&month, spent);
                self.monthly_policy = Some(policy.clone());
                self.lock_if_spent(&month, spent);
            }
            Event::Reserve { request } => {
                let config = self
                    .active_fiscal_configuration(&month, now, version)
                    .ok_or_else(fail)?;
                if self.bound_exceeded
                    || self.blocked_periods.contains(&month)
                    || !self.pending.is_empty()
                    || !opaque_id(request)
                    || self.seen.contains(request)
                    || self.pending.len() >= MAX_PENDING
                {
                    return Err(fail());
                }
                let amount = convert(MAX_RESERVE_USD, &config)?;
                let used = self
                    .spent
                    .get(&month)
                    .copied()
                    .unwrap_or(0)
                    .checked_add(self.total_reserved()?)
                    .and_then(|n| n.checked_add(amount))
                    .ok_or_else(fail)?;
                if used > self.policy_limit() {
                    return Err(fail());
                }
                self.seen.insert(request.clone());
                self.pending.insert(
                    request.clone(),
                    Quote {
                        configuration: config,
                        period: month,
                        amount,
                        dispatched: false,
                        recurring: self.monthly_policy.is_some(),
                    },
                );
            }
            Event::Dispatch { request } => {
                let quote = self.pending.get(request).ok_or_else(fail)?;
                if quote.dispatched
                    || !configuration_valid_at_version(&quote.configuration, now, version)
                    || quote.period != month
                    || self.bound_exceeded
                {
                    return Err(fail());
                }
                if quote.recurring {
                    let spent = self.spent.get(&month).copied().unwrap_or(0);
                    if self.blocked_periods.contains(&month)
                        || self
                            .active_fiscal_configuration(&month, now, version)
                            .as_ref()
                            != Some(&quote.configuration)
                        || spent
                            .checked_add(quote.amount)
                            .is_none_or(|total| total > self.policy_limit())
                    {
                        return Err(fail());
                    }
                } else if self.confirmed_period.as_deref() != Some(&month)
                    || self.monthly_policy.is_some()
                {
                    return Err(fail());
                }
                self.pending.get_mut(request).ok_or_else(fail)?.dispatched = true;
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
                let period = quote.period.clone();
                self.spent.insert(period.clone(), next);
                self.lock_if_spent(&period, next);
                self.pending.remove(request);
            }
            Event::Reconcile { request, charge } => {
                let quote = self.pending.get(request).ok_or_else(fail)?;
                if *charge
                    > if quote.recurring {
                        MAX_LIMIT_PENCE * NANO_GBP_PER_PENNY
                    } else {
                        LIMIT
                    }
                {
                    return Err(fail());
                }
                let next = self
                    .spent
                    .get(&quote.period)
                    .copied()
                    .unwrap_or(0)
                    .checked_add(*charge)
                    .ok_or_else(fail)?;
                self.bound_exceeded |= *charge > quote.amount || (!quote.recurring && next > LIMIT);
                let period = quote.period.clone();
                self.spent.insert(period.clone(), next);
                self.lock_if_spent(&period, next);
                self.pending.remove(request);
            }
        }
        self.maximum_time = now;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReviewedCharge {
    period: String,
    ceiling_nano_gbp: u64,
    head: String,
}
impl ReviewedCharge {
    pub fn period(&self) -> &str {
        &self.period
    }
    pub fn estimated_maximum_gbp(&self) -> String {
        let pence = self.ceiling_nano_gbp.div_ceil(NANO_GBP_PER_PENNY);
        format!("£{}.{:02}", pence / 100, pence % 100)
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
            if !(record.version == 1
                || (record.version == 2
                    && contributor_policy::record_version(record.timestamp) == 2))
                || sorted.insert(record.sequence, (record, bytes)).is_some()
            {
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
                .apply_at_version(&record.event, record.timestamp, record.version)
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
            version: contributor_policy::record_version(now),
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
    pub fn validate_monthly_review(
        &self,
        policy: &MonthlyPolicy,
        reviewed_at: u64,
        now: u64,
    ) -> Result<(), StoreError> {
        self.ensure_current()?;
        let mut candidate = self.ledger.clone();
        candidate.apply(
            &Event::SetMonthlyPolicy {
                policy: policy.clone(),
                reviewed_at,
            },
            now,
        )
    }
    pub fn configure_monthly(
        &mut self,
        policy: MonthlyPolicy,
        reviewed_at: u64,
        now: u64,
    ) -> Result<(), StoreError> {
        self.commit(
            Event::SetMonthlyPolicy {
                policy,
                reviewed_at,
            },
            now,
        )
    }
    pub fn permits_provider_metadata(&self, now: u64) -> Result<bool, StoreError> {
        self.ensure_current()?;
        if now < self.ledger.maximum_time {
            return Err(fail());
        }
        let month = period(now)?;
        Ok(!self.ledger.blocked_periods.contains(&month))
    }
    pub fn review_charge(&self, now: u64) -> Result<ReviewedCharge, StoreError> {
        let month = period(now)?;
        if self.ledger.monthly_policy.is_none()
            || now < self.ledger.maximum_time
            || !self.can_append_request()
            || self.ledger.bound_exceeded
            || !self.ledger.pending.is_empty()
            || self.ledger.blocked_periods.contains(&month)
        {
            return Err(fail());
        }
        let config = self
            .ledger
            .active_fiscal_configuration(&month, now, contributor_policy::record_version(now))
            .ok_or_else(fail)?;
        let amount = convert(MAX_RESERVE_USD, &config)?;
        let used = self
            .ledger
            .spent
            .get(&month)
            .copied()
            .unwrap_or(0)
            .checked_add(amount)
            .filter(|&sum| sum <= self.ledger.policy_limit())
            .ok_or_else(fail)?;
        debug_assert!(used >= amount);
        Ok(ReviewedCharge {
            period: month,
            ceiling_nano_gbp: amount,
            head: self.head.clone(),
        })
    }
    pub fn reserve_reviewed(
        &mut self,
        request: &str,
        now: u64,
        reviewed: &ReviewedCharge,
    ) -> Result<(), StoreError> {
        if self.review_charge(now)? != *reviewed {
            return Err(fail());
        }
        self.reserve(request, now)
    }
    fn reserve(&mut self, request: &str, now: u64) -> Result<(), StoreError> {
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
        let limit = self.ledger.policy_limit();
        let reserved = self.ledger.total_reserved().unwrap_or(limit);
        let fiscal_profile_configured = healthy
            && self
                .ledger
                .configuration
                .as_ref()
                .is_some_and(|config| configuration_valid(config, now));
        let configured = fiscal_profile_configured
            && self.ledger.monthly_policy.is_some()
            && current.is_some()
            && !self.ledger.bound_exceeded;
        let limit_reached = current
            .as_ref()
            .is_some_and(|month| self.ledger.blocked_periods.contains(month));
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
            month_resets_at: current.as_deref().and_then(month_resets_at),
            period: current,
            limit_nano_gbp: limit.to_string(),
            spent_nano_gbp: spent.to_string(),
            reserved_nano_gbp: reserved.to_string(),
            remaining_nano_gbp: if limit_reached {
                0
            } else {
                limit.saturating_sub(spent.saturating_add(reserved))
            }
            .to_string(),
            warning_level: if healthy {
                self.ledger
                    .monthly_policy
                    .as_ref()
                    .map_or(0, |policy| policy.warning_level(spent))
            } else {
                0
            },
            limit_reached,
            monthly_policy: self.ledger.monthly_policy.clone(),
            fiscal_profile_configured,
            requires_reconciliation: !healthy
                || self.ledger.bound_exceeded
                || !self.ledger.pending.is_empty(),
            configuration_expires_at: self
                .ledger
                .configuration
                .as_ref()
                .map(|c| c.valid_until_unix),
            configuration_max_until: configuration_max_until(now),
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
        !status.limit_reached
            && !status.requires_reconciliation
            && self.can_append_request()
            && status.period.as_ref().is_some_and(|month| {
                self.ledger
                    .active_fiscal_configuration(
                        month,
                        now,
                        contributor_policy::record_version(now),
                    )
                    .and_then(|config| convert(MAX_RESERVE_USD, &config).ok())
                    .and_then(|amount| {
                        self.ledger
                            .spent
                            .get(month)
                            .copied()
                            .unwrap_or(0)
                            .checked_add(amount)
                    })
                    .is_some_and(|total| total <= self.ledger.policy_limit())
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
    fn recurring(now: u64, cap_pence: u64) -> Budget {
        let mut b = Budget::restore(PrivateStore::memory(Domain::Budget)).unwrap();
        let mut fiscal = config();
        fiscal.valid_until_unix = now + 15 * 86_400;
        b.configure(fiscal, now, now).unwrap();
        b.configure_monthly(monthly(cap_pence), now, now).unwrap();
        b
    }
    fn monthly(limit_pence: u64) -> MonthlyPolicy {
        MonthlyIntent {
            limit_pence,
            first_remaining_bps: None,
            second_remaining_bps: None,
            tax_bps: None,
        }
        .validate()
        .unwrap()
    }
    #[test]
    fn monthly_proposal_applies_uk_defaults_and_rejects_unknown_or_misordered_fields() {
        let intent: MonthlyIntent = serde_json::from_str(r#"{"limitPence":12000}"#).unwrap();
        let accepted = intent.validate().unwrap();
        assert_eq!(accepted.limit(), 120_000_000_000);
        assert_eq!(
            (
                accepted.first_remaining_bps,
                accepted.second_remaining_bps,
                accepted.tax_bps
            ),
            (1000, 500, 2000)
        );
        for input in [
            r#"{"limitPence":0}"#,
            r#"{"limitPence":1000001}"#,
            r#"{"limitPence":200,"firstRemainingBps":500,"secondRemainingBps":500}"#,
            r#"{"limitPence":200,"firstRemainingBps":10000,"secondRemainingBps":0}"#,
            r#"{"limitPence":200,"taxBps":10001}"#,
        ] {
            assert!(serde_json::from_str::<MonthlyIntent>(input)
                .unwrap()
                .validate()
                .is_err());
        }
        assert!(
            serde_json::from_str::<MonthlyIntent>(r#"{"limitPence":200,"approved":true}"#).is_err()
        );
        let adjusted = serde_json::from_str::<MonthlyIntent>(
            r#"{"limitPence":200,"firstRemainingBps":1234,"secondRemainingBps":123,"taxBps":0}"#,
        )
        .unwrap()
        .validate()
        .unwrap();
        assert_eq!(
            (
                adjusted.first_remaining_bps,
                adjusted.second_remaining_bps,
                adjusted.tax_bps
            ),
            (1234, 123, 0)
        );
        assert!(serde_json::from_str::<FiscalIntent>(r#"{"accountId":"a","gbpPerUsdPpm":1000,"marginBps":100,"paymentBps":0,"validUntilUnix":1797000000,"taxBps":0}"#).is_err());
    }
    #[test]
    fn warnings_follow_settled_spend_at_exact_90_and_95_percent_not_reservations() {
        let now = contributor_policy::EXPIRES_AT;
        let mut b = recurring(now, 1000);
        for (spent, expected) in [
            (8_999_999_999, 0),
            (9_000_000_000, 1),
            (9_499_999_999, 1),
            (9_500_000_000, 2),
        ] {
            b.ledger.spent.insert(period(now).unwrap(), spent);
            assert_eq!(b.status(now).warning_level, expected);
        }
        b.ledger.spent.insert(period(now).unwrap(), 8_999_999_999);
        b.reserve(&"1".repeat(32), now).unwrap();
        assert!(b.ledger.total_reserved().unwrap() > 1);
        assert_eq!(b.status(now).warning_level, 0);
    }
    #[test]
    fn new_policy_latches_month_on_exact_settlement_survives_restart_and_resumes_at_london_boundary(
    ) {
        let now = 1_796_083_199; // 30 November 2026 23:59:59 GMT.
        let month = period(now).unwrap();
        assert_eq!(month, "2026-11");
        let mut b = recurring(now, 20);
        let amount = b
            .ledger
            .active_fiscal_configuration(&month, now, 2)
            .map(|fiscal| convert(MAX_RESERVE_USD, &fiscal).unwrap())
            .unwrap();
        let cap = b.ledger.policy_limit();
        assert!(cap > amount && cap - amount <= amount);
        // First settle the exact synthetic statement-confirmed amount that
        // leaves room for one maximum quote. The second validated usage then
        // reaches 100% exactly; both entries and the latch survive replay.
        let id = "a".repeat(32);
        b.reserve(&id, now).unwrap();
        b.mark_dispatch(&id, now).unwrap();
        b.reconcile(&id, cap - amount, now).unwrap();
        assert!(!b.status(now).limit_reached);
        let id = "c".repeat(32);
        let reviewed = b.review_charge(now).unwrap();
        b.reserve_reviewed(&id, now, &reviewed).unwrap();
        b.mark_dispatch(&id, now).unwrap();
        b.settle(
            &id,
            TokenUsage {
                prompt_tokens: 1_048_576,
                completion_tokens: 8192,
                total_tokens: 1_048_576 + 8192,
                cached_tokens: 0,
                reasoning_tokens: 0,
            },
            now,
        )
        .unwrap();
        assert!(b.status(now).limit_reached);
        assert_eq!(b.status(now).remaining_nano_gbp, "0");
        b.configure_monthly(monthly(200), now, now).unwrap();
        let mut b = Budget::restore(b.store).unwrap();
        assert!(!b.available(now));
        assert!(!b.permits_provider_metadata(now).unwrap());
        assert!(b.reserve(&"b".repeat(32), now).is_err());
        assert_eq!(b.status(now).month_resets_at, Some(now + 1));
        assert_eq!(b.status(now + 1).period.as_deref(), Some("2026-12"));
        assert!(!b.status(now + 1).limit_reached);
        assert!(b.permits_provider_metadata(now + 1).unwrap());
        assert!(b.available(now + 1));
        assert_eq!(b.ledger.spent[&month], cap);
        assert!(b.reserve(&"b".repeat(32), now + 1).is_ok());
        assert!(b.mark_dispatch(&"b".repeat(32), now + 1).is_ok());
    }
    #[test]
    fn exact_native_charge_review_is_invalidated_by_even_equal_policy_revision_or_month_boundary() {
        let now = 1_796_083_199;
        let mut b = recurring(now, 200);
        let quote = b.review_charge(now).unwrap();
        assert_eq!(quote.period(), "2026-11");
        assert_eq!(quote.estimated_maximum_gbp(), "£0.12");
        let id = "f".repeat(32);
        assert!(b.reserve_reviewed(&id, now + 1, &quote).is_err());
        assert!(b.ledger.pending.is_empty());
        b.configure_monthly(monthly(200), now, now).unwrap();
        assert!(b.reserve_reviewed(&id, now, &quote).is_err());
        assert!(b.ledger.pending.is_empty());
        let updated = b.review_charge(now).unwrap();
        assert!(b.reserve_reviewed(&id, now, &updated).is_ok());
        assert!(b.reserve_reviewed(&id, now, &updated).is_err());
    }
    #[test]
    fn old_fiscal_profile_is_not_implicit_authority_for_a_recurring_charge() {
        let now = contributor_policy::EXPIRES_AT;
        let mut b = Budget::restore(PrivateStore::memory(Domain::Budget)).unwrap();
        let mut fiscal = config();
        fiscal.valid_until_unix = now + 3600;
        b.configure(fiscal, now, now).unwrap();
        assert!(b.status(now).fiscal_profile_configured);
        assert!(!b.status(now).configured);
        assert!(b.review_charge(now).is_err());
    }
    #[test]
    fn recurring_vat_bound_in_quotes_and_change_before_dispatch_denies() {
        let now = contributor_policy::EXPIRES_AT;
        let mut b = recurring(now, 89);
        let id = "4".repeat(32);
        b.reserve(&id, now).unwrap();
        let quote = b.ledger.pending[&id].amount;
        let mut policy = monthly(89);
        policy.tax_bps = 0;
        b.configure_monthly(policy, now, now).unwrap();
        assert!(b.mark_dispatch(&id, now).is_err());
        b.release_unsent(&id, now).unwrap();
        b.reserve(&"5".repeat(32), now).unwrap();
        assert!(b.ledger.pending["55555555555555555555555555555555"].amount < quote);
        assert_eq!(
            b.ledger.pending["55555555555555555555555555555555"]
                .configuration
                .tax_bps,
            0
        );
    }
    #[test]
    fn cap_preflight_never_blinds_ledgers_or_clears_cross_month_unknown_charges() {
        let now = 1_796_083_199;
        let mut b = recurring(now, 200);
        b.configure_monthly(monthly(1), now, now).unwrap();
        assert!(!b.available(now));
        assert!(
            !b.status(now).limit_reached,
            "cannot admit a worst-case quote does not mean fully spent"
        );
        b.configure_monthly(monthly(200), now, now).unwrap();
        let id = "b".repeat(32);
        b.reserve(&id, now).unwrap();
        b.mark_dispatch(&id, now).unwrap();
        assert!(b
            .validate_monthly_review(&monthly(200), now, now + 1)
            .is_err());
        let mut b = Budget::restore(b.store).unwrap();
        assert!(b.status(now + 1).requires_reconciliation);
        assert!(!b.available(now + 1));
        assert!(b.reserve(&"c".repeat(32), now + 1).is_err());
        b.reconcile(&id, 1, now + 1).unwrap();
        assert!(b.available(now + 1));
        let next = now + 15 * 86_400;
        assert!(!b.status(next).fiscal_profile_configured);
        assert!(!b.available(next));
        assert!(b.status(next).monthly_policy.is_some());
    }
    #[test]
    fn monthly_calendar_uses_bst_and_gmt_without_extending_legacy_review() {
        for (origin, next, utc_year, utc_month, offset) in [
            ("2026-03", "2026-04", 2026, 4, 3600),
            ("2026-10", "2026-11", 2026, 11, 0),
            ("2026-12", "2027-01", 2027, 1, 0),
        ] {
            let time = month_resets_at(origin).unwrap();
            assert_eq!(
                time,
                (days_from_civil(utc_year, utc_month, 1) * 86_400 - offset) as u64
            );
            assert_eq!(period(time - 1).unwrap(), origin);
            assert_eq!(period(time).unwrap(), next);
        }
    }
    #[test]
    fn configuration_proposal_rejects_the_reported_expiry_and_accepts_valid_activation() {
        let now = contributor_policy::REVIEWED_AT + 20 * 3600 + 33 * 60;
        let mut proposed = config();
        proposed.gbp_per_usd_ppm = 743_053;
        proposed.margin_bps = 1000;
        proposed.tax_bps = 2000;
        proposed.payment_bps = 300;
        proposed.valid_until_unix = contributor_policy::EXPIRES_AT + 20 * 3600 + 33 * 60;
        assert_eq!(
            validate_configuration_proposal(&proposed, now),
            Err("budget_expiry_out_of_range")
        );
        let mut b = Budget::restore(PrivateStore::memory(Domain::Budget)).unwrap();
        assert!(b.configure(proposed.clone(), now, now).is_err());
        assert!(!b.status(now).configured);
        assert_eq!(b.status(now).spent_nano_gbp, "0");
        assert_eq!(
            b.status(now).configuration_max_until,
            Some(contributor_policy::EXPIRES_AT)
        );
        proposed.valid_until_unix = contributor_policy::EXPIRES_AT - 6 * 86_400 - 60;
        assert_eq!(validate_configuration_proposal(&proposed, now), Ok(()));
        b.configure(proposed.clone(), now, now).unwrap();
        assert!(b.available(now));
        let b = Budget::restore(b.store).unwrap();
        assert!(b.available(now));
        assert_eq!(
            b.status(now).configuration_expires_at,
            Some(proposed.valid_until_unix)
        );
    }
    #[test]
    fn configuration_proposal_enforces_exact_host_deadline_and_numeric_bounds() {
        let now = contributor_policy::REVIEWED_AT + 3600;
        let mut proposed = config();
        proposed.valid_until_unix = contributor_policy::EXPIRES_AT;
        assert_eq!(validate_configuration_proposal(&proposed, now), Ok(()));
        proposed.valid_until_unix += 1;
        assert_eq!(
            validate_configuration_proposal(&proposed, now),
            Err("budget_expiry_out_of_range")
        );
        proposed.valid_until_unix = now;
        assert_eq!(
            validate_configuration_proposal(&proposed, now),
            Err("budget_expiry_not_future")
        );
        proposed.valid_until_unix = now + 1;
        proposed.margin_bps = 0;
        assert_eq!(
            validate_configuration_proposal(&proposed, now),
            Err("budget_configuration_invalid")
        );
        proposed.margin_bps = 1000;
        proposed.gbp_per_usd_ppm = 10_000_001;
        assert_eq!(
            validate_configuration_proposal(&proposed, now),
            Err("budget_configuration_invalid")
        );
        let now = contributor_policy::EXPIRES_AT;
        proposed.gbp_per_usd_ppm = 800_000;
        proposed.valid_until_unix = now + MAX_CONFIGURATION_SECONDS;
        assert_eq!(validate_configuration_proposal(&proposed, now), Ok(()));
        let mut b = Budget::restore(PrivateStore::memory(Domain::Budget)).unwrap();
        b.configure(proposed.clone(), now, now).unwrap();
        assert!(Budget::restore(b.store).unwrap().available(now));
        assert!(!configuration_valid_at_version(&proposed, now, 1));
        proposed.valid_until_unix += 1;
        assert_eq!(
            validate_configuration_proposal(&proposed, now),
            Err("budget_expiry_out_of_range")
        );
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
        assert!(b.status(NOW).fiscal_profile_configured);
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
