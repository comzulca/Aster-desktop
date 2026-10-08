# 8 October 2026 source and documentation reconciliation

## Dated successor preparation checkpoint — 8 October 2026, 18:59 UTC

This is an in-progress evidence amendment, not the final completion record. Main is v0.2.2 at cf126816ac7347110fb522acfac3838813108d47 after normally merging PR #32. Its tree bef3dd2765df63ec8a565c8821d1873e79476289 equals tested merge be69a74a3334834215048fede3ef634be1bda035/head88a0cb305c2344975aab44e756539d7f756c14fe; all four jobs passed in run37822485130. Its unsigned candidate is 21437413bytes/SHA25663acdd05f7b267c9ff3e63b3fcb8073c2264128fa7b1b6f994a90cd797898768, built2026-10-08T18:55:12.7102196Z, artifact11572566869/archiveSHA25612a77d7259cc7ee7c9756e7cf106d38329506146718ff17113d667162cb651bb. This is hosted candidate evidence, not an owner-delivered/installed or production artifact.

PR #31's actual main push run37822321356 independently passed all four jobs. Its unsigned installer is 21437413bytes/SHA256757c9c31625acb9871f558f76ed44464663bfb0e72e81d5f8c40fb568d1b6e34, built2026-10-08T18:41:10.4995904Z, artifact11572055353/archiveSHA2569597ae237a314230860bddfd7079b44ed0fde5832677c399f6357771cd50add5. Same-source trees do not imply identical binary hashes.

PR #33 is refreshed safely on actual main #32: headb550d73e50fe75c804377f1ccd3eb5b080258ffa/tested merge95ac5bc90d56101bb9c223575390373c01357abd/tree89d9efbda1b403ce8daea44315f2783393206ca6, run37828328378. The full152leaf comparison preserves150 leaves plus reviewed API manifest/lock delta; Cargo19f8f19a793aa3bfc19d6931f96b92c8a172ae1f retains Tokio1.53.2/dialog2.8.1. Source-policy passed; other gates are pending, not inherited PASS. PR #34–#37 still require individual latest-main refresh and every fresh mandatory gate before merge.

### Actual P7 completed FAIL, retained separately from the proposal

P7 head ddb932ddd70fb141b17e07819ca9c0bd44f92956, tested merge e9e63fcdaf0d0726100c191def2efe4f91ca9a77, equal tree a50005e0757c68c7c8f2dcaa821fc8f0d24a6774, run37824444382 attempt1: source-policy113473513202, frontend113473585544 and codec-source-fixtures113473585597 PASS; windows-host113473931021 FAIL; engineering-installer113481749294 SKIPPED. Windows formatting/check/Clippy passed, but the new documented_memory_stream_round_trip_clone_and_lifetime regression failed at line1193 on fresh.CanWrite() at2026-10-08T18:46:06.7567770Z. Final library result227PASS/oneFAIL/twoignored at18:46:06.7574475Z, exit1 at18:46:07.3639321Z. The unrestricted synthetic PDF test passed separately; no restricted-child PDF, fifteen-codec/six-resource or installer qualification ran. Later separate authorised credential/audit evidence cannot be credited after the failed step. Windows evidence artifact11571278097/archiveSHA256734b1d51587f14dbed7f69e674b25c07319ce450f844df941fddf01c3fb961c7 records this failure, not a qualified setup. The hosted runner is not the owner's interactive Windows11.

The [P7 design review](DESIGN-P7-20261008.md) is exact-source scoped design-only PASS, not runtime acceptance; this later actual test FAIL does not rewrite that earlier scope. P6 stage65, P5 PDF, P4 stdout, earlier failures and owner v0.2.2 flicker/local Code Integrity FAILs remain immutable historical observations.

The P8 [pre-implementation contract](PDF-SH-MEMORY-STREAM-20261008.md) selects one documented SHCreateMemStream(None) backing allocation, the same typed BSOS_DEFAULT adapter and fail-closed CanRead/CanWrite checks. A synthetic Stat(NONAME) access-mode assertion supplements every unchanged original clone/lifetime/round-trip assertion. P7 Stat was not measured: a mode-metadata mismatch is an inference, not a proved cause. No token, environment, loader, Job, budget, format, parser, worker fallback/retry or qualification weakening is authorised. P8 is staged with fresh design/source/native/probe/package gates NOT RUN until its exact published source is reviewed/executed. No merged v0.2.3 or qualified owner hand-off exists.

### Supplemental registry metadata gap resolved

At2026-10-08T18:28:58.0749601Z the remaining primary npm registry metadata read for @tauri-apps/cli-darwin-arm64@2.12.1 succeeded and its integrity matched the exact cumulative candidate. The reviewed51/51 changed package metadata/integrity/license fields are complete. The earlier50/51 transport-gap checkpoint below remains dated history. This does not prove downloaded tarball/license text, regenerated installed notices, runtime compatibility or production redistribution rights; hosted final-source dependency/license/SBOM/package gates remain separate.

Three reviewed CI failure notices remain UNREAD pending actual resolution: retained P5, P6 and new P7. Only four superseded earlier notices had UNREAD removed. No unrelated promotion, key/security/subscription mutation or notification change was made. Final fresh primary/broad searches and final documentation/data reconciliation are still required.

## Retained P7 prepublication checkpoint


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
