# FRONTEND-V023-0001 — dirty-checkpoint renderer evidence

Status on 4 October 2026: **implemented engineering draft; fresh final-source/native/installed qualification pending**. Renderer and host source controls exist; the results below are scoped dirty-checkpoint receipts, not proof of complete native implementation, execution or release. Code Integrity currently blocks the local build helper, and the two disclosed cargo-audit warnings remain in [LOCAL-V023-0001](LOCAL-V023-0001.md). No full Cargo/native PDF/installed/Meta PASS or flicker cure is implied.

Date: 4 October 2026, Edinburgh / Europe/London. Scope: renderer/type/build portions on the isolated development working tree, **not final clean-source qualification, native processing/viewer, installed acceptance or release evidence**. Root LOCAL-V023-0001 and the active control ledger retain the final gate disposition. Any later relevant source change invalidates dependent portions of this checkpoint.

## Latest dirty checkpoint — 10:52 UTC, 100 tests

Receipt `reports/frontend-20261004T105233523Z/receipt.json` records `2026-10-04T10:52:33.6564404Z`–`10:52:47.7447242Z`, principal `QUARK2\CodexSandboxOffline`, Windows NT `10.0.26300.0`, against predecessor HEAD `7f8de9c1db74de92672976eb5af3ee9f54e68fef` / tree `846bb82a537240f25b8b859108ac418f8a6ae00c` plus explicit uncommitted v0.2.3 source. Canonical runner remains `scripts/qa/Run-LocalEngineeringChecks.ps1 -Stage frontend` with the configured bundled runtime. No clean final revision is asserted.

| Command | Actual result | UTC interval / local log |
| --- | --- | --- |
| `pnpm typecheck` | PASS, exit 0 | `10:52:33.6631902Z`–`10:52:34.5634192Z`; `typecheck.txt` |
| `pnpm test` | PASS, exit 0; **100 tests / eight files** | `10:52:34.5668373Z`–`10:52:46.4777826Z`; `vitest.txt` |
| `pnpm build` | PASS, exit 0 | `10:52:46.4781673Z`–`10:52:47.7379371Z`; `vite-build.txt` |

The rerun includes help assertions for the additional 16-MiB combined inline native-PDF/normalised-image ceiling, 24-MiB serialised request and 16-MiB conversation processed-content ceiling including inert text/PNG pages. Help qualifies the bounded ten-second native reveal attempt by OS/UI-loop availability, not guaranteed visibility/paint. Existing case count remains 100; prior receipts below remain historical. Native cancellation/source changes and further documentation are independent; this checkpoint cannot qualify their full Cargo/native execution or final source. No app/setup, private file/store, key, Meta request or protection change occurred.

## Retained earlier dirty checkpoint — 10:26 UTC, 100 tests

After integration clarified the exact native batch replacement/error/viewer contracts, the canonical frontend runner passed all three commands again. Receipt `reports/frontend-20261004T102607704Z/receipt.json`, `2026-10-04T10:26:07.8035909Z`–`10:26:22.3931193Z`; principal `QUARK2\CodexSandboxOffline`, Windows NT `10.0.26300.0`, Node 24.19.0/pnpm 11.19.0. The recorded HEAD/tree remain predecessor `7f8de9c1db74de92672976eb5af3ee9f54e68fef` / `846bb82a537240f25b8b859108ac418f8a6ae00c` plus explicit uncommitted v0.2.3 changes, **not** final clean source.

| Command | Actual result | UTC interval / local log |
| --- | --- | --- |
| `pnpm typecheck` | PASS, exit 0 | `10:26:07.8088969Z`–`10:26:08.5367038Z`; `typecheck.txt` |
| `pnpm test` | PASS, exit 0; **100 tests / eight files** | `10:26:08.5396449Z`–`10:26:21.2825770Z`; `vitest.txt` |
| `pnpm build` | PASS, exit 0 | `10:26:21.2833643Z`–`10:26:22.3859204Z`; `vite-build.txt` |

The host replaces a complete temporary batch before its native selector opens; pending UI drops prior identifiers/classification, cancellation returns an empty batch and cannot restore originals. Drafts remain exact and nothing sends automatically. New fixtures map `attachment_capacity`, `attachment_unavailable` and `attachment_convert_to_pdf` to safe British-English guidance. Processed text metadata/content remains in the saved user message; no text page-view button or text native-view intent is offered. Only image/PDF derivatives have local page-view controls. These new fixtures supersede affected portions of the earlier 95-test checkpoint below, not its historical result.

