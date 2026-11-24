# Aster Desktop v0.2.5 security review

On 23 November 2026 all five required engineering jobs passed for [main `a8d612313f94c5d81498fcfe6bd9e698f48aea82`](https://github.com/comzulca/Aster-desktop/commit/a8d612313f94c5d81498fcfe6bd9e698f48aea82), tree `e94927ecaa8f320a69e213656b965c5b22472e14`. Four medium-priority implementation findings remain open. This review supports engineering use with the limits below. Production qualification remains incomplete.

## Architecture and review scope

The WebView expresses intent through a Windows-only command allowlist. Its content security policy restricts network connections to the local application and IPC. The Rust host owns the key, fixed HTTPS destination, native disclosure review and spending admission. It disables redirects and proxies, checks the current key again before dispatch, and records the spending intent durably. The speech route sends local PCM to a fixed decoder. Only text returns to the editable draft. Disclosure to Meta requires a separate action. [Tauri permissions](../../src-tauri/capabilities/main.json) · [WebView policy](../../src-tauri/tauri.conf.json) · [Host admission](../../src-tauri/src/lib.rs) · [Transport](../../src-tauri/src/provider.rs) · [Speech host](../../src-tauri/src/speech.rs)

The source review followed those boundaries through the renderer, IPC, credential proof, native capture, worker startup and packaging. It checked accepted and failed operations, retries, asynchronous completion and the difference between bytes tested in CI and bytes delivered. These are applications of SSDF practices PW.7 (source review) and PW.8 (executable testing). The requirement mapping below uses the project's own acceptance criteria. [NIST SP 800-218 v1.1](https://csrc.nist.gov/pubs/sp/800/218/final) · [SPEC-0009](../specs/SPEC-0009.md)

## Open findings

| Priority and component | Observed in the reviewed source | Evidence required to close |
| --- | --- | --- |
| P2 · Microphone cancellation | Cancel removes the session and listening indicator immediately. A failed cancellation IPC is silently ignored. Windows can continue capture until the 30-second limit. The stale transcript is discarded, and window close has its own host guard. | Keep a visible stopping or unconfirmed state until the host reports capture completion. Cover rejected and delayed cancellation with a simulated bridge and verify device release on Windows. [UI cancellation](../../src/SpeechControl.tsx) · [Native capture](../../src-tauri/src/speech/windows.rs) |
| P2 · Decoder startup | The pinned child starts before the host attaches its Windows Job Object. Resource and child-process limits therefore take effect after startup. The host waits for attachment before writing audio to the pipe. | Apply the limits before the decoder can run, using the existing attachment-worker pattern. Verify ordering and cleanup on setup failure with a synthetic worker. [Speech worker](../../src-tauri/src/speech/windows.rs) · [Attachment worker](../../src-tauri/src/attachment_worker.rs) |
| P2 · Credential retry | Verification clears the stored key fingerprint before requesting model metadata. A transient failure or interruption can leave the identity unknown. Retrying with the same key can then revoke consent, invalidate temporary attachments and sever prior conversation continuation. Saved conversations and spending records survive. | Preserve the last acknowledged identity separately from current authentication proof. Test the same key after interruption and restart, and a genuinely different key. Require fresh successful proof before Send in every case. [Proof journal](../../src-tauri/src/credential_validation.rs) · [Host verification](../../src-tauri/src/lib.rs) |
| P2 · Credential store status | An OS error reading Credential Manager becomes `configured=false` and validation state `missing`. The host blocks Send, but the UI can tell someone to configure a key that already exists. | Surface `unavailable` on an OS error. Verify the displayed state and next action separately for an empty store, a failed read and an authentication rejection. [Host status](../../src-tauri/src/lib.rs) · [Renderer guidance](../../src/App.tsx) |

**Native dependency coverage.** The build pins the whisper.cpp b5907 archive and model, verifies the nine speech resources, and exercises the packaged decoder. The frontend and Rust SBOMs and advisory scans do not inventory the separately supplied native binaries. Record the whisper.cpp and GGML revisions behind the shipped files, map them to the hashes, and review upstream advisories as part of dependency maintenance. This is a coverage gap. The review found no evidence of substituted audio resources. [Pinned inputs](../../scripts/prepare-speech.py) · [CI](../../.github/workflows/ci.yml) · [Dependency update configuration](../../.github/dependabot.yml)

## Requirement coverage

| SPEC-0009 criterion | Evidence on main | Remaining acceptance work |
| --- | --- | --- |
| FR-038 / AC-096 / T-106 | A native prompt precedes capture. The [session tests](../../src-tauri/src/speech.rs) cover one owner, matching IDs and terminal cancellation. | Observe actual input release after Finish, Cancel, timeout and closing the installed app. Include the failed-cancellation case above. |
| SEC-040 / AC-097 / T-107 | The [bridge tests](../../src/bridge.test.ts) constrain speech IPC to a session ID. Package verification checks resource identities. The extracted decoder transcribed the public reference recording. | Verify containment from process creation and cover native advisories. |
| UX-022 / AC-098 / T-108 | The [composer fixtures](../../src/speech-readiness.test.tsx) cover draft insertion, the UTF-8 bound, late results and reclassification after edits. | Confirm the listening indicator against physical capture in the installed app. |
| FR-039 / AC-099 / T-109 | Focus refresh and failed verification are exercised in [UI fixtures](../../src/speech-readiness.test.tsx). [Proof tests](../../src-tauri/src/credential_validation.rs) cover rotation, service errors and stale keys. Two disposable Windows credential cases passed. | Exercise retry across interruption, distinguish store errors, and validate the installed Settings-to-Send journey. |
| NFR-013 / AC-100 / T-110 | CI executed source, Windows and package gates. The resource tests reject missing or substituted assets. | The known-file probe starts downstream of the physical microphone. No live Meta request or interactive installed-app test was performed in this review. |

One specification detail needs clarification. A pre-acceptance send rejection restores the draft. After a later terminal provider failure, the submitted message remains in the conversation and the composer becomes usable again, but the original text is not restored into the editor. Specify which recovery the final sentence of SPEC-0009 requires and test both phases, including a newer draft. [Send and terminal handling](../../src/App.tsx)

## Execution evidence

| Main push run 45232167632, attempt 1 | Observed result |
| --- | --- |
| [Source policy](https://github.com/comzulca/Aster-desktop/actions/runs/45232167632/job/137894304529) | Source secret guard, vendored provenance, locked dependency cohorts and CPU receipt checks passed. |
| [Frontend](https://github.com/comzulca/Aster-desktop/actions/runs/45232167632/job/137894410513) | 200 tests across 16 files passed. The pnpm advisory scan reported no known vulnerabilities. |
| [Codec source fixtures](https://github.com/comzulca/Aster-desktop/actions/runs/45232167632/job/137894410741) | The production codec module and vendored source fixtures passed. |
| [Windows host](https://github.com/comzulca/Aster-desktop/actions/runs/45232167632/job/137894670830) | Formatting, all-target checking and strict Clippy passed. The default Rust library suite passed 422 tests and skipped two disposable credential cases, both of which passed in the dedicated step. RustSec reported three warnings across 493 scanned crates: chrono 0.4.45 (RUSTSEC-2026-0543), proc-macro-error 1.0.4 (RUSTSEC-2024-0370), and glib 0.18.5 (RUSTSEC-2024-0429). |
| [Engineering installer](https://github.com/comzulca/Aster-desktop/actions/runs/45232167632/job/137905352930) | The packaged executable passed all 15 codec and six resource probes. The extracted speech payload passed the known-recording test. |

On the Windows review machine, the frozen pnpm install, TypeScript/Vite build and all 200 frontend tests passed. A fresh pnpm advisory query reported no known vulnerabilities. The local source/provenance guards, 184 negative CPU receipt controls, 177 installer-provenance fixtures and 46 resource fixtures passed. The default sandbox initially blocked creation of Vitest workers and access to the vendor test's temporary directory. The same checks passed with access for those operations. Native Rust execution is attributed to the hosted Windows job.

## Unsigned main installer

The [main run](https://github.com/comzulca/Aster-desktop/actions/runs/45232167632) produced artifact `14850686221`. Independent passive verification in Windows returned `VERIFIED_NOT_INSTALLED` in main mode, with 29 NSIS entries, all 21 required resources and Authenticode status `NotSigned`.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| GitHub Actions ZIP | 67,477,000 | `a4afc411794a378d83d30c195c1b6c8cca51e2ce65b719b95ea62aaa3bb42b15` |
| Aster Desktop_0.2.5_x64-setup.exe | 88,978,237 | `452fb23e09db4fbce45c595b61290ae842b57d7347de5c0df582ba9c41f0969a` |

The package reports `qualificationComplete=false`. The 42 compound protocols, physical microphone, owner-installed GUI, live account and production signing remain open. The [GLib deferral](GLIB-UPSTREAM-DEFERRAL-20261009.md) remains unresolved. Build and verification commands are in [BUILDING.md](../BUILDING.md).
