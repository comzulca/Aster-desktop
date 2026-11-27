//! Fixed, public release discovery. No installer is fetched or opened.
use crate::{
    budget::{civil_from_days, days_from_civil},
    private_store::{new_id, Domain, PrivateStore, StoreError},
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

const RELEASES: &str = "https://api.github.com/repos/comzulca/Aster-desktop/releases";
const GRACE: u64 = 14 * 86_400;
const REFRESH: Duration = Duration::from_secs(6 * 3_600);
const PAGE_SIZE: usize = 100;
const MAX_PAGES: usize = 4;
const MAX_BODY: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
struct Version(u32, u32, u32);

impl Version {
    fn parse(s: &str) -> Option<Self> {
        fn component(s: &str) -> Option<u32> {
            if s.is_empty()
                || (s.len() > 1 && s.starts_with('0'))
                || !s.bytes().all(|b| b.is_ascii_digit())
            {
                return None;
            }
            s.parse().ok()
        }
        let mut fields = s.split('.');
        let version = Self(
            component(fields.next()?)?,
            component(fields.next()?)?,
            component(fields.next()?)?,
        );
        fields.next().is_none().then_some(version)
    }
}

fn timestamp(s: &str) -> Option<u64> {
    let b = s.as_bytes();
    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return None;
    }
    let number = |a, z| {
        let digits: &[u8] = &b[a..z];
        digits.iter().try_fold(0_i64, |n, &d| {
            d.is_ascii_digit().then(|| n * 10 + i64::from(d - b'0'))
        })
    };
    let (y, m, d) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
    let (h, min, sec) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
    if !(2026..=2099).contains(&y)
        || !(1..=12).contains(&m)
        || !(1..=31).contains(&d)
        || h > 23
        || min > 59
        || sec > 59
    {
        return None;
    }
    let days = days_from_civil(y, m, d);
    if civil_from_days(days) != (y, m, d) {
        return None;
    }
    u64::try_from(days * 86_400 + h * 3_600 + min * 60 + sec).ok()
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    published_at: Option<String>,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    state: String,
    size: u64,
    created_at: String,
}

impl Release {
    fn available_at(&self, installed: Version, now: u64) -> Option<u64> {
        if self.draft || self.prerelease {
            return None;
        }
        let tag = self.tag_name.strip_prefix('v').unwrap_or(&self.tag_name);
        if Version::parse(tag)? <= installed {
            return None;
        }
        let published_at = timestamp(self.published_at.as_deref()?)?;
        // GitHub can replace spaces with dots in the uploaded asset name.
        // Match only the two exact spellings, never a display label or URL.
        let name = format!("Aster Desktop_{tag}_x64-setup.exe");
        let github_name = format!("Aster.Desktop_{tag}_x64-setup.exe");
        self.assets
            .iter()
            .filter(|asset| {
                asset.state == "uploaded"
                    && asset.size > 0
                    && (asset.name == name || asset.name == github_name)
            })
            .filter_map(|asset| timestamp(&asset.created_at))
            .map(|uploaded| published_at.max(uploaded))
            .filter(|available| *available <= now)
            .min()
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    installed: Version,
    available_at: u64,
}

#[derive(Default)]
struct State {
    store: Option<PrivateStore>,
    evidence: Option<Evidence>,
}

#[derive(Clone, Default)]
pub(crate) struct Lifecycle(Arc<Mutex<State>>);

impl Lifecycle {
    pub(crate) fn open(base: &Path) -> Result<Self, StoreError> {
        Self::restore(PrivateStore::open(base, Domain::ReleaseLifecycle)?)
    }

    fn restore(store: PrivateStore) -> Result<Self, StoreError> {
        let installed = installed()?;
        let mut evidence: Option<Evidence> = None;
        for (_, payload) in store.records()? {
            let record: Evidence =
                serde_json::from_slice(&payload).map_err(|_| StoreError::Corrupt)?;
            if record.available_at < 1_767_225_600 || record.available_at >= 4_102_444_800 {
                return Err(StoreError::Corrupt);
            }
            if record.installed == installed
                && evidence.is_none_or(|prior| record.available_at < prior.available_at)
            {
                evidence = Some(record);
            }
        }
        Ok(Self(Arc::new(Mutex::new(State {
            store: Some(store),
            evidence,
        }))))
    }

