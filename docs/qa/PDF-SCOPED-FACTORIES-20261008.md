<!-- OCT9 FINAL ENGINEERING APPLICABILITY BEGIN -->
> Final engineering applicability — 9 October 2026, 19:38:51 UTC: the [canonical receipt](FINAL-ENGINEERING-V023-20261009.md) records actual main `54c64e404b3147170bd5d1a320e11536e9e5406e`, push run `37974140580`, attempt 1: all five mandatory CI jobs PASS, fifteen codec/six resource probes PASS, and local passive package verification PASS (`VERIFIED_NOT_INSTALLED`). This is an unsigned engineering build, not installed or production acceptance. `qualificationComplete` remains false; previous FAILs are preserved. GLib remains OPEN and UNRESOLVED; all 42 compound protocols and manual/live/signing/production qualification remain NOT RUN. Earlier status paragraphs below are dated source-bound history; these results do not qualify later changed bytes.
<!-- OCT9 FINAL ENGINEERING APPLICABILITY END -->

<!-- OCT9 CPU AND GLIB OWNER DECISION BEGIN -->
> Owner decision and current applicability — 2026-10-09 17:14:45 UTC: PR #30 is normally merged; its distinct actual-main package run remains FAIL after two CPU-probe failures, not a qualified main installer. The [separate CPU repair contract](CPU-CONTAINMENT-20261009.md) preserves the 10-second user-CPU and 15-second wall budgets and all mandatory gates; changed-source execution remains NOT RUN until fresh evidence. [GLib 0.18.5 remains OPEN and UNRESOLVED](GLIB-UPSTREAM-DEFERRAL-20261009.md), deferred to a complete compatible stable upstream chain; no backport, dependency remediation, dismissal or ignore is authorised. GTK4/Tauri 3 is a future owner-preferred architecture review, not an approved migration or promised fix. Earlier bodies below are retained dated source-bound history; no historical PASS transfers to changed source or owner/manual/live/signing/production acceptance.
<!-- OCT9 CPU AND GLIB OWNER DECISION END -->

