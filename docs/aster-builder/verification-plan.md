# Aster-desktop v0.1.0 — proposed verification procedures

Status: **NOT RUN**; these are planned tests, not evidence. For each actual run record source SHA, exact command/manual steps, UTC time, Windows version when relevant, fixture ID, PASS/FAIL, output path and observed artifact SHA. Keep credentials synthetic. A fake-server test does not prove a live NVIDIA call; a Linux test does not prove Windows native behavior.

| ID | Procedure and expected observation |
| --- | --- |
| T-001 | On Windows 11, build the pinned revision and launch the app. Record OS version, executable hash and startup state; verify key management uses real native APIs. Source-only CI does not pass this test. |
| T-002 | Authenticated GitHub check of `comzulca/Aster-desktop`: private visibility, organization owners/collaborators, current revision and intended branch rules. Record redacted API output; 404 is inconclusive. |
| T-003 | With fake registry, start app; assert Nemotron exact default ID and Muse exact alternative. Submit third ID and assert deny before outbound request. |
| T-004 | Fake NVIDIA Nemotron server: assert exact request shape and sample parameters; drive split UTF-8/SSE `delta.content`, separate `reasoning_content`, empty choices, usage and `[DONE]`; malformed/missing terminator must fail. |
| T-005 | Fake NVIDIA Muse server: assert Muse-specific `stream=true` request shape and documented sampling; drive normal, malformed, truncated and empty-choice SSE. Test non-streamed JSON as an error if the streaming contract was selected. |
| T-006 | Send cancellation during connect and midstream plus duplicate/delayed frames for two conversation IDs; expect exactly one terminal per request and zero post-terminal file actions. |
| T-007 | Render empty, no key, offline, ready, sending, cancelled, authentication failure and recovery states; each control/status must reflect actual host capability, not screenshot text. |
| T-008 | Launch without a root. Use native project-selection dialog to grant a local directory, then rename/delete it; reads must fail and grant must expire, with no startup default root. |
| T-009 | Under an approved root read a small valid UTF-8 text file; verify exact bounded bytes, grant consumption and no network. |
| T-010 | Attempt binary/oversize/invalid UTF-8, symlink/junction/reparse/hardlink and denied secret files; no byte disclosure or traversal. |
| T-011 | Feed typed allowed and malformed/oversize model proposals. Native trusted review must show complete path, old/new hashes and bounded complete diff; reject approval from model or renderer IPC. |
| T-012 | On Windows NTFS use exact one-use approval for a new file. Race a concurrent creator before native `CREATE_NEW`, crash at each journal/write/flush point and resume; never overwrite or report false success. |
| T-013 | On Windows NTFS try competing writers/rename/delete and process termination before/after guarded same-handle write. Verify file identity, old hash, backup integrity, VERIFIED/ATTENTION_REQUIRED and no false atomicity claim. Until PASS, existing-file apply is disabled. |
| T-014 | Exchange messages, close and reopen app; no chat records in app storage and UI shows empty state. Credential still exists only in Windows Credential Manager. |
| T-015 | In Windows native host store, read and remove a disposable synthetic NVIDIA-target Generic Credential, inspect UI IPC/log capture for bytes. Never use a real user key. |
| T-016 | Send forged shell, browser, arbitrary URL, MCP and unregistered tool names through IPC/model text; host rejects without launching process or issuing requests. |
| T-017 | Hash bundled JPG and compare `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887`. Review screenshots at 1917×1137 and 1366×768 for usable composer, status and honest unavailable controls. |
| T-018 | Keyboard-only and screen-reader journey for model selection, key status, send/cancel, project root and proposal review. Focus order, visible focus, names, contrast and streamed status must be understandable. |
| T-019 | Replay the same validated input, policy version, grant state and fake stream twice; compare admission, transition sequence, terminal type and journal hashes. Do not compare model prose or timing. |
| T-020 | Cross each versioned file, diff, body, SSE frame/event and timeout limit by one unit. Assert bounded memory, single terminal, no automatic retry and no unreported success. |
| T-021 | At a fixed source SHA run lint/typecheck/tests, dependency/license/secret/SBOM checks and Windows build; identify exact installer SHA/signature or mark them NOT RUN. Record promotion decision and blockers. |
| T-022 | Register second forged window and send unknown/oversized IPC/renderer injection; no privileged command or credential bytes cross to it. |
| T-023 | Configure fake redirect, wrong DNS/host, proxy environment, wrong model and failed authentication. Assert Authorization header reaches only fixed origin, with zero fallback/retry. |
| T-024 | Try `..`, UNC/device path, ADS, reserved names, differing case, reparse components, hardlinks, `.git`, `.env` and replaced parent/root identity; deny read/write with unchanged outside sentinel. |
| T-025 | Approve exact native diff, then replay nonce, alter target bytes, mutate diff, expire grant, change root or present `approved=true` from renderer. Every stale/replayed operation denies without file change. |
| T-026 | Inject crashes at each journal/backup/flush/write/verify step, restart and inspect identity/hash against journal. Block ambiguous writes with ATTENTION_REQUIRED; never silently restore over unrelated generation. |
| T-027 | Select a nonsecret file excerpt; send another message without opting to include it and assert zero excerpt in request. Opt in for one request, inspect exact sent content, then scan logs/repo/crash captures for key and private text. |

Executable tests should replace or link to these IDs as implementation progresses. Preserve FAIL evidence and distinguish fixture PASS, target Windows PASS, provider canary PASS and signed-artifact PASS as separate observations.