    fn observe(&self, evidence: Evidence) {
        let Ok(mut state) = self.0.lock() else { return };
        if state
            .evidence
            .is_some_and(|previous| previous.available_at <= evidence.available_at)
        {
            return;
        }
        // Cache unavailability must not invent a deadline or discard live evidence.
        // A failed append still enforces this verified observation in this process.
        if let Some(store) = &mut state.store {
            let _ = (|| {
                let payload = serde_json::to_vec(&evidence).map_err(|_| StoreError::Corrupt)?;
                store.append(&new_id()?, &payload)
            })();
        }
        state.evidence = Some(evidence);
    }

    pub(crate) fn deadline(&self) -> Option<u64> {
        self.0
            .lock()
            .ok()?
            .evidence?
            .available_at
            .checked_add(GRACE)
    }

    pub(crate) fn require_current(&self, now: u64) -> Result<(), &'static str> {
        if self.deadline().is_some_and(|deadline| now >= deadline) {
            Err("aster_version_expired")
        } else {
            Ok(())
        }
    }

    pub(crate) async fn watch(self) {
        let (Ok(installed), Ok(client)) = (installed(), client()) else {
            return;
        };
        loop {
            if let Ok(Some(available_at)) = fetch(&client, installed).await {
                self.observe(Evidence {
                    installed,
                    available_at,
                });
            }
            tokio::time::sleep(REFRESH).await;
        }
    }
}

fn installed() -> Result<Version, StoreError> {
    // The engineering revision uses numeric build metadata for the NSIS
    // four-part Windows version. Published release tags remain strict triples.
    let version = env!("CARGO_PKG_VERSION");
    let release_base = if version == "0.2.5+1" {
        "0.2.5"
    } else {
        version
    };
    Version::parse(release_base).ok_or(StoreError::Corrupt)
}

fn client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .https_only(true)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .timeout(Duration::from_secs(8))
        .user_agent("Aster-Desktop-release-check")
        .build()
}