<!-- OCT9 FINAL FOLLOW-UP BEGIN -->
> Operational follow-up — 2026-10-09 14:18:44 UTC: the [completed-source reconciliation](OCT9-RECONCILIATION-20261009.md#completed-p11-and-final-documentation-follow-up) records the newer tested-source facts and owner decision. For the observed status of later documentation-only CI, normal integration, actual main-push and Gmail handling, consult the [PR #30 closing receipt](https://github.com/comzulca/Aster-desktop/pull/30#issuecomment-6082465272), [live PR #30](https://github.com/comzulca/Aster-desktop/pull/30) and [Actions](https://github.com/comzulca/Aster-desktop/actions). Earlier cutoffs below remain dated history; green results never transfer to a changed source. The glib alert remains OPEN for separate work; manual/live/signing/production acceptance is not inferred.
<!-- OCT9 FINAL FOLLOW-UP END -->

<!-- OCT9 CURRENT APPLICABILITY BEGIN -->
> Current-source applicability — 9 October 2026, 13:02:58 UTC: the [dated reconciliation](OCT9-RECONCILIATION-20261009.md) owns the newer exact-source outcomes and their limits. Earlier “current”, “pending” and NOT RUN statements below remain source-bound history where superseded there; original bodies and before-code approvals are preserved. P10 hosted success does not qualify a changed cumulative source. Final-source CI/package gates require re-execution; owner flicker FAIL, all 42 compound cases NOT RUN and independent manual/live/signing/production obligations remain.
<!-- OCT9 CURRENT APPLICABILITY END -->

# 8 October 2026 P10 scoped WinRT factory proposal

Status: **BEFORE-CODE governance proposal.** No P10 runtime, package, worker, compound, manual or live-provider result has been obtained. Main-agent review of this complete contract and the seven governance amendments is required before behavioral source or manifest/lock changes.

This amendment narrows the existing SEC-028 / NFR-009 -> TM-029 -> AC-072 / AC-073 -> T-078 / T-082 / T-083 chain. It creates no new requirement, threat, acceptance or test ID. It does not approve v0.2.3, owner acceptance, credential use, project disclosure, signing or a production release.

## Phase 0 - exact baseline, inventory and claim boundaries

### Published P9 observation

The source under observation is PR #30 P9 head `3be1d7365e23b7f2f6c9820fa7c9bae249159b62`, tree `6d4c24794d6ff21714bf528c274a510f26c387e5`, actual tested merge `7fe50dbc24b7774e6681f5fb9227fe1f823c2dc0`, GitHub Actions run `37843254867`, attempt `1`. The local observation is recorded in `evidence/oct8-pr30-p9-native-crash-isolated-pass.json`, observed at 2026-10-08 21:20:09 UTC.

| P9 scope | Observed result | Exact boundary |
| --- | --- | --- |
| Source, frontend and Linux-codec jobs | PASS | These job results do not approve the Windows suite or installer. |
| Complete Windows native suite, job `113538055535` | FAIL | 231 tests announced, 20 individual `ok` outcomes, no aggregate summary, followed by `STATUS_ACCESS_VIOLATION / 0xc0000005` at `2026-10-08T21:13:44.7617924Z`. |
| Separate bounded synthetic PDF diagnostic | PASS | Exactly one reported synthetic test, complete 83-checkpoint trace, cleanup complete, not poisoned, 2,959 captured bytes and 359 ms. Hosted Python 3.12.10 and 112 counted pure checks are diagnostic evidence, not restricted-worker evidence. |
| Engineering installer, job `113544542397` | SKIPPED | Its 15-codec-case and six-resource-case qualifications are NOT RUN; there is no qualified setup from this run. |
| Credential follow-ups, Windows audit/SBOM | SKIPPED | Neither cached historical results nor the diagnostic fill these gaps. |

The full-suite observation contains six test-thread markers, ending in `ASTER_PDF_DIAG_V1|valid|66|enter` around the unchanged `DataWriter::CreateDataWriter` expression. That is the last observed location of the diagnostic test thread, not proof that it is the crashing thread, that the creator caused the access violation, or that a static cache, concurrency, apartment teardown or any other mechanism is the root cause. The isolated case is `attachment_processing::tests::synthetic_pdf_retains_all_pages_in_order_and_rejects_over_limit`; its success does not repair the complete-suite FAIL.

Artifact `11578902930` is 603,990 bytes; API and upload logs agree on archive SHA-256 `dcc6a1daba8cb1878b415c1e8a33ef0134804a7ac76ad898c82724ece155cdc4`. An independent archive download was NOT RUN. Do not convert the matching metadata into independent artifact inspection or installer qualification.

### Fixed authority and historical constraints

The proposed change is private Windows implementation plumbing, not a new tool capability. No browser, provider, network request, credential operation, user file, account, Windows security setting, administrator policy, workspace flag or deployment is needed by the proposed new regression. The owner's existing NVIDIA credential must be preserved and not read, printed, replaced or deleted by this work.

Restricted-token creation, the empty child environment, handle allowlist, create-suspended-before-Job admission, Job resource limits, bounded input/output/time, cleanup, journal/privacy policy and content-free production failure stages remain unchanged. Contributor mode and all four project flags remain false unless separate existing governed prerequisites are actually met. P10 does not satisfy them.

The owner's local v0.2.2 startup/flicker report and local Code Integrity FAIL remain historical observations. No v0.2.3/manual PASS may be inferred. Issues #11 and #12 remain historically CLOSED with completed and not_planned dispositions respectively; their closure is not a new-source gate result. Frozen v0.1.0, earlier revision, owner and source/test ledgers must not be rewritten to imply P10 qualification. The source-specific auxiliary dependency result is a private-store isolated-child lease-denial PASS, not execution of the ignored native Credential Manager units. P10 neither changes that erratum nor inherits credential qualification.

### Primary pinned source/type review

The review concerns the cached primary Rust sources for `windows 0.62.2`, `windows-core 0.62.2` and the already-locked `windows-future 0.3.2`; reading source and the crate archive is not executing it. Before implementation, review the exact generated creator signatures in `Windows/Storage/Streams/mod.rs`, `Windows/Data/Pdf/mod.rs` and `Windows/Graphics/Imaging/mod.rs`, plus `windows-core/src/windows.rs`, `imp/factory_cache.rs`, `imp/generic_factory.rs` and `type.rs`.

The public `windows::core::factory<C, I>()` delegates to pinned `imp::load_factory<C, I>()`. It bypasses the generated `FactoryCache<C, I>::call` in the application creator expression; it is not a guarantee that Windows has no internal cache. The generated cache retains an agile factory in a static atomic slot on its cacheable path. The proposed operation-local helper is intended to avoid that application-level static lifetime. Neither this source structure nor the isolated P9 success establishes that cache retention caused the P9 crash.

The inherited `load_factory` has existing internal behavior: `RoGetActivationFactory`; on `CO_E_NOTINITIALIZED`, an MTA-usage attempt and a second OS activation attempt; and, for `REGDB_E_CLASSNOTREG`, existing runtime-name-based DLL search/activation behavior. P10 calls this same pinned implementation used by the old empty-cache path. There is **no new Aster/caller fallback, loader, retry or path selection**, but it would be false to say the upstream fallback is absent. A balanced explicit caller apartment is retained; it is not proof that every internal branch is unreachable. P10 must not add or retain an Aster-owned MTA cookie or make a new static COM lifetime leak.

## Phase 1 - selected narrow provisional design

### Five fixed operation-local factory wrappers

Add five private, Windows-only, typed helpers. Each class/interface pairing is fixed in its helper's source, not selected by the request, model, file, test input, callback, environment or configuration. The helper obtains an owned factory with pinned `windows::core::factory<C, I>()`, invokes exactly the corresponding native method, then drops the factory normally within the current balanced caller apartment. No factory is cached, stored in a static, leaked, passed between threads or retained across an apartment's teardown by Aster.

| Fixed runtime class | Fixed factory interface | Operation/return correspondence |
| --- | --- | --- |
| `DataWriter` | `IDataWriterFactory` | `CreateDataWriter`, fixed typed input `Param<IOutputStream>`, returning `Result<DataWriter>`. |
| `DataReader` | `IDataReaderFactory` | `CreateDataReader`, fixed typed input `Param<IInputStream>`, returning `Result<DataReader>`. |
| `PdfDocument` | `IPdfDocumentStatics` | `LoadFromStreamAsync`, fixed typed input `Param<IRandomAccessStream>`, returning `Result<windows_future::IAsyncOperation<PdfDocument>>`. |
| `PdfPageRenderOptions` | Public `IActivationFactory` | `ActivateInstance` to owned `IInspectable`, then the normal typed `.cast()` to `PdfPageRenderOptions`. |
| `BitmapEncoder` | `IBitmapEncoderStatics` | `PngEncoderId`, returning the generated method's `Result<GUID>`. |

The helper interface must not expose a free class name, runtime name, IID, generic caller-selected class/interface, loader path, cache-reset hook, retry policy or fallback selection. Typed stream parameters are already-created bounded in-memory interfaces, not caller-selected file paths. Ordinary Rust/COM RAII remains the only application ownership mechanism. Do not replace the generated options conversion with a raw cast or reinterpretation.

Only five creator expressions in `render_pdf` may be replaced, plus the writer/reader creator expressions in the existing stream roundtrip fixture. Every other render expression, stage mapping, async wait/join, page/order/limit assertion, cleanup guard and explicit drop remains unchanged. This includes the existing direct stream capability and over-limit regressions. No image-codec path, capability policy, persistence format, production diagnostic stage or native worker control may be altered. Keep all P9 marker definitions and call sites, and the expected 83-checkpoint trace, the P9 collector, diagnostic bounds/classifications and CI byte-for-byte unchanged.

### ABI, failure and ownership equivalence

Each helper must mirror its pinned generated method's unsafe ABI contract, not construct a new one:

- Use the exact pinned typed interface and its generated vtable, the normal raw `this` pointer and the normal parameter ABI conversion; retain the borrowed parameter for the duration of the native call.
- Initialize the output slot exactly as the generated implementation does. Check the returned HRESULT by the same `.and_then(...)` or `.map(...)` path. A failed native result must return its original error without reading or treating output storage as a successful interface/value.
- On successful interface output, use pinned `Type::from_abi` exactly once for ownership transfer. Its null-interface behavior is internal `Error::empty()`; do not invent a fixed HRESULT/null mapping, use unchecked non-null assumptions, duplicate ownership, transmute an arbitrary pointer, or leak an output.
- The PNG ID output is a GUID value: retain the generated successful-HRESULT `.map` behavior; do not use an interface conversion for that output.
- For options, retain the public activation method's normal `IInspectable` creation and typed `.cast()` result/error semantics, corresponding to the original generated generic factory's sequence. Do not bypass a failed activation or failed cast.
- Return the owned typed value/async operation normally. The factory need survive only the native creator invocation. The original render-owned streams, returned object, pending operations and joins remain governed by their existing ownership/drop order, before the existing apartment guard uninitializes. Do not move uninitialization into a helper or prolong an apartment with a cookie.

Before publication, the main agent must independently compare each helper against the exact pinned source. Review error paths and null semantics even if the platform cannot naturally produce them in a positive synthetic fixture. A static comparison is not a runtime PASS, and fabricated/raw fake COM pointers must not be introduced merely to provoke ABI faults.

### Exact dependency-only exposure

The public async operation return type requires direct access to the already-transitive, already-locked `windows-future` crate. The only manifest addition is under `[target.'cfg(windows)'.dependencies]`:

```toml
windows-future = { version = '=0.3.2', default-features = false }
```

The only lockfile difference is adding `windows-future` to the root application's dependency edge list. Preserve every resolved package/version/checksum and all other lock bytes. No new dependency resolution, checksum, package, feature, target support or transitive upgrade is permitted. Existing `windows 0.62.2` default `std` already enables `windows-future/std`; this direct declaration does not expand the effective feature set. Independently compare that pinned feature path.

The locked `windows-future 0.3.2` checksum is `e1d6f90251fe18a279739e78025bd6ddc52a7e22f921070ccdc67dde84c605cb`; the cached crate archive SHA-256 was independently matched during source/type review. This verifies the reviewed package archive identity, not execution, safety or a successful build. A graph/feature change beyond the one edge is a stop condition, not an opportunity to regenerate the lockfile broadly.

### Exactly one two-apartment regression

Add one Windows-only unit with exactly two successive, non-overlapping, balanced apartment scopes on the same ordinary test thread. Use the same explicit apartment initialization and RAII uninitialization discipline as the existing governed test. Initialization failure is a test failure, not a silent skip. Do not use extra threads, sleeps, providers, credentials, environment selection, user files or a native child for this new unit.

Within each scope, construct a fixed synthetic in-memory stream, create a DataWriter and DataReader using the new helpers, write and read the same short fixed bytes with exact byte-count/content assertions, and retain the original stream/detach/seek/read ownership rules. Also create fresh render options through the helper, check fixed bounded property roundtrip behavior, and obtain the PNG ID through the helper. Check a nonzero successful GUID and equality of the value across the two scopes. Do not assert unequal interface pointer addresses; allocator reuse is not evidence of cached ownership.

Only copyable synthetic scalars/bytes/GUID evidence may cross from the first scope into the second. Drop every COM interface and pending async object in a scope before its guard uninitializes; the second scope starts after the first guard has completed. No captured interface, factory, mutable static or callback may bridge the scopes. Both scopes must run; early-return success after one scope is invalid.

This new unit covers the writer, reader, options and encoder factory families. It does not introduce a PDF parser fixture. The original unchanged complete synthetic PDF unit retains PdfDocument and all-five integration coverage, including valid page order and over-limit rejection. The original full parallel suite remains unchanged apart from inclusion of this single new unit. Do not mark any test ignored, filter or serialize the complete suite, change existing test assertions, add retries, or substitute an isolated run for the complete suite.

## Phase 2 - threat refinement and acceptance plan

### Existing TM-029 threat and residual mapping

| Threat/refinement under TM-029 | Required control/check | Residual/claim boundary |
| --- | --- | --- |
| Generated static factory outliving a balanced operation apartment | Five fixed uncached application helpers; no static owned factory, leaked COM interface or retained Aster MTA cookie; explicit two-successive-scope regression. | Stale cached lifetime remains a hypothesis, not the attributed P9 cause. OS/upstream internals are not proven cache-free. |
| Incorrect unsafe vtable/ABI, failed-HRESULT handling or null output | Exact pinned interface, parameter and output correspondence; generated error conversion and single Type::from_abi ownership transfer; normal options activation-plus-cast. | Positive fixture execution alone cannot cover every native failure/null return; independent source review is required. |
| COM object or async operation survives uninitialization, double ownership or premature drop | Existing render guard/drop/join order unchanged; each helper's factory call scoped; regression retains no interface between scopes. | A source or two-scope PASS does not establish all native thread/concurrency scenarios. |
| Caller gains runtime-class/IID/loader authority or a new recovery bypass | Class/interface pairs fixed privately; no caller-selected names/paths, cache reset, new loader/retry/fallback, provider or environment input. | The pinned upstream load_factory fallback still exists; an absence claim is prohibited. |
| Dependency/feature drift obscures the reviewed native interface | Exact Windows-only pinned declaration and sole root lock edge; unchanged packages/checksums/effective features, with source/archive identity review. | Archive/hash equality is not runtime or license qualification; current-source dependency review remains required. |
| Diagnostic or CI evidence relabels failure as success or leaks content | Original P9 marker definitions/call sites and expected 83-checkpoint trace and collector unchanged; unchanged complete suite; diagnostic remains supplemental; exact source/run/artifact receipts. | Last observed marker is not crash attribution. Isolated PASS never overrides suite FAIL. |
| Fix expands native child or application authority | Restricted token, empty environment, handle allowlist, suspended-before-Job launch, limits and cleanup remain exact; no flag/policy/credential changes. | Hosted library units are not packaged restricted-worker or owner qualification. |
| Historical owner/engineering records are silently rewritten | Dated supersedable amendments only; frozen source/test/owner ledgers remain intact; auxiliary lease erratum stays source-specific. | Closed issues and historical engineering acceptance do not satisfy v0.2.3/manual/live/production gates. |

The positive/regression and fail-closed review remain under existing T-078/T-082/T-083 and AC-072/AC-073. These rows refine existing controls; they are not new acceptance IDs.

### Before-code review gate

The main agent must read this full contract and the dated P10 sections in AGENTS, SPEC-0005, THREAT-MODEL-0005, ADR-0019, verification-plan, CI-V023-0001 and PDF-DIAGNOSTIC before authoring source. Require agreement on exactly five production creator substitutions, only writer/reader fixture substitutions, the one new two-scope unit and the one pinned manifest/root-lock dependency edge. Reject any undocumented creator, behavioral stage/assertion/lifetime change, expanded dependency graph, CI/collector change, new caller fallback or authority change.

The seven local baseline documents were read completely and retained without rewriting their historical content. Their P10 additions are delimited by `<!-- P10 BEFORE-CODE BEGIN -->` and `<!-- P10 BEFORE-CODE END -->`; reversing each exact inserted block must restore the original bytes and SHA-256. The new document is additive. The main agent's before-code review and actual implementation are NOT RUN for this local draft; this file itself is not approval to skip that gate.

### Source/type/security review after the authorized narrow implementation

For the exact candidate source, independently verify the five helper signatures and native invocation/error/null/ownership paths against the pinned sources; options activation-plus-cast; exactly two balanced successive apartment scopes; expected drops; no static/retained factory/cookie or raw caller-selected loader/class/IID; and preserved negative/error-stage behavior. Review the complete diff, not just the newly named functions.

Reverse the permitted creator substitutions, new helpers/test and dependency edge in a scratch comparison to confirm that all other behavioral baseline bytes remain unchanged. Separately require byte equality of CI, P9 collector/marker definitions/calls, resource controls and lifecycle guards. Preserve the absence of per-test filtering/ignore/serialization. Exact rollback-comparison metadata and feature/lock checks should be recorded as source review evidence, never as native execution.

Any compile/type issue must be corrected within this contract or brought back for a new before-code review; it must not trigger a speculative fallback or broader factory rewrite. No final-source hash, tested merge or new test count should be prefilled before the candidate exists.

### Mandatory runtime/package qualification

Run the original complete parallel Windows suite with its ordinary CI command. Record announced count, executed aggregate summary, failures/ignored cases and exit classification from the actual final source; do not predict an aggregate PASS or infer one from individual `ok` lines. The new two-scope unit and unchanged full PDF integration must both complete within that suite. A crash, failed unit, missing aggregate, timeout or incomplete receipt is FAIL/NOT RUN as appropriate, not a successful partial run.

All existing mandatory jobs must independently finish successfully on the final head and actual tested merge before the owner-authorized ordinary PR merge: source policy, frontend, Linux codec, Windows and engineering installer. If the existing Windows diagnostic step runs after failure, preserve its one exact bounded case and separate classification. Its result cannot fix the mandatory job's outcome. Do not change any workflow dependency/condition, use an earlier green run, or omit a job.

The actual installer source/build/artifact identity must be recorded and qualified under the existing engineering gates, including all 15 codec cases and six resource cases. A library test, two-scope unit, isolated PDF diagnostic, source review or another revision's installer cannot satisfy that package/worker gate. No qualified P10 setup exists yet.

All 42 compound cases remain NOT RUN until their complete existing protocol is executed on the exact intended source/package. Owner/manual startup and flicker, actual account/credential preservation and live-provider acceptance require their existing evidence; P10 does not add authorization or turn them PASS. Do not run a provider, touch the owner's Credential Manager data or claim production readiness to improve CI.

### Current P10 evidence ledger

| P10 activity | Current state |
| --- | --- |
| Additive local before-code governance drafting | Draft only; no implementation or main-agent approval recorded here. |
| Candidate behavioral code/helper/test and manifest/root-lock edge | NOT RUN / not authored by this task. |
| New two-successive-apartment regression | NOT RUN. |
| Candidate complete native parallel suite and all mandatory jobs | NOT RUN. |
| Candidate diagnostic, package, 15 codec and six resource qualifications | NOT RUN. |
| 42 compound cases; owner/manual; live provider; production/signing | NOT RUN; no inferred approval. |

Source/type reading and exact cached archive identity review are inputs to the proposal, not replacements for these results. When actual receipts exist, append a dated, source-bound reconciliation; never erase P9's full-suite FAIL or replace frozen ledgers with an unscoped PASS.

## Documentation and publication boundaries

The seven governing amendments and this new QA contract are the complete P10 BEFORE-CODE documentation delta. After implementation and final-source receipts, reconcile all affected current operational/readme/security/testing/qualification/index documentation with actual outcomes and outstanding gates. Keep the historical frozen source/test and owner records unchanged; historical contract/navigation index corrections must identify the actual prior contract rather than imply new qualification.

Publish no P10 source, issue mutation, PR merge, Gmail read-state change, release/tag, installer approval, security/billing change or branch deletion from this local proposal task. Parent/root retains the authorized implementation and external workflow; stop now for complete main-agent review before code.

## Local reversible-baseline inventory

The exact P9 local baselines used for these seven additive amendments are:

| Governance path | Baseline bytes | Baseline SHA-256 |
| --- | ---: | --- |
| `AGENTS.md` | 19,167 | `54704057e25ad3aedafd86753822019a9d66c781a1887cb061a946e620e3851d` |
| `docs/specs/SPEC-0005.md` | 34,730 | `414b5080bba837fa936bc927d75ff35d9b45317a3e597e9b29f663271ec808f2` |
| `docs/security/THREAT-MODEL-0005.md` | 14,352 | `12418793578cc04b2e5f0f3682581a539885110ff2698e1bc4a203a517ea9612` |
| `docs/adr/ADR-0019.md` | 28,900 | `db32767d91ad1506979ef81641d522572463e6fa208d3b96e665d36d7c8f8b15` |
| `docs/aster-builder/verification-plan.md` | 53,345 | `6b891e28df222687f31c7a7e470ee135a0c1f03fce878adb34a542f1a7c935c9` |
| `docs/qa/CI-V023-0001.md` | 49,373 | `56d769482044a7036ebd18fc2cfbd2220e328f8b144a0b386cbf96585556b01c` |
| `docs/qa/PDF-DIAGNOSTIC-20261008.md` | 14,287 | `155cd4e3eef1f590721bf031d69eb8a0fe7b7d78f1a558b6d1cb1dbc76fcc5b8` |

Final amended-file hashes and byte-exact reversal outcomes are to be handed to the main agent with this draft. These local hashes identify the proposed documentation only; they are not a published head/tree or runtime receipt.

## Pinned primary references - source review 2026-10-08

The upstream source revision is `32c3144490c016fe496a0aed769bce60987a2e9d`. The fixed acquisition/cache correspondence is specified in [public factory](https://github.com/microsoft/windows-rs/blob/32c3144490c016fe496a0aed769bce60987a2e9d/crates/libs/core/src/windows.rs) and [FactoryCache/load_factory](https://github.com/microsoft/windows-rs/blob/32c3144490c016fe496a0aed769bce60987a2e9d/crates/libs/core/src/imp/factory_cache.rs). These primary sources include the inherited loader/MTA behavior; they do not establish the P9 crash cause.

The pinned generated native signatures are in [Streams](https://github.com/microsoft/windows-rs/blob/32c3144490c016fe496a0aed769bce60987a2e9d/crates/libs/windows/src/Windows/Storage/Streams/mod.rs), [PDF](https://github.com/microsoft/windows-rs/blob/32c3144490c016fe496a0aed769bce60987a2e9d/crates/libs/windows/src/Windows/Data/Pdf/mod.rs), [Imaging](https://github.com/microsoft/windows-rs/blob/32c3144490c016fe496a0aed769bce60987a2e9d/crates/libs/windows/src/Windows/Graphics/Imaging/mod.rs) and [public WinRT activation](https://github.com/microsoft/windows-rs/blob/32c3144490c016fe496a0aed769bce60987a2e9d/crates/libs/windows/src/Windows/Win32/System/WinRT/mod.rs).

Microsoft documents thread initialization and balancing in [RoInitialize](https://learn.microsoft.com/en-us/windows/win32/api/roapi/nf-roapi-roinitialize) and cleanup in [RoUninitialize](https://learn.microsoft.com/en-us/windows/win32/api/roapi/nf-roapi-rouninitialize). Every successful initialization, including `S_FALSE`, requires a matching uninitialization. These API contracts support the retained balanced scopes; they are not runtime evidence for P10 or attribution of P9's access violation.
