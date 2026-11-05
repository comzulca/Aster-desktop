# Current engineering status

**Current maintenance:** [PR #70](https://github.com/comzulca/Aster-desktop/pull/70) implements the [spending/Send correction](qa/SEND-READINESS-20261105.md), IPC protocol 5 and unchanged financial policy 4. Its initial completed candidate run is `42109052799`, attempt 1, tested merge `8e92ef76a4e7910ae2193e87e7085c4910b58d1e` from head `c6974cd`. All five jobs and packaged probes passed. The PR delivery receipt separately binds final main source, run and installer hashes.

Reconciled 5 November 2026. The previous delivered software snapshot was [`57339a687125a0b1fd8d6cda787649b5a111fa65`](https://github.com/comzulca/Aster-desktop/commit/57339a687125a0b1fd8d6cda787649b5a111fa65), after [PR #68](https://github.com/comzulca/Aster-desktop/pull/68), with [main CI 42081672227, attempt 1](https://github.com/comzulca/Aster-desktop/actions/runs/42081672227). These identities describe the software artifact; later documentation revisions are not a new installer qualification.

## Capability and evidence matrix

| Area | Implemented behaviour | Evidence / remaining boundary |
| --- | --- | --- |
| Engineering installer | Unsigned Windows x64 v0.2.4 with spending/Send maintenance | Initial candidate: all five jobs, fifteen codec/six resource probes and twelve-resource passive verification PASS. Final main identity: [PR #70 delivery receipt](https://github.com/comzulca/Aster-desktop/pull/70). The earlier [57339a6 receipt](qa/FINAL-ENGINEERING-V024-20261105.md) remains historical; owner GUI acceptance is separate. |
| Policy and authentication | IPC protocol 5 / financial policy 4; sole Meta Contributor pair, current-key model retrieval proof, native terms review | Source and offline tests PASS. Proof is distinct from billing, eligibility, quota and an actual inference. |
| Policy renewal | Terms 2026-10-09, geography 2026-10-12, reviewed 2026-11-05; local deadline 2026-12-06T00:00:00Z | Legacy terms and financial replay preserved; fresh current review/configuration required. [Primary review](providers/META-POLICY-REVIEW-20261105.md). |
| Attachments | Additive native selection; previous documents/draft retained on cancellation or atomic rejection | Rust/frontend regressions PASS. Aggregate bounds and earliest five-minute custody remain; malformed/uncertain receipts fail closed. |
| Text, images and PDFs | Twelve direct extensions; complete eligible PDF originals and normalised images use the fixed media request contract | Actual packaged executable: fifteen codec cases and six resource probes PASS. No owner/live multimodal certification follows. |
| Protected data | Existing encrypted transcript and separate processed-media domains; originals/grants excluded from restored history | Current-user protection and replay fixtures PASS. OS paging/dumps, filesystem overhead and same-user rollback remain residuals. |
| Money | £89 local calendar-month ceiling, £80.10 warning, conservative reservation, unknown-spend blocking | Expiry/range/activation, integer/replay/renewal and Send-prerequisite fixtures PASS. Host-bounded date, visible Settings errors and confirmed active/inactive status; no global invoice cap, fabricated FX, silent refund or reset. |
| Personalization | Separate local-only protected profile and local UI | Source/negative fixtures PASS; profile is excluded from provider/generation/tool payloads. Owner native privacy acceptance remains separate. |
| Skills/Plugins | Bounded inert catalogue/editor/import and isolated Muse-assisted draft generation | No general script, installer hook or arbitrary MCP execution; generation uses a separate native-reviewed ordinary request. |
| GitHub | Built-in dedicated-App authentication, account/repository selection and bounded read routes | Implemented source and synthetic checks; actual Aster live account connection is not inferred from Codex's connection. `mutationsAvailable=false`. |
| Mutation/storage foundations | 34 pure GitHub mutation, 27 capacity, 27 transcript-layout and eight lease cases executed in hosted Rust suite | Test-only foundations. No runtime mutation dispatcher, 4-GiB migration, reliable full-window count or automatic compaction. |
| Startup/rendering | Dark native/WebView setup, guarded reveal and bounded capture-only trace | One earlier owner recording is not repeated acceptance or a causal flicker cure. The 6,000/10,000-ms bounds remain. |

## Source-pinned limits

| Limit | Value / interpretation | Source |
| --- | --- | --- |
| Policy / IPC protocol | 4 / 5 | [policy.rs](../src-tauri/src/policy.rs), [host](../src-tauri/src/lib.rs) |
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

The initial complete maintenance run reported 184 frontend PASS and 414 default Rust PASS / two ignored. The two explicitly named disposable Credential Manager cases were then executed separately and passed; this is not native credential-entry GUI or the owner's real-key evidence. Strict Clippy passed. RustSec passed with the existing two allowed warnings, not a warning-free or GLib-remediation claim.

A separately authorised local synthetic Meta text stream returned HTTP 200, final stop, DONE and consistent usage (78 prompt / 1021 completion / 1099 total tokens). This is service evidence only: owner-installed startup/visual/native confirmation, live multimodal/GitHub/account/invoice qualification, all 42 compound protocols, signing and production remain independent. The [GLib deferral](qa/GLIB-UPSTREAM-DEFERRAL-20261009.md) remains OPEN. Keep original failures and dated receipts; no result is transferred to changed application source.

[Operating guide](SETUP-AND-USE.md) · [Build procedures](BUILDING.md) · [Current index](V0.2.4-DOCUMENTATION-INDEX.md) · [Reconciliation](qa/DOCUMENTATION-RECONCILIATION-20261105.md)
