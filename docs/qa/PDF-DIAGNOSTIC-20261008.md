# 8 October 2026 PDF failure-stage diagnostic amendment

<!-- P10 BEFORE-CODE BEGIN -->
## 8 October P10 scoped WinRT factories - before implementation

Read the [P10 before-code factory contract](PDF-SCOPED-FACTORIES-20261008.md) before implementation. This local governance proposal does not implement or authorize a release.

The latest published P9 source is head `3be1d7365e23b7f2f6c9820fa7c9bae249159b62`, tree `6d4c24794d6ff21714bf528c274a510f26c387e5`, tested merge `7fe50dbc24b7774e6681f5fb9227fe1f823c2dc0`, run `37843254867` attempt 1. Its complete Windows suite is FAIL (`0xc0000005`); its separate bounded synthetic PDF diagnostic is PASS. Neither result identifies a crash cause or proves a cure, worker qualification, packaging or owner acceptance.

Keep the P9 diagnostic language, all P9 marker definitions/call sites and the expected 83-checkpoint trace, bounds, collector, classifications and failure gates byte-for-byte unchanged in the proposed behavior delta. The last P9 full-suite marker only locates the observed test thread around the unchanged DataWriter creator; it is not crashing-thread proof. P10 changes creator acquisition only, retaining every render body/join/assertion, guard/drop and diagnostic expression. The new two-apartment fixture covers four factory families; the existing complete synthetic PDF fixture retains PdfDocument/all-five integration coverage.

All new P10 runtime/package/compound results are NOT RUN. Historical P9 and earlier ledgers below remain source-bound records, not current-source PASS claims.
<!-- P10 BEFORE-CODE END -->

## 8 October P9 pre-implementation bounded crash diagnostic

Baseline is published P8 head e12238573d2a4a2ef0ef341b147919eafc62f868, tree 96245af612833f883e0dfaae2528d4a0239ab367, actual tested merge a12bed6f7551bdfc04012e1b7d2b1f343f87a202, run 37831962851 attempt 1. Windows job 113499637646 completed FAIL: 231 tests announced, 20 individual ok outcomes, then STATUS_ACCESS_VIOLATION / 0xc0000005 at 2026-10-08T19:42:44.3402152Z, with no aggregate summary, named failing assertion or crash attribution. Both new direct stream regressions reported ok; engineering-installer 113506283489 was SKIPPED. This is not a PDF cure or restricted-worker result.

The [P9 before-code diagnostic contract](PDF-CRASH-DIAGNOSTIC-20261008.md) adds only a separate closed test-only stderr checkpoint language, not a production PdfFailureStage, worker exit mapping or successful packet. Existing stages64..81 retain their exact meanings; valid/limit enter/leave and body_complete/uninit_enter/uninit_leave are fixed synthetic control locations, not raw HRESULTs or causal claims. cfg(all(test, windows)) private TLS phase is enabled only inside the existing exact PDF case; non-test source has no new producer/caller.

A strict bounded collector accepts only the fixed grammar/order/pairing/repetition, actual one-case result and zero process exit with complete cleanup. Unknown/duplicate/out-of-order/truncated/forged completion cannot pass. Whole stderr/stdout, panic/native text, path/content/key, page-index/dimensions and environment are never replayed. No exception bypass, in-host fallback, split async chain/manual COM release, worker/error-category/authority change or production logging is added. The already failed complete suite executes before diagnosis and stays FAIL; all existing fifteen/six and compound qualification remain mandatory.

Explicit dependency-evidence corrigendum: the auxiliary one-PASS / 157-filtered case is private_store::tests::isolated_native_child_lease_is_denied, not native credential execution. Both native credential tests remain ignored / NOT RUN in those dependency runs. The inaccurate retained P8 reconciliation paragraph must be preserved as dated history with this explicit correction; it grants no Credential Manager PASS and does not negate genuine completed dependency-job results. P8's authorised credential follow-up was itself SKIPPED.

P9 is a before-implementation diagnostic proposal: no P9 source SHA, run, design/runtime/package PASS or qualified payload is claimed. All earlier P8/P7/P6/P5 and other failures, owner startup/Code Integrity results and manual/live/production gaps remain source-bound history; all 42 compound tests remain NOT RUN. The separate P8 design review cannot promote P9.

## 8 October P8 constructor-capability mapping — before implementation

