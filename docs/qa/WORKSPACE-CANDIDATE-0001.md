# WORKSPACE-CANDIDATE-0001 — protocol-2 project capability acceptance

Historical v0.1.0 evidence/procedure only. Its stable IDs, authorisation history, hashes and PASS receipts are preserved, not transferred to v0.2.0. Current scope is [SPEC-0002](../specs/SPEC-0002.md) and [v0.2.0 verification](../qa/LOCAL-V020-0001.md). Retired providers, ephemeral history and project permissions below do not apply to the current profile.

Reconciled 2026-10-02, Edinburgh (`Europe/London`, BST, UTC+01:00). Related: issue #12, ADR-0003/0004/0006/0009, AC-008–AC-011, AC-022–AC-025, AC-028–AC-030, T-028–T-040. This document defines the unchanged capability boundary and reference procedures. The owner reports **PASS for project/files and engineering scope acceptance** on the delivered build, using the principal administrator account with Aster non-elevated; see [OWNER-ACCEPTANCE-0001.md](OWNER-ACCEPTANCE-0001.md). Detailed observations not supplied by that category-level report are explicitly tracked for [v0.2.0 review](V0.2.0-FOLLOW-UP-0001.md), not reconstructed as individual test results.

## Capability boundary

Protocol 2 reports exactly four host-owned Boolean capabilities:

| Field | Verified engineering-build value | Meaning |
| --- | --- | --- |
| `projectRead` | `true` on supported Windows host | Native root/file selection and guarded host snapshot. Renderer receives relative path, request ID, SHA-256 and byte count only. |
| `proposals` | `true` on supported Windows host | Strict versioned proposal parsing and host-owned native review. |
| `applyNewFile` | `true` on supported Windows host | Exact native approval followed by private-journal `CREATE_NEW` publication. |
| `applyExistingFile` | `false` | Existing-file replacement remains proposal-only. |

An unknown/missing field, incompatible protocol or unsupported platform denies. These bits are not claims that CI, laptop acceptance or a live provider has passed.

No renderer/model approval Boolean, generic path/write endpoint, shell, arbitrary URL, web or MCP tool exists. A project file may be read up to the guarded workspace ceiling, but a project-bearing composed prompt is capped at 32 KiB and is denied rather than truncated. The renderer never receives project plaintext.

## Exact disclosure journey

1. The host obtains a project root and file through native pickers and pins/revalidates root, ancestor, parent and file identities. It rejects traversal, ADS, device/UNC aliases, sensitive names, binary/invalid UTF-8, hardlinks and any reparse component.
2. The host composes the project prompt and serializes the exact fixed-destination provider body before consent. The native review is owned by the normal application HWND and shows the exact bytes to be transmitted, not a summary.
3. Confirmation creates/reserves only a host-private one-use approval. Before dispatch the host revalidates the selected snapshot, consumes approval and sends the reviewed byte sequence once without reserialization. Denial, close, expiry, drift or replay causes zero transmission. Failed/cancelled egress revokes the snapshot; completion retains only its bounded proposal right.
4. `reserve_ephemeral` excludes the project-bearing user message and its derived answer from ordinary continuation history. Snapshot/proposal rights have a five-minute deadline and a 60-second host heartbeat removes expired plaintext/candidates. Re-disclosure requires a new selection/review. Exact transport fixtures do not spend the two live canaries.

T-039 must prove metadata-only IPC, exact-body equality from review through loopback receipt, owner-HWND routing, one-use/expiry/drift denial, zero transmission on denial and absence from subsequent ordinary continuation. These automatic portions pass locally at immutable runtime source `2bcacd7c6f2496eda233d4a069ec7b65a93d47b3` in `LOCAL-0006.md`. Final source/package binding is now PASS at source `1782d3375db47466f89be6bd3cd3c6affbfa95b7`, main CI run `37040168700` attempt 1 (all four checks successful), engineering artifact `11243338033`; see the owner acceptance record for installed identity. The owner reports project/files PASS without an individual T-039 step record. Because no loopback seam ships, the positive installed equality journey remains NOT RUN and is explicitly deferred to v0.2.0 review.