async fn fetch(client: &reqwest::Client, installed: Version) -> Result<Option<u64>, ()> {
    let mut earliest = None;
    for page in 1..=MAX_PAGES {
        let response = client
            .get(format!("{RELEASES}?per_page={PAGE_SIZE}&page={page}"))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await
            .map_err(|_| ())?;
        if response.status() != reqwest::StatusCode::OK
            || response
                .content_length()
                .is_some_and(|n| n > MAX_BODY as u64)
        {
            return Err(());
        }
        let mut bytes = response.bytes_stream();
        let mut body = Vec::new();
        while let Some(chunk) = bytes.next().await {
            let chunk = chunk.map_err(|_| ())?;
            if chunk.len() > MAX_BODY - body.len() {
                return Err(());
            }
            body.extend_from_slice(&chunk);
        }
        let releases: Vec<Release> = serde_json::from_slice(&body).map_err(|_| ())?;
        if releases.len() > PAGE_SIZE {
            return Err(());
        }
        let now = crate::now().map_err(|_| ())?;
        if let Some(candidate) = releases
            .iter()
            .filter_map(|r| r.available_at(installed, now))
            .min()
        {
            earliest = Some(earliest.map_or(candidate, |prior: u64| prior.min(candidate)));
        }
        if releases.len() < PAGE_SIZE {
            break;
        }
    }
    Ok(earliest)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PUBLISHED: u64 = 1_795_564_800;
    const INSTALLED: Version = Version(0, 2, 5);

    fn windows_release() -> Release {
        serde_json::from_value(serde_json::json!({
            "tag_name": "v0.2.6", "draft": false, "prerelease": false,
            "published_at": "2026-11-25T00:00:00Z",
            "assets": [{ "name": "Aster Desktop_0.2.6_x64-setup.exe", "state": "uploaded", "size": 1,
                "created_at": "2026-11-24T00:00:00Z" }]
        })).unwrap()
    }

    #[test]
    fn only_stable_newer_published_windows_installer_qualifies() {
        let mut release = windows_release();
        assert_eq!(release.available_at(INSTALLED, PUBLISHED), Some(PUBLISHED));
        assert_eq!(release.available_at(Version(0, 2, 6), PUBLISHED), None);
        assert_eq!(release.available_at(Version(0, 2, 10), PUBLISHED), None);
        assert_eq!(release.available_at(INSTALLED, PUBLISHED - 1), None);
        release.draft = true;
        assert_eq!(release.available_at(INSTALLED, PUBLISHED), None);
        release.draft = false;
        release.prerelease = true;
        assert_eq!(release.available_at(INSTALLED, PUBLISHED), None);
        release.prerelease = false;
        release.assets[0].size = 0;
        assert_eq!(release.available_at(INSTALLED, PUBLISHED), None);
        release.assets[0].size = 1;
        release.assets[0].name = "source.zip".into();
        assert_eq!(release.available_at(INSTALLED, PUBLISHED), None);
        assert!(serde_json::from_str::<Vec<Release>>(
            r#"[{"head_sha":"a","conclusion":"success"}]"#
        )
        .is_err());
        assert!(serde_json::from_str::<Vec<Release>>(r#"[{"number":90,"merged":true}]"#).is_err());
    }

    #[test]
    fn github_sanitized_windows_asset_name_preserves_release_gates() {
        let mut release = windows_release();
        let asset = &mut release.assets[0];
        asset.name = "Aster.Desktop_0.2.6_x64-setup.exe".into();
        asset.created_at = "2026-11-27T00:00:00Z".into();
        let available = PUBLISHED + 2 * 86_400;
        assert_eq!(release.available_at(INSTALLED, available), Some(available));
        assert_eq!(release.available_at(INSTALLED, available - 1), None);
        release.assets[0].state = "starter".into();
        assert_eq!(release.available_at(INSTALLED, available), None);
        release.assets[0].state = "uploaded".into();
        for name in [
            "Aster_Desktop_0.2.6_x64-setup.exe",
            "Aster.Desktop_0.2.7_x64-setup.exe",
            "Aster.Desktop_0.2.6_arm64-setup.exe",
            "Aster.Desktop_0.2.6_x64-setup.exe.zip",
        ] {
            release.assets[0].name = name.into();
            assert_eq!(release.available_at(INSTALLED, available), None);
        }
    }

    #[test]
    fn late_installer_starts_at_availability_not_at_empty_release() {
        let mut release = windows_release();
        release.assets[0].created_at = "2026-11-27T00:00:00Z".into();
        assert_eq!(release.available_at(INSTALLED, PUBLISHED + 86_400), None);
        assert_eq!(
            release.available_at(INSTALLED, PUBLISHED + 2 * 86_400),
            Some(PUBLISHED + 2 * 86_400)
        );
    }

    #[test]
    fn preserves_first_deadline_through_restart_and_outage() {
        let lifecycle = Lifecycle::restore(PrivateStore::memory(Domain::ReleaseLifecycle)).unwrap();
        assert_eq!(lifecycle.require_current(PUBLISHED + 100 * GRACE), Ok(()));
        let installed = installed().unwrap();
        for available_at in [PUBLISHED + 10, PUBLISHED, PUBLISHED + 1] {
            lifecycle.observe(Evidence {
                installed,
                available_at,
            });
        }
        let store = lifecycle.0.lock().unwrap().store.take().unwrap();
        let restored = Lifecycle::restore(store).unwrap();
        assert_eq!(restored.require_current(PUBLISHED + GRACE - 1), Ok(()));
        assert_eq!(
            restored.require_current(PUBLISHED + GRACE),
            Err("aster_version_expired")
        );
    }

    #[test]
    fn evidence_for_another_installed_version_is_not_inherited() {
        let lifecycle = Lifecycle::restore(PrivateStore::memory(Domain::ReleaseLifecycle)).unwrap();
        lifecycle.observe(Evidence {
            installed: Version(0, 0, 0),
            available_at: PUBLISHED,
        });
        let store = lifecycle.0.lock().unwrap().store.take().unwrap();
        assert_eq!(Lifecycle::restore(store).unwrap().deadline(), None);
    }

    #[test]
    fn invalid_dates_and_noncanonical_versions_are_rejected() {
        for tag in [
            "0.02.6",
            "0.2.6-rc.1",
            "0.2.6+build",
            "+0.2.6",
            "0.2",
            "0.2.6.0",
        ] {
            assert_eq!(Version::parse(tag), None);
        }
        for time in [
            "2026-02-30T00:00:00Z",
            "2026-11-25T24:00:00Z",
            "2026-11-25T00:00:00+00:00",
            "2026-1é-25T00:00:00Z",
            "2026-11-25T00:00:60Z",
        ] {
            assert_eq!(timestamp(time), None);
        }
        assert_eq!(timestamp("2026-11-25T00:00:00Z"), Some(PUBLISHED));
    }
}
