<!-- OCT9 FINAL ENGINEERING APPLICABILITY BEGIN -->
> Final engineering applicability — 9 October 2026, 19:38:51 UTC: the [canonical receipt](FINAL-ENGINEERING-V023-20261009.md) records actual main `54c64e404b3147170bd5d1a320e11536e9e5406e`, push run `37974140580`, attempt 1: all five mandatory CI jobs PASS, fifteen codec/six resource probes PASS, and local passive package verification PASS (`VERIFIED_NOT_INSTALLED`). This is an unsigned engineering build, not installed or production acceptance. `qualificationComplete` remains false; previous FAILs are preserved. GLib remains OPEN and UNRESOLVED; all 42 compound protocols and manual/live/signing/production qualification remain NOT RUN. Earlier status paragraphs below are dated source-bound history; these results do not qualify later changed bytes.
<!-- OCT9 FINAL ENGINEERING APPLICABILITY END -->

<!-- OCT9 CPU AND GLIB OWNER DECISION BEGIN -->
> Owner decision and current applicability — 2026-10-09 17:14:45 UTC: PR #30 is normally merged; its distinct actual-main package run remains FAIL after two CPU-probe failures, not a qualified main installer. The [separate CPU repair contract](CPU-CONTAINMENT-20261009.md) preserves the 10-second user-CPU and 15-second wall budgets and all mandatory gates; changed-source execution remains NOT RUN until fresh evidence. [GLib 0.18.5 remains OPEN and UNRESOLVED](GLIB-UPSTREAM-DEFERRAL-20261009.md), deferred to a complete compatible stable upstream chain; no backport, dependency remediation, dismissal or ignore is authorised. GTK4/Tauri 3 is a future owner-preferred architecture review, not an approved migration or promised fix. Earlier bodies below are retained dated source-bound history; no historical PASS transfers to changed source or owner/manual/live/signing/production acceptance.
<!-- OCT9 CPU AND GLIB OWNER DECISION END -->

# Proposed documented PDF memory-stream replacement — 8 October 2026

Status: pre-implementation proposal. No changed-source Cargo, native worker or package result exists. This proposal does not establish the native root cause or grant an environment, token, Job, filesystem, credential or app-execution exception.

## Source-bound observation

PR #30 P6 head `b79d19862998eaaf2b3f6d2785bee40f7d5cfac3`, tested merge `7476cd3dd73bcb71a20e159d53d15966065e8cd2`, tree `02038302434b990e2d8cee8a0f3a22c3e3d01441`, run `37813484995`, installer job `113446659411`: receipt at `2026-10-08T17:43:07.3888342Z` passed text, PNG, JPEG and static GIF; native-two-page-pdf failed with `pdf-create-input-stream` (reserved stage 65). Ten later codec cases and all six resource probes were NOT RUN. `allPassed=false` and `qualificationComplete=false`. The 21.82 MiB NSIS build is not qualified. Evidence artifact `11567853367` has SHA-256 `0f004062e58ce85f324ba741f4af27be927476d9bcae2c1558a4e68e4c3a1df1`.

The exact source maps stage 65 to `InMemoryRandomAccessStream::new()` returning an error before any PDF bytes are written or document loaded. Locked `windows`/`windows-core` 0.62.2 implements this constructor through `RoGetActivationFactory` and `ActivateInstance`. The stage does not distinguish those calls or expose their HRESULT. No output indicates a PDF parser/content failure at this location.

Read-only registry metadata from the owner's host, not the CI host, records `Windows.Storage.Streams.InMemoryRandomAccessStream` DLLPath as REG_EXPAND_SZ `%SystemRoot%\system32\shcore.dll` when queried with `DoNotExpandEnvironmentNames`. Default registry property access expands this value and must not be cited as a literal registration. Five remaining class registrations (DataReader, DataWriter, PdfDocument, PdfPageRenderOptions and BitmapEncoder) on this host use absolute REG_SZ DLL paths. These facts support an environment-expansion hypothesis but do not establish the hosted registry, actual native error or causal role of SystemRoot. Microsoft documents that an absent environment variable can remain unexpanded, not that it caused this observed activation failure.

## Requirement and acceptance amendment (before implementation)

Existing SEC-028 / NFR-009 -> TM-029 -> AC-072/AC-073 -> T-078/T-082/T-083 applies. The proposed PDF worker uses one fixed documented memory-stream construction path for both the input and rendered output: `CreateStreamOnHGlobal(NULL, TRUE)` followed by `CreateRandomAccessStreamOverStream(..., BSOS_DEFAULT, IID_IRandomAccessStream)`. It never first attempts WinRT class activation and then retries or falls back. It must not use a file-backed stream, temporary file, custom loader, runtime path, explicit factory DLL call, registry override or alternative parser. No original or processed bytes may become a filename, DLL name, environment value or registration.

Allocation and adapter failures remain fail-closed at the existing input/output construction stages 65/77. Existing writer, exact store count, detach, seek, native PDF load, all-page render, output size/exact read, dimension, aggregate, normalisation, cancellation and boundary checks retain their current categories and bounds. The child retains the empty environment, restricted token/Low IL, explicit handle allowlist, suspended creation/Job assignment, limits, termination, cleanup and poisoning. The success protocol and renderer contract remain unchanged.

