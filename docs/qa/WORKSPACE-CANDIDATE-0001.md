# WORKSPACE-CANDIDATE-0001 — host-only proof seams

Date: 2026-09-30 UTC. Related: issue #12, ADR-0004, AC-008–AC-011, AC-022–AC-025, T-028–T-032. This document defines remaining manual gates; it is not a completed result record.

## What the candidate does and does not enable

Revision-specific owner-context fixture results are in [LOCAL-0002.md](LOCAL-0002.md), including retained error 1314 and independently successful rerun, and [LOCAL-0003.md](LOCAL-0003.md) for the later PR15 diagnostic runtime. [LOCAL-0004.md](LOCAL-0004.md) tracks the separate changed journal-privacy candidate, which cannot inherit those passes. The manual gates below remain separate.

The candidate contains a closed edit parser, a lossless native read-only review window, request-bound post-review disclosure revalidation, retained root/ancestor pins, and a Windows-only new-file CREATE_NEW/encrypted-metadata-journal proof seam. Existing-file replacement remains proposal-only. All shipping file-read/disclosure, proposal and apply capability flags stay **false**; the six-command IPC allowlist is unchanged. No renderer/model approval Boolean, general write endpoint, shell, arbitrary URL or tool loop is added.

The internal `reserve_ephemeral` proof seam excludes a project-bearing request and its derived answer from continuation history. This is candidate behavior, not an active shipping file-upload route: shipping `send_message` still uses ordinary `reserve`, and host-file upload/disclosure capabilities remain disabled. Disclosure candidates bind the exact serialized outbound bytes, fixed NVIDIA destination, conversation/request, selected root, file identity/hash and nonce. Finalization consumes its dispatch chance and native approval before revalidation; root/file drift, expiry or replay denies. No network integration is activated by this seam.

New-file candidates show the complete host-bound root/absolute target, parent identities, hashes and exact new content. CREATE_NEW never overwrites an existing target. Immutable user-scoped DPAPI records contain bounded metadata, not project text or credentials, with an internal versioned integrity digest. Same-user malicious re-encryption is outside DPAPI's isolation guarantee. Unproven/mismatched recovery is ATTENTION_REQUIRED; there is no automatic delete, restore or cleanup of a target. At most 64 journaled operations fit the bounded scan. A full/ambiguous journal blocks further candidates.

## Run the inert native-view diagnostic manually

Use the exact reviewed engineering candidate under the owner's existing administrator account, launched normally/non-elevated. On 2026-09-30 the owner reported personally enabling **Windows Developer Mode** and requested continuation. This is an explicit owner-made baseline exception, not an agent modification or authority for further changes. Do not create or switch accounts, elevate automatically, alter remaining Windows/ESET protections, change additional policies/trust, or supply a real key. Close any ordinary Aster window first to keep the diagnostic observation separate from the normal application. Identify the installed **application executable**, not the NSIS setup executable.

Preserve the prior non-elevated symlink fixture **FAIL with Windows error 1314** as historical evidence. The owner report alone does not imply a successful rerun or a passed NTFS gate. [LOCAL-0002.md](LOCAL-0002.md) separately records the exact-source 2026-09-30 owner non-elevated rerun: all 59 fixtures passed, including symlink/hardlink/reparse denial. This is internal fixture evidence at its recorded source, not interactive acceptance of a later candidate. A behavior change requires fresh source-bound checks. Keep the Developer Mode baseline field separate from the requirement that remaining protections stay unchanged.

In normal PowerShell, substitute its literal path:

```powershell
$asterApp = Read-Host 'Full path to the installed Aster application EXE'
Get-FileHash -LiteralPath $asterApp -Algorithm SHA256
$asterSmallReview = Start-Process -FilePath $asterApp -ArgumentList '--aster-review-fixture' -Wait -PassThru
$asterSmallReview.ExitCode
$asterLargeReview = Start-Process -FilePath $asterApp -ArgumentList '--aster-review-fixture-large' -Wait -PassThru
$asterLargeReview.ExitCode
```

These flags select synthetic data before normal app initialization. They do not read project files, access credentials, make NVIDIA requests or publish a file. The reviewed text deliberately includes inert markup, mixed newlines, a supplementary-Unicode character and an end marker. The large fixture approaches the 5-MiB review ceiling; oversized content must be denied rather than truncated.

For each size, inspect full scrolling/read-only behavior and the final `END-SYNTHETIC` content marker; use Tab/Shift+Tab, Escape and Enter without moving focus from default Cancel. Repeat only these offline diagnostics to test explicit Confirm and Cancel. Record font/DPI/viewport, Narrator/control names, focus visibility and whether any part becomes inaccessible or visually reordered. Wait five minutes in one offline diagnostic to check expiry denies. Outcome classifications are sanitized: exit 0 = confirmed synthetic review **with no publication**, 2 = denied, 3 = unavailable/expired/display failure. Confirmation here is not approval for a real project or proof of new-file apply.

The primary commands wait for the GUI process and report its returned `ExitCode`; direct GUI invocation can return immediately and leave `$LASTEXITCODE` stale. Show this interactive test window deliberately; do not use it as a background helper.

## Independent activation gates — all initially NOT RUN

| Gate | Required owner/target proof before activation |
| --- | --- |
| Native review | Full small/near-limit content, default-deny/cancel, keyboard, DPI/accessibility, expiry and display/readback failure. Future normal-app use must bind an authenticated owner HWND rather than the diagnostic's null owner. |
| Disclosure | Native review of the exact request including all prior messages; denied request causes zero transmission; root/file mutation during review denies; one-use/expiry/replay; project and derived output never redisclosed without fresh consent. No live project upload is authorized by the two plain-text canaries. |
| NTFS scope | Traversal/ADS/device/UNC/secret names, hardlinks, file/directory reparse points, parent/root aliases and races, outside sentinel unchanged. Retain the prior non-elevated symlink FAIL/error 1314 and the separate exact-source successful fixture rerun in LOCAL-0002.md. Neither the owner's Developer Mode report nor those internal fixtures grants configuration authority or completes normal-app interactive/race acceptance of a later candidate. |
| Journal privacy | Host-selected private journal directory, owner-only intended access with explicit DACL evidence, DPAPI/current-user limitations, ciphertext tampering and foreign/corrupt entries. No project text in records. |
| New-file publication | Concurrent creator preserved; retained parent pins and target sharing; exact content/id/hash; actual process termination at each intent/create/write/flush/complete boundary; restart and conservative ATTENTION_REQUIRED; independently assessed power-loss durability. Injected returns are not process-kill or power-loss proof. |
| Existing replacement | No implementation or activation in this candidate. Separate identity/race/crash-safe design and evidence remain mandatory. |

Record PASS/FAIL/NOT RUN, exact source SHA, app/installer hashes, Windows build, existing account/non-elevated token, UTC, commands/steps, fixture IDs and redacted evidence for each gate. Record the owner-made Developer Mode exception separately from observed remaining protection states; do not assert configuration verification from the owner's report alone. Preserve historical failures and append new results without rewriting them. A subsequent activation change requires updated specifications/threat review, authenticated normal-app integration, independently passed gates and full exact-revision CI. Do not close issue #12 just because these internals compile or its fixtures pass.