## Proposal and new-file journey

The model response is untrusted text. The host admits only one versioned, size-bounded typed proposal with a closed operation and validated relative path; surrounding prose, unknown fields, approval claims, URLs/tools, multiple operations and stale hashes deny. The host displays the complete target/content/digests in a separate native review and consumes that approval once.

For a new file, the host revalidates root/parent identities, writes a bounded current-user-DPAPI immutable intent under the private owner-only journal, uses Windows `CREATE_NEW`, retains/validates the created identity, writes/flushes/verifies exact bytes and records the outcome. A competing creator is preserved. Ambiguous interruption enters `ATTENTION_REQUIRED`; safe-helper recovery may classify only evidence it can verify and never blindly deletes, overwrites or restores project data.

T-040 must include strict proposal negatives, wrong HWND/caller, replay/expiry/drift, journal privacy and tampering, concurrent creator preservation, and real independent helper-process termination followed by restart/recovery at the defined creation boundaries. Injected errors alone are not process-kill, restart or power-loss evidence. These automatic portions, including the independent helper process, pass locally at immutable runtime source `2bcacd7c6f2496eda233d4a069ec7b65a93d47b3` in `LOCAL-0006.md`; final main-branch CI/package binding is now PASS as identified above. Positive installed owner-HWND review has no separate detailed record beyond the owner's project/files PASS. Physical power loss remains NOT RUN, and no proposal-injection seam ships. These detailed evidence gaps are explicitly deferred to v0.2.0 review, without enabling additional capabilities or authorizing a power cut on the primary laptop.

Existing-file replacement has no activation route. It requires a second native file-picker choice; the host snapshot's relative path and SHA-256 must exactly match the parsed proposal before review. The reviewed replacement must end as proposal-only, consume the review safely and leave the target unchanged. A separate race/recovery-safe design and evidence would be required to change `applyExistingFile` from false.

## Installed Windows 11 acceptance

Use the exact reviewed installer in the owner's existing administrator account, launching the app normally/non-elevated. Keep all remaining Windows/ESET protections unchanged. The owner previously enabled Developer Mode; record it as an owner-made baseline exception, not authority for another security change. Preserve the earlier symlink error 1314 and later successful source-bound fixture as distinct historical results; neither certifies this candidate.

Record for each journey: immutable source/tree, CI run, manifest/ZIP/EXE hashes, installed app hash, Windows/build/NTFS/account/token, UTC, exact step, PASS/FAIL/NOT RUN and redacted evidence. Inspect:

- small and near-limit native review, exact end marker, default-deny/cancel, keyboard, DPI/Narrator, owner HWND, expiry and display failure;
- root/file metadata in the renderer with no selected plaintext, exact outbound disclosure review, mutation during review, denial/close/replay and no-history restart;
- typed proposal and a disposable new-file creation, exact bytes/hash, concurrent creator preservation and conservative recovery observation;
- explicit proposal-only result and unchanged target for an existing-file proposal.

No live project upload is authorized by the two plain-text provider canaries. Do not put private contents, raw request bodies or credentials in evidence. The owner's 2026-10-02 declaration accepts the bounded unsigned v0.1.0 engineering scope and moves everything not covered by the six reported PASS categories to v0.2.0 review. This is the scope disposition for issue #12, not a claim that every reference-procedure row was independently recorded or that local fixtures alone establish installed acceptance. No guest/new account is allowed. Rows requiring absent loopback/proposal-injection seams retain their explicit NOT RUN evidence boundary; existing-file replacement remains disabled. Follow-up obligations and non-production limitations are visible in [V0.2.0-FOLLOW-UP-0001.md](V0.2.0-FOLLOW-UP-0001.md).
