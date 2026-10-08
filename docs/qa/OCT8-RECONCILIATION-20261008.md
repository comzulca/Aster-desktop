# 8 October 2026 source and documentation reconciliation

Status: **work in progress**, not completion, installed acceptance or production release. Observed source facts below supersede current-status prose in retained earlier checkpoints, not their original results. Later source changes need fresh applicable gates.

## Authority and release boundary

The owner explicitly authorises correction and normal integration of PR #30 only after every mandatory final-source gate passes, and individual reviewed/gated merges of Dependabot PR #31–#37. There is no gate bypass, manual-test approval, signing or production promotion in that authorisation. The owner asks all affected documentation to reflect the completed changes and remaining obligations. This record is navigation/evidence, not runtime authority.

## Completed dependency integration

| PR | Change | Exact completed result |
| --- | --- | --- |
| #38 | source-map-js 1.2.1 → 1.2.2 on both css-tree/PostCSS edges | Normal merge a3955c7fe76a74908bc22b94f9b4aec9472bb9e1; tested tree f2ae0ccb06d0cfa203b3dfa557cc8b5aa741f0da; all four jobs PASS in run37811857521 attempt1. Post-merge main run37816308691 also passed all four jobs. |
| #31 | Tokio 1.53.1 → 1.53.2 | Normal merge 56bd9db07993e376f983812b63ea3ebd786db06b; tested head755fa07931c55ea9b481a84a1652d0569ce77b90, merge2595f2f55a75b1103f06e5e3668fd0d412c7c099, tree304d74b8dd57793fbc527e5f4d1ddc536a6f09c0; all four jobs PASS in run37816551412 attempt1. |

The observed main after PR #31 is v0.2.2 at 56bd9db07993e376f983812b63ea3ebd786db06b. Its tested dependency tree does not contain the unmerged v0.2.3 application work. See [source-map-js advisory correction](DEPENDENCY-20261008-0001.md). No override, ignored advisory, audit-threshold or workflow weakening is introduced.

Both completed dependency candidates passed 69 frontend tests, frozen lock installation, type checking, build and moderate-threshold audit. Windows library evidence is 156 PASS/two ignored, with the separately executed authorised hosted synthetic credential fixture recorded in the corresponding logs. Broad-lock RustSec warnings RUSTSEC-2024-0370 (proc-macro-error 1.0.4, unmaintained) and RUSTSEC-2024-0429 (glib 0.18.5, unsound) remain explicitly allowed and disclosed, not remediated or zero-warning evidence.

PR #31's installer is an unsigned engineering candidate: Aster Desktop_0.2.2_x64-setup.exe, 21437413 bytes, SHA-256 200d883ea2613c8cee0aec25e12193ac7088a06e277573742182441c03945aed, built at 2026-10-08T18:07:17.4181319Z, artifact11569985601, archive digest98ff356df1b0d94536d56a7997380f5a1f00ef876d15dd26b735692a6078427e. It was not downloaded, installed or executed on the owner's host by this task. It is not the earlier delivered owner installer or proof of a flicker fix.

## Remaining individual dependency gates

| PR | Intended reviewed update | Current disposition at this record |
| --- | --- | --- |
| #32 | tauri-plugin-dialog 2.8.0 → 2.8.1 | Refreshed head88a0cb305c2344975aab44e756539d7f756c14fe; cumulative treebef3dd2765df63ec8a565c8821d1873e79476289 preserves Tokio/source-map fixes. Run37822485130 started; final gates pending, not merged. |
| #33 | @tauri-apps/api 2.12.0 → 2.12.1 | Pending individual latest-main/final-source qualification. |
| #34 | Vite 8.3.1 → 8.3.2 and required upstream lock delta | Pending individual latest-main/final-source qualification. |
| #35 | Vitest 5.0.2 → 5.0.3 and required upstream lock delta | Pending individual latest-main/final-source qualification. |
| #36 | jsdom 30.1.1 → 30.1.2 and required upstream lock delta | Pending individual latest-main/final-source qualification. |
| #37 | @tauri-apps/cli 2.12.0 → 2.12.1 and platform packages | Pending individual latest-main/final-source qualification. |

Conditional cumulative staging, immutable-source/lock graph review, primary manifest compatibility, lockfile-only frozen supply-chain checks and zero-vulnerability local audits are narrower than installed frontend, Windows or installer PASS. The supplemental registry review checked 50 of 51 changed npm package identities/integrities/licences; Darwin-arm64 CLI metadata remained unavailable after bounded transport failures. Do not claim complete supplemental licence evidence from that partial check. Hosted final notice/SBOM/license gates remain mandatory. Original seven failed audit runs and skipped dependent jobs retain FAIL/SKIPPED.

## PR #30 completed predecessor and successor

P6 headb79d19862998eaaf2b3f6d2785bee40f7d5cfac3, tested merge7476cd3dd73bcb71a20e159d53d15966065e8cd2, tree02038302434b990e2d8cee8a0f3a22c3e3d01441, run37813484995 attempt1 completed **FAIL**. Source-policy, frontend, codec-source-fixtures and windows-host succeeded. Scoped evidence includes 100 frontend tests, 14 production-module Linux fixtures plus ten vendor-private fixtures, 227 Windows library PASS/two ignored, four diagnostic regressions PASS, and two separately authorised hosted synthetic credential tests with one PASS each. Direct unrestricted PDF library success does not qualify the restricted child.

Engineering-installer job 113446659411 failed: receipt `2026-10-08T17:43:07.3888342Z` passed text/PNG/JPEG/static GIF, then native-two-page-pdf failed with closed reason `pdf-create-input-stream` (stage 65). Ten later codec cases and all six resource probes are NOT RUN; `allPassed=false` and `qualificationComplete=false`. The built 21.82-MiB setup is unqualified. Failure evidence artifact 11567853367 has digest `0f004062e58ce85f324ba741f4af27be927476d9bcae2c1558a4e68e4c3a1df1`. P1–P6 failures remain historical, not changed to PASS.

The [documented memory-stream amendment](PDF-MEMORY-STREAM-20261008.md) specifies one COM memory-stream construction/adapter path for both input and output, not a retry, environment exception, custom factory/DLL loader, file-backed stream or unconfined fallback. It retains every admission, resource, cleanup, token, empty-environment, handle and packet control. A Windows-only clone/lifetime regression and three feature flags are added without a new crate version. Constructor errors remain closed stages65/77; the renderer retains a generic denial. Stage65 localises construction, not a proven SystemRoot/HRESULT cause or cure. Final COM release is not secure zeroisation; paging/dump/OS-copy and compromised-parser confinement residuals remain.

The locally reviewed successor design and format checks do not establish a published-source or execution PASS. Every successor gate is **NOT RUN before publication/execution**. It must pass all five CI jobs, all fifteen actual-release-executable codec cases and all six resource probes on final latest-main source before normal integration. A later dependency or documentation amendment needs fresh final-source checks.

## Unchanged independent obligations

Owner-installed Windows11 startup/focus/DPI/accessibility, native disclosure/cancellation/history/spending and live/account/terms acceptance remain source/artifact-bound independent gates. Preserve the owner's local v0.2.2 flicker FAIL and local Code Integrity/headless failures. No hosted result or documentation update creates an owner manual PASS. No owner app, real credential target, paid live request, Windows setting or production release was changed by these hosted corrections.

Historical ledgers, original source/installer identities, upstream licences and dated evidence remain unchanged. Current documents should link this record and [the v0.2.3 index](../V0.2.3-DOCUMENTATION-INDEX.md); frozen evidence describes only its own date/source. Final reconciliation and its source/CI results are still pending.