An ad-hoc `pnpm exec vitest run src/composer.test.tsx` invocation failed before Vitest started because that minimal-shell command could not resolve `vitest`; its preceding typecheck exited 0. Retain that command-resolution failure separately: it is not a test assertion failure, native app block or evidence that a test ran. The canonical configured runner above subsequently executes the full suite successfully. No app/setup, live call, private user file/store or protection setting was touched. Final clean-source and native/package qualification remains required.

## Retained earlier dirty checkpoint — 95 tests

The receipt starts at predecessor HEAD `7f8de9c1db74de92672976eb5af3ee9f54e68fef`, tree `846bb82a537240f25b8b859108ac418f8a6ae00c`, **plus explicit uncommitted v0.2.3 runtime changes**. This HEAD is not falsely identified as the tested final v0.2.3 source. Final qualification requires an independently recorded clean source commit and rerun; no binary hash is claimed.

Actual principal: `QUARK2\CodexSandboxOffline`; OS: Microsoft Windows NT `10.0.26300.0`; bundled Node 24.19.0/pnpm 11.19.0. Receipt `reports/frontend-20261004T100521036Z/receipt.json`; interval `2026-10-04T10:05:21.1522584Z`–`2026-10-04T10:05:36.1951310Z`. The read-only/native-startup source assertions and React fake-host tests do not represent the owner's interactive administrator-session observations.

Invocation: PowerShell 7 `scripts/qa/Run-LocalEngineeringChecks.ps1 -Stage frontend` with the configured bundled RuntimeRoot, RustBin and CargoCache. The existing runner records executable/arguments and per-command UTC/logs. No normal app or installer was executed; no real key, Meta call, owner history/spending mutation or security-setting change occurred.

| Command | Result | UTC interval / local log |
| --- | --- | --- |
| `pnpm typecheck` | PASS, exit 0 | `10:05:21.1575025Z`–`10:05:21.9661423Z`; `typecheck.txt` |
| `pnpm test` | PASS, exit 0; **95 tests / eight files** | `10:05:21.9696698Z`–`10:05:34.9810305Z`; `vitest.txt` |
| `pnpm build` | PASS, exit 0; TypeScript plus production Vite build | `10:05:34.9815431Z`–`10:05:36.1878423Z`; `vite-build.txt` |

An earlier dirty checkpoint at `reports/frontend-20261004T095648807Z` passed typecheck, 88 tests/eight files and build before processed-history rendering/strict refs were added. That earlier result is narrower, not final or cumulative evidence for subsequent changes. Preserve it without transferring PASS.

## New renderer portions covered

- Uniform-surface plus control before the fixed model label; genuine nested Classify content after Extra high; independent default Medium and initially unset eligibility; keyboard/Home/End/arrows/Escape/outside-click/focus and modal/busy behaviour.
- Native absent/unqualified capabilities remain unavailable; complete host batch metadata is validated instead of renderer-created merges; unknown path/bytes/approval fields, unsafe labels, IDs, excessive counts/pages/bytes and incompatible protocol are denied.
- Draft preservation; per-chat temporary IDs; removal/switch/restart behaviour; no implicit send, financial/credential configuration, restored import ID or restored eligibility.
- Only user entries can carry optional processed metadata; legacy text entries remain admitted; assistant refs, original/path/base64/grant/approval injection are denied.
- Local processed text/page view uses only conversation/request/attachment IDs and a strict `{viewed: boolean}` receipt. It sends nothing, restores no original and is never copied into fresh attachmentIds. Malformed approval-bearing receipts fail closed.
- One-shot shell scheduling after settled branding assets/two animation frames, independent of status hydration, including StrictMode and cleanup. Source asserts fixed six/ten-second bounds, dark pre-module surfaces, unchanged CSP, process-local colour and owned UI-thread scoped CBT activation veto; no generic renderer window/input hook is granted.

These tests are renderer/pure source seams, not filesystem identity, signature/structure validation, actual WinRT PDF render, Job/token enforcement, all-page visual review, encrypted processed-media append/recovery, exact typed Meta JSON-body compatibility or native focus/pixel evidence. Animation frames can be throttled while hidden, so the deadline may be the presentation path. Observe installed startup for at least 20 seconds **after visibility**; neither frames nor a timing assertion proves cure. Compound T-075..083 remain NOT RUN until all required source/native/package portions have exact evidence.