The remaining WinRT factories are not claimed compatible from registry inspection. Their actual execution, including PDF render options, encoder and reader, must succeed within the unchanged restricted child. If any later stage fails, retain that FAIL and subsequent NOT RUNs; do not widen the environment/token or substitute an unrestricted host route.

## Threat amendment

TM-029 additionally assesses adapter ownership, lifetime, mutable position and error confusion. The memory IStream and IRandomAccessStream are private local COM interfaces owned by the same bounded child. The constructor receives only a null HGLOBAL and fDeleteOnRelease=true: the OS owns the fresh allocation and frees it on final release. No existing handle, caller-owned pointer, GlobalLock, GlobalReAlloc or GetHGlobalFromStream route is introduced. RAII retains the adapter until use completes. Only documented static OS API imports are added. No generic path, file-open, shell, COM class override, native factory export or provider capability is exposed. The adapter can grow in memory like the predecessor stream; existing input/output admission and Job memory/CPU/wall limits remain mandatory. A direct library success does not prove restricted-child containment.

## Tests and evidence (before implementation)

Final COM release frees the allocation but is not secure zeroisation. This change does not prove removal of private bytes from paging, dumps, process memory or other OS-managed copies. The predecessor's physical-erasure and parser-compromise residuals remain; the adapter is not a new confidentiality or network/filesystem sandbox.

- T-082: a Windows-only direct synthetic memory-stream test must construct independent empty streams, assert readable/writable empty output, round-trip a fixed short byte buffer through the unchanged DataWriter/DataReader path with exact stored/loaded counts, verify an independent clone seek position and successful use after the original adapter is dropped, and reject mismatched content. It is not owner-app or restricted-child evidence.
- T-082: retain all four closed-stage diagnostic regressions, normalisation category preservation, PDF-only/codec-only known-code decoding and unknown-code refusal.
- T-083: retain the private error-injection regression's actual FAIL classification, computed false qualification flags, following NOT RUN and zero resource executions.
- T-078/T-083: the actual release executable must pass all fifteen codec cases and all six resource probes, including exact two-page pixel/order checks and page-overflow/malformed PDF denials. No expected failure is redefined or removed.
- Run locked Linux fixtures/source/frontend checks and Windows all-target check, clippy, full test suite, separately authorised hosted disposable credential tests, audit/SBOM/licenses and engineering installer diagnostics against the new exact head/merge/tree. Capture source, principal/OS, command, UTC and artifact identities. Mandatory source/executable/package results remain NOT RUN until genuinely executed. Owner installed/native/live and production gates remain separate.

## Primary Microsoft documentation reviewed

- [CreateStreamOnHGlobal](https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-createstreamonhglobal): documented empty memory stream for a null input handle; fDeleteOnRelease=true automatically frees the private handle on final release. The stream supports reading/writing and coordinated clones.
- [CreateRandomAccessStreamOverStream](https://learn.microsoft.com/en-us/windows/win32/api/shcore/nf-shcore-createrandomaccessstreamoverstream): documented IStream-to-WinRT random-access adapter, returning HRESULT on failure.
- [BSOS_OPTIONS](https://learn.microsoft.com/en-us/windows/win32/api/shcore/ne-shcore-bsos_options): BSOS_DEFAULT uses the base stream's mode and behavior.
- [RoGetActivationFactory](https://learn.microsoft.com/en-us/windows/win32/api/roapi/nf-roapi-rogetactivationfactory): retrieves the registered runtime-class factory; stage 65 is not an actual HRESULT report.
- [Registry value types](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types) and [ExpandEnvironmentStringsW](https://learn.microsoft.com/en-us/windows/win32/api/processenv/nf-processenv-expandenvironmentstringsw): explain unexpanded REG_EXPAND_SZ references and absent-variable behavior. These are hypothesis support, not CI-cause proof.

## Selected API and staging authority

Parent review explicitly selects a deterministic documented memory-stream replacement as a locally staged proposal. The pinned windows 0.62.2 wrapper requires `CreateStreamOnHGlobal(HGLOBAL::default(), true)` (not an Option parameter), then `CreateRandomAccessStreamOverStream::<_, IRandomAccessStream>(&stream, BSOS_DEFAULT)`. Select this over SHCreateMemStream because it avoids Win32_UI_Shell and manual null/allocation-error conversion, while explicitly retaining OS-owned final-release cleanup. Add only windows feature flags Win32_System_Com, Win32_System_Com_StructuredStorage and Win32_System_WinRT; there is no new crate or lock resolution. This selection is not a proven SystemRoot cause or cure. Every mandatory changed-source result remains NOT RUN before implementation.

No implementation is authorised for publication by this document alone. Parent independently reviews the staged diff before any GitHub change.

## Local proposal checkpoint

At the `2026-10-08T17:56:37Z` observation checkpoint, rustfmt 1.97.0, edition 2021, skip_children=true formatted and checked only the staged attachment_processing.rs file with exit 0. This is source-format/syntax evidence, not Cargo type checking or executed native tests. The diff against exact P6 blobs d6b2fe72637f8c9efdb159aea8095c0c13a336be and 22d1e461183cc17b9cea12aaf6f01cf08158baa4 changes the two constructors, one private helper, one Windows-only synthetic regression and the three feature flags only. No worker, fixture, credential, launch, token, environment, Job, framing, guard/category or package-workflow code is altered. All mandatory changed-source gates remain NOT RUN.