Latest executed baseline is P7 head `ddb932ddd70fb141b17e07819ca9c0bd44f92956`, tested merge `e9e63fcdaf0d0726100c191def2efe4f91ca9a77`, identical tree `a50005e0757c68c7c8f2dcaa821fc8f0d24a6774`, [run 37824444382](https://github.com/comzulca/Aster-desktop/actions/runs/37824444382), attempt 1: three independent jobs PASS, windows-host `113473931021` FAIL, engineering-installer `113481749294` SKIPPED. At `2026-10-08T18:46:06.7574475Z` the library reports 227 PASS, one FAIL, two ignored; the retained direct stream test failed its original `fresh.CanWrite().unwrap()` assertion. P7 did not measure IStream::Stat. The earlier exact-P7 phases 0–2 design PASS remains scoped historical evidence, not native success or P8 qualification.

The [P8 pre-implementation contract](PDF-SH-MEMORY-STREAM-20261008.md) replaces the backing constructor only, retains fixed typed BSOS_DEFAULT adaptation and requires successful true CanRead/CanWrite before returning the adapter. Null memory allocation maps internally to documented E_OUTOFMEMORY; false capability maps internally to fixed E_FAIL; API errors propagate internally. All are returned at existing closed constructor stages 65/77. Eighteen static codes/reasons, PDF-input/codec-mode-only decoding, generic normal renderer refusal and successful packet validation remain unchanged. Never publish the internal HRESULT, error prose, stream name/path, private bytes or a successful content packet from these failures.

Retain the original read/write assertion and all mapped-error/normalisation/category/operation-only and injected-stop/false-receipt regressions. Add synthetic Stat(NONAME) access-mode verification and pure capability rejection before new native qualification. P7's direct CanWrite failure did not measure Stat and is not a restricted-PDF stage receipt. No cause/cure, retry/fallback, broader environment/token/Job, diagnostic data or erasure/confinement claim follows.

P8 is a before-implementation proposal with no source SHA/run or fresh design/runtime/package PASS. The earlier documents below retain their original source-specific checkpoints where superseded here. Preserve P7 FAIL/SKIPPED, P6/P5/earlier execution failures and owner/local startup/Code Integrity failures; all 42 compound tests remain NOT RUN.

## 8 October 2026 executed P6 and memory-stream successor scope

Latest completed diagnostic source is P6 head `b79d19862998eaaf2b3f6d2785bee40f7d5cfac3`, actual tested merge `7476cd3dd73bcb71a20e159d53d15966065e8cd2`, equal tree `02038302434b990e2d8cee8a0f3a22c3e3d01441`, [run 37813484995](https://github.com/comzulca/Aster-desktop/actions/runs/37813484995), attempt 1. Its four new pure regressions passed in windows-host `113436577060`; this is scoped regression evidence, not compound T-082/T-083 or native containment/PDF PASS. Installer `113446659411` receipt at `2026-10-08T17:43:07.3888342Z` passed four codec cases, failed `native-two-page-pdf` at `pdf-create-input-stream`, and left ten codec/six resource cases NOT RUN with false qualification flags. Preserve P5 and earlier failures. The 21.82-MiB compiled setup is unqualified.

The P6 stage identifies its former input-stream constructor before any PDF bytes are written/document loaded; the HRESULT and activation subcall remain unreported. No SystemRoot/environment/token cause or cure is established. The pre-implementation [PDF-MEMORY-STREAM-20261008](PDF-MEMORY-STREAM-20261008.md) amendment selects a fixed documented COM memory allocation/adapter construction for stages 65/77 without a prior activation attempt, retry or fallback. Therefore the staged P7 mapping covers **native stream/WinRT/normalisation errors**, including the two COM constructor error-return sites, not only WinRT activation. It retains the exact eighteen static stages/codes, generic numeric/invariant/boundary categories and PDF-codec-only nonzero-error semantics. No raw native message/HRESULT/private bytes/path/environment/key reaches receipt or renderer.

P7 remains locally staged, not a published head or executed result. Its new direct memory-stream regression, existing four diagnostic regressions and complete fifteen/six actual-executable probes need fresh exact-source execution. Empty environment, token/Low IL, Job limits, inherited handles, packet/admission and cleanup poisoning remain unchanged. Final COM release/free is not secure zeroisation; unmanaged memory paging/dump and parser-compromise/no-network/filesystem residuals remain. No installed/manual/live/package/production PASS follows.

All baseline/proposed/uncommitted and validation statements below are retained dated pre-P6 records, not current execution status. This overlay expands only the two mapped native construction sites for the specified successor, not the successful protocol or diagnostic data permitted.

Status: proposed diagnostic change; final-source compilation and hosted execution NOT RUN.
Baseline: draft PR #30 head fcf5526c0889a6df737925718d1c2ff323baa4d1. Latest run 37207862545 tested merge 2664cf18592d38bb7cae6d2eff5e1dae5512bd51 and failed engineering-installer job 111457997051. Retain every predecessor failure and NOT RUN outcome.

## Requirement and acceptance amendment (before implementation)

This narrows existing SEC-028 / NFR-009 -> TM-029 -> AC-072/AC-073 -> T-078/T-082/T-083. The actual-executable synthetic receipt shall identify one static stage from a closed enum when a mapped Windows Runtime call (including runtime initialisation) fails, or rendered-page PNG normalisation returns Invalid/Unsupported/Worker. Coverage is deliberately limited to these exception/error-return sites. Normalisation Limit or boundary/cancellation errors retain their existing categories. Short store/read counts, numeric bounds, empty/oversized output, dimension mismatches, page/aggregate validation and other invariant guards retain their existing Invalid/Limit categories and generic child exit 3. Panic, I/O and unknown failures also remain generic; no stage is invented for them. The receipt must not return WinRT error messages, HRESULT values, source bytes, filenames, paths, environment contents, principal identifiers or credentials. A stage is a diagnostic location, not a proven cause, a bypass authorisation or a successful PDF result.

The successful worker binary protocol and all admission, validation, token, empty environment, inherited handles, suspended Job assignment, CPU/memory/wall/process bounds, termination, cleanup poisoning, temporary originals and disclosure/storage rules remain unchanged. Reserved nonzero child exit codes carry only closed PDF stage identities. The parent may interpret these codes only for its exact PDF codec operation and must return an error without accepting content. Unknown, non-PDF or resource-probe exit codes remain generic worker failure. No retry, in-host PDF processing or unrestricted fallback is added.

## Threat amendment

TM-029 additionally covers diagnostic text/data exfiltration, confusion of a child exit code with successful content and misclassification of unknown or unrelated failure codes. Use exhaustive static stage/reason/exit mappings, strict known-code decoding and operation-bound admission. The renderer receives no stage string, error payload or new capability. A compromised child can lie about a failure stage, but cannot use the stage to admit content. The stage therefore guides bounded synthetic debugging only and cannot prove the native cause or parser confinement.

## Mandatory regression definitions (before implementation)

- T-082: every reserved stage round-trips to its unique nonzero code; generic/unknown codes are refused; names contain only ASCII lowercase/hyphens and have bounded length.
- T-082: even a known stage exit code is a generic error for image/text or resource-probe mode; no stage code is success. Existing unknown/suffix/output/source binding tests remain mandatory.
- T-082: the exact normalisation error-mapping helper preserves Limit, Isolation, Cancelled, Timeout and an existing PDF-stage error; only Invalid/Unsupported/Worker maps to the closed normalisation stage. This pure regression does not decode a PDF or credit resource containment.
- T-083: a private injected-processing/resource seam exercises the same production classification, later-work stopping and report computation. Inject every Pdf(stage) error on an accepted synthetic PDF case; require its actual branch to emit FAIL/static reason, the following case and all six resource probes to become NOT RUN, exactly one processor call and zero resource calls, and computed allPassed=false/qualificationComplete=false. No private error fields can be serialised. This seam adds no runtime IPC or arbitrary fixture input route.
- Existing fifteen codec cases and six actual-executable resource probes, strict malformed whole-input cases, all-page PDF pixel/order/page-overflow assertions and all five CI jobs remain mandatory. No expected FAIL is converted into PASS.

## Source-bound evidence

Latest Windows library test synthetic_pdf_retains_all_pages_in_order_and_rejects_over_limit passed at 2026-10-04T14:20:09.5312853Z in job 111452909256. The actual restricted executable receipt at 14:48:54.5718866Z passed text, PNG, JPEG and static GIF, then failed native-two-page-pdf with worker-failed-on-valid-synthetic-input. Ten later codec cases and six resource probes were NOT RUN. NSIS build reported 21.87 MiB, which is not package qualification. The library success does not prove the restricted worker route.

The predecessor installer job 111446599731 (run 37204516581) failed stdout capture with OS error 232. Current explicit pipeline consumer fixes capture and exposes the distinct PDF failure; preserve both results. Current source collapses child errors/panics to exit 3 and parent Worker, so the exact WinRT/normalisation stage and narrower native cause are unproved. Empty environment or low integrity are hypotheses, not grounds to weaken controls.

## Validation plan

Run the existing locked Linux production-module codec harness, Windows all-target Cargo check/Clippy/full fixture suite, exact synthetic credential tests on the disposable hosted runner only, and engineering-installer actual-executable diagnostic with all fifteen/six outcomes. Capture exact head/merge tree, run/job IDs, UTC, principal/OS, commands and receipt. If a native PDF stage still fails, retain FAIL and diagnose from that evidence; no final package or native/installed/live PASS follows. No owner app/setup/private data is executed or modified by this proposal.

## Local source-format checkpoint

Follow-up at 2026-10-08T16:53:34Z: the revised private fixture-injection seam and category-preserving normalisation mapper passed the same four-file rustfmt syntax/format command (exit 0). Four new regression definitions remain NOT RUN; no executed Rust/native/package result is implied.

At 2026-10-08T16:42:56Z, rustfmt 1.97.0 with edition 2021 and skip_children=true checked only the four staged Rust files and returned exit 0. This is syntax/format evidence, not Cargo type checking, executed Rust tests, restricted-worker or native PDF PASS. Cargo/full Windows/native/package checks remain NOT RUN for this uncommitted staged proposal. Static comparison additionally confirms unchanged token, Job-limit, child-boundary-verifier, launch-setup and cleanup regions against the exact baseline blobs.


