# OWNER-STARTUP-V022-0001 — v0.2.2 flickering report and v0.2.3 follow-up

Recorded on 4 October 2026, Europe/London (BST), at 2026-10-04T09:03:08Z. Status: **FAIL (owner-reported installed v0.2.2 flickering)**. This is a manual owner observation, not an agent-executed test or a universal Windows acceptance verdict.

## Owner report and superseded request

The owner initially requested startup tests be recorded as PASS. On clarification, the owner explicitly stated that **Aster v0.2.2 still has flickering problems**, that their correction is planned for **Aster v0.2.3**, and requested a documentation update. The owner identified the **previously delivered local installer**, not a GitHub Actions candidate. The clarified observed failure supersedes the requested PASS; no startup PASS is recorded.

The report was received on 4 October 2026. The actual test execution date/time, number of launches, observation durations, idle/cold/warm conditions, current Windows/WebView2 build and elevation state were not supplied. Do not invent them or assume the current flickering has the identical timing, appearance or root cause of the earlier v0.2.1 white flash. The owner's stated test policy uses the existing Windows 11 administrator account; no guest or separate account is involved.

## Source and artifact attribution

The following identities come from the qualified local delivery records in [LOCAL-V022-0001](LOCAL-V022-0001.md) and [CI-V022-0001](CI-V022-0001.md). Owner identification of that delivered installer binds this report to its engineering runtime; it is not a new independent hash measurement of the installed executable.

- Runtime R1: `bfff47075c5d94f774e6b9b43ae241f0f72bce25`, tree `2cdcf0a7f69cf31ff98161a1acc98b194b62f049`.
- Local delivered `Aster_0.2.2_x64-setup.exe`: 21,470,693 bytes, SHA-256 `3a9207030fc3f4656d566e08a8a2a58d01e8c613098b90e7a9e8f3b45b2b90aa`, `NotSigned`.
- Recorded packaged application SHA-256: `326464a38ad03f3858c6a020fc5c4d8423c7f271e691c1ee18104101a9296ec8`.
- Reviewed published runtime checkpoint: PR #29 head P2 `82b799f4a0194463c4c3559aa3445ddfed9f71d2`. This documentation amendment changes no runtime or delivered bytes.

Traceability: **NFR-007 → AC-063 → T-073**. T-073 is now **FAIL** because the owner reports persistent installed flickering; source/configuration/headless dark-paint PASS remains separately scoped. Earlier v0.2.1 startup FAIL, local full Cargo FAIL101/T-057/phase8 FAIL, two ignored credential fixtures and unrelated NOT RUN obligations remain unchanged. Signing, broad target-OS acceptance, live provider acceptance and production remain unverified.

## Planned v0.2.3 correction

Keep the flickering defect **open and pending for v0.2.3**. This is a planned follow-up, not an implemented correction, an independently established cause or a promise that unbuilt v0.2.3 bytes pass.

For that version, investigate and bind any correction to its own source, installer/application hashes and fresh regression evidence. Preserve dark native/WebView/early document paint, CSP, visible readiness failures and the existing authority boundaries; do not hide failure with delays/reloads or security/GPU workarounds. Repeat installed idle/cold/warm launches, including at least three idle launches observed for at least ten seconds each and the previously reported 3–5-second window. Record actual environment, steps, UTC execution time and observed results. Only successful observations of the new exact artifact may establish a scoped startup PASS.

The current owner request authorises resolving PR #29 through its normal fresh CI gates and recording this known engineering defect. It does not authorise a signed/production release, a v0.2.3 implementation in this documentation-only amendment, agent app/installer execution, a key/live call or mutation of owner history/spending. Source merge does not close the defect or promote the failing startup gate.
