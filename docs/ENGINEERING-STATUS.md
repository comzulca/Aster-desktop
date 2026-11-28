# Current engineering status

## 28 November 2026

[PR #99](https://github.com/comzulca/Aster-desktop/pull/99) reached main at `3115186`. The manifests declare `0.2.5+1`, the numeric Windows product version is 0.2.5.1, and the host reports IPC 6 and policy 4. Projects and the local Markdown vault are included in this source. Live full-window compaction remains disabled. The candidate evidence below records the stage at which it was collected.

## 27 November 2026: v0.2.5.1 engineering candidate

See the [v0.2.5.1 candidate contract and pre-CI evidence](qa/V0251-ENGINEERING-CANDIDATE.md). At the pre-merge checkpoint, the numeric Windows product version is 0.2.5.1 and the spending IPC version is 6; the earlier main and its artifacts retained the identities below. The full-window work remains an offline qualification path and no Windows installer or live feature is implied by these local results.

## 25 November 2026: Windows release lifecycle

[PR #90](https://github.com/comzulca/Aster-desktop/pull/90) reached main at [`e40943d`](https://github.com/comzulca/Aster-desktop/commit/e40943d0cecefd6ec64c9f87daf3a331f3c3b3a3). The [change contract](qa/RELEASE-LIFECYCLE-20261125.md) replaces the fixed 6 December cutoff in new builds with a 14-day grace period after a newer stable Windows release has a matching installer. All five jobs passed on [PR candidate `c602f83`](https://github.com/comzulca/Aster-desktop/actions/runs/45565643154). Its installer is an unsigned PR artifact, not an installed copy, a stable release or the [separate main build](https://github.com/comzulca/Aster-desktop/actions/runs/45570054196).

[PR #92](https://github.com/comzulca/Aster-desktop/pull/92) reached main at [`8f4069e`](https://github.com/comzulca/Aster-desktop/commit/8f4069e87dd718dc1e0bc527fd3fa0ecb24e4381). It accepts both exact installer filenames, with the original space or GitHub's dot, without changing the stable-version, architecture, upload or timing checks. [PR #92 CI](https://github.com/comzulca/Aster-desktop/actions/runs/45575027647) and the [synchronised documentation PR #91 CI](https://github.com/comzulca/Aster-desktop/actions/runs/45599379976) each passed all five jobs on their respective PR candidates. PR CI does not qualify an installer from main or constitute a stable release.

The [main run for `94baeec`](https://github.com/comzulca/Aster-desktop/actions/runs/45603586648) passed all five jobs and includes the installer-name fix from PR #92. The [Windows x64 installer from this run](https://github.com/comzulca/Aster-desktop/actions/runs/45603586648/artifacts/15018076220) is an unsigned engineering artifact, not a stable release, and does not start the 14-day clock.

[PR #96](https://github.com/comzulca/Aster-desktop/pull/96) reached main at [`971b0c1`](https://github.com/comzulca/Aster-desktop/commit/971b0c1d2dfea225af70bf9c2c13e0944893cfc4). When the host reports that the installed version has expired, Settings disables the native review buttons for Contributor terms and spending configuration. Saved history and reconciliation of pending charges remain available. The unsigned `94baeec` installer linked above predates this Settings fix. See the [PR checks](https://github.com/comzulca/Aster-desktop/actions/runs/45638484220) and the [separate main run for `971b0c1`](https://github.com/comzulca/Aster-desktop/actions/runs/45644541539). Neither run publishes a stable release.

[PR #87](https://github.com/comzulca/Aster-desktop/pull/87) addressed four findings from the 23 November review: microphone-release acknowledgement, decoder Job Object assignment at creation, preservation of known key identity during transient verification failures, and distinct guidance when Credential Manager cannot be read. [PR #88](https://github.com/comzulca/Aster-desktop/pull/88) changed the pnpm pin to 11.28.12 without changing resolved dependencies. [PR #89](https://github.com/comzulca/Aster-desktop/pull/89) added installed-payload provenance for all nine packaged speech resources to Actions; this does not add advisory coverage. Owner-operated Windows, physical microphone and live provider acceptance remain separate.

## 23 November 2026

All five jobs passed on [main `a8d6123`, attempt 1](https://github.com/comzulca/Aster-desktop/actions/runs/45232167632), after the dependency update in [PR #84](https://github.com/comzulca/Aster-desktop/pull/84) and interface preview in [PR #85](https://github.com/comzulca/Aster-desktop/pull/85). The unsigned Windows installer passed independent passive verification. The [security review and installer receipt](qa/SECURITY-REVIEW-V025-20261123.md) traces SPEC-0009 to the executed tests and identifies open defects in microphone lifecycle and credential handling. Full qualification remains open.

[PR #83](https://github.com/comzulca/Aster-desktop/pull/83) integrated v0.2.5 at `e348b63`. Dictation transcribes locally into an editable draft. The user classifies the text and reviews disclosure before sending. Returning to the app refreshes the saved credential state. All five jobs passed on [the final PR candidate](https://github.com/comzulca/Aster-desktop/actions/runs/45052852286), including the extracted installer speech probe. Physical microphone capture and installed Windows GUI acceptance still require an interactive session. [Setup and use](SETUP-AND-USE.md) · [Specification](specs/SPEC-0009.md).

The 12 direct frontend packages matched the npm latest tags. Updating the transitive lock moved @babel/runtime from 7.29.14 to 7.29.15 and undici from 8.12.4 to 8.13.1. Cargo updated anyhow, cookie, hashbrown, jiff, jiff-static, toml, toml_edit, toml_parser and walkdir in the [host lock](../src-tauri/Cargo.lock). Both QA locks resolved without changes. A second host resolution found zero compatible updates. Cargo retained older generic-array, toml, toml_datetime and toml_edit entries in the graph and the Windows ABI pin on windows-future 0.3.2. The three workflow actions matched their upstream stable releases.

## Earlier maintenance

Dependency maintenance, 20 November 2026. The source locks the Tauri Rust crate at 2.12.7, the JavaScript API at 2.12.5 and the CLI at 2.12.6. The test suite uses user-event 14.6.13. The frontend uses jsdom 31.0.1, Vite 8.3.8 and Vitest 5.1.1. [Integrated changes and CI for main `0766478`](RELEASE-NOTES-v0.2.4.md#dependency-maintenance--20-november-2026). [Retained dependency constraints](qa/DEPENDENCY-REVIEW-20261119.md).

**Earlier spending/Send maintenance:** [PR #70](https://github.com/comzulca/Aster-desktop/pull/70) implements the [spending/Send correction](qa/SEND-READINESS-20261105.md), IPC protocol 5 and unchanged financial policy 4. Its initial completed candidate run is `42109052799`, attempt 1, tested merge `8e92ef76a4e7910ae2193e87e7085c4910b58d1e` from head `c6974cd`. All five jobs and packaged probes passed. The PR delivery receipt separately binds final main source, run and installer hashes.

Reconciled 5 November 2026. The previous delivered software snapshot was [`57339a687125a0b1fd8d6cda787649b5a111fa65`](https://github.com/comzulca/Aster-desktop/commit/57339a687125a0b1fd8d6cda787649b5a111fa65), after [PR #68](https://github.com/comzulca/Aster-desktop/pull/68), with [main CI 42081672227, attempt 1](https://github.com/comzulca/Aster-desktop/actions/runs/42081672227). These identities describe the software artifact; later documentation revisions are not a new installer qualification.

## Capability and evidence matrix

| Area | Implemented behaviour | Evidence / remaining boundary |
| --- | --- | --- |
| Local dictation | Windows x64 v0.2.5, 30-second capture and editable transcript | Hosted CI transcribed a public recording with the extracted decoder. Current source waits for release acknowledgement and assigns the decoder to its Job Object at creation; physical microphone and installed Windows acceptance remain open. |
| Previous engineering installer | Unsigned Windows x64 v0.2.4 with spending/Send maintenance | Initial candidate: all five jobs, fifteen codec/six resource probes and twelve-resource passive verification PASS. Spending/Send delivery receipt for the earlier main installer: [PR #70](https://github.com/comzulca/Aster-desktop/pull/70). The earlier [57339a6 receipt](qa/FINAL-ENGINEERING-V024-20261105.md) remains historical; owner GUI acceptance is separate. |
| Policy and authentication | IPC protocol 6 / financial policy 4; sole Meta Contributor pair, current-key model retrieval proof, native terms review | Source and offline tests PASS. Proof is distinct from billing, eligibility, quota and an actual inference. |
| Historical 5 November policy renewal | Terms 2026-10-09, geography 2026-10-12, reviewed 2026-11-05; local deadline 2026-12-06T00:00:00Z | Legacy terms and financial replay preserved; fresh current review/configuration required. [Primary review](providers/META-POLICY-REVIEW-20261105.md). |
| Attachments | Additive native selection; previous documents/draft retained on cancellation or atomic rejection | Rust/frontend regressions PASS. Aggregate bounds and earliest five-minute custody remain; malformed/uncertain receipts fail closed. |
| Text, images and PDFs | Twelve direct extensions; complete eligible PDF originals and normalised images use the fixed media request contract | Actual packaged executable: fifteen codec cases and six resource probes PASS. No owner/live multimodal certification follows. |
| Protected data | Existing encrypted transcript and separate processed-media domains; originals/grants excluded from restored history | Current-user protection and replay fixtures PASS. OS paging/dumps, filesystem overhead and same-user rollback remain residuals. |
| Money | User-confirmed gross London calendar-month ceiling and two configurable remaining-budget warnings, conservative reservation, unknown-spend blocking | Expiry/range/activation, integer/replay/renewal and Send-prerequisite fixtures PASS. Host-bounded date, visible Settings errors and confirmed active/inactive status; no global invoice cap, fabricated FX, silent refund or reset. |
| Personalization | Separate local-only protected profile and local UI | Source/negative fixtures PASS; profile is excluded from provider/generation/tool payloads. Owner native privacy acceptance remains separate. |
| Skills/Plugins | Bounded inert catalogue/editor/import and isolated Muse-assisted draft generation | No general script, installer hook or arbitrary MCP execution; generation uses a separate native-reviewed ordinary request. |
| GitHub | Built-in dedicated-App authentication, account/repository selection and bounded read routes | Implemented source and synthetic checks; actual Aster live account connection is not inferred from Codex's connection. `mutationsAvailable=false`. |
| Mutation/storage foundations | 34 pure GitHub mutation, 27 capacity, 27 transcript-layout and eight lease cases executed in hosted Rust suite | Test-only foundations. No runtime mutation dispatcher, 4-GiB migration, reliable full-window count or automatic compaction. |
| Startup/rendering | Dark native/WebView setup, guarded reveal and bounded capture-only trace | One earlier owner recording is not repeated acceptance or a causal flicker cure. The 6,000/10,000-ms bounds remain. |

## Source-pinned limits

| Limit | Value / interpretation | Source |
| --- | --- | --- |
| Policy / IPC protocol | 4 / 6 | [policy.rs](../src-tauri/src/policy.rs), [host](../src-tauri/src/lib.rs) |
| User message | 32 KiB UTF-8; composed processed text must also fit admission | [policy.rs](../src-tauri/src/policy.rs), [host](../src-tauri/src/lib.rs) |
| Runtime continuation | 20 completed prior turns / 512 KiB text context | [policy.rs](../src-tauri/src/policy.rs) |
| Text-only serialised request | 512 KiB | [provider.rs](../src-tauri/src/provider.rs) |
| Complete media request | 24 MiB including inline/base64 representation; separate text/source checks still apply | [provider.rs](../src-tauri/src/provider.rs) |
| Completion | 8,192 tokens including reasoning; fixed streaming contract | [provider.rs](../src-tauri/src/provider.rs) |
| Per-process request admission | 32 conversation IDs, 64 request IDs, one active request | [policy.rs](../src-tauri/src/policy.rs) |
| Attachment batch | Four files, 8 MiB source each / 16 MiB total; earliest five-minute deadline | [attachments.rs](../src-tauri/src/attachments.rs) |
| Processed batch | 4 MiB each / 16 MiB total; 64 MiB RGBA; 20 PDF pages total | [attachments.rs](../src-tauri/src/attachments.rs) |
| Processed text | 128 KiB per file / 256 KiB batch parser bounds; smaller composed-message limits still apply | [processing](../src-tauri/src/attachment_processing.rs) |
| Saved transcripts | 128 conversations, 512 KiB per conversation; 64-MiB domain payload | [history.rs](../src-tauri/src/history.rs), [private_store.rs](../src-tauri/src/private_store.rs) |
| Processed-media store | Separate 128-MiB domain payload | [private_store.rs](../src-tauri/src/private_store.rs) |

The model's documented 1,048,576-token context and the owner's proposed 4-GiB transcript preference are not implemented runtime capacity. Payload quotas are not exact disk/ciphertext occupancy. Exceeding current limits denies; it does not silently trim or implement compaction.

## Qualification limits

For v0.2.5 main `a8d6123`, CI recorded 200 frontend tests, 422 default Rust tests and two separately passed disposable credential cases. The packaged app passed 15 codec and six resource probes. The Rust advisory check reported three informational warnings. The native speech distribution falls outside the Cargo and pnpm advisory scans. See the [full results and open obligations](qa/SECURITY-REVIEW-V025-20261123.md).

### Earlier v0.2.4 spending and Send evidence

The initial complete maintenance run reported 184 frontend PASS and 414 default Rust PASS / two ignored. The two explicitly named disposable Credential Manager cases were then executed separately and passed; this is not native credential-entry GUI or the owner's real-key evidence. Strict Clippy passed. RustSec passed with the existing two allowed warnings, not a warning-free or GLib-remediation claim.

A separately authorised local synthetic Meta text stream returned HTTP 200, final stop, DONE and consistent usage (78 prompt / 1021 completion / 1099 total tokens). This is service evidence only: owner-installed startup/visual/native confirmation, live multimodal/GitHub/account/invoice qualification, all 42 compound protocols, signing and production remain independent. The [GLib deferral](qa/GLIB-UPSTREAM-DEFERRAL-20261009.md) remains OPEN. Keep original failures and dated receipts; no result is transferred to changed application source.

[Operating guide](SETUP-AND-USE.md) · [Build procedures](BUILDING.md) · [Earlier v0.2.4 index](V0.2.4-DOCUMENTATION-INDEX.md) · [Earlier reconciliation](qa/DOCUMENTATION-RECONCILIATION-20261105.md)
