# LOCAL-0002 — owner-context candidate verification

> Current-source applicability — 8 October 2026, P9 preparation: the [dated reconciliation](OCT8-RECONCILIATION-20261008.md) supersedes current-status claims below. PR #38 and #31–#34 are normally merged after four source-bound gates; PR #35–#37 and PR #30 are not complete. P8's native Windows access violation remains FAIL, its installer SKIPPED. The [P9 contract](PDF-CRASH-DIAGNOSTIC-20261008.md) admits only test-only diagnosis after the unchanged complete suite fails; 112 counted pure checks and static review are not native qualification or an installer PASS. The dependency auxiliary test is an isolation lease test, not native credential execution. Original dated results, owner v0.2.2 flicker/Code Integrity FAIL, credential custody and manual/live/production obligations remain unchanged.

**Retained dated v0.2.3 applicability — 4 October 2026, Edinburgh / Europe/London:** dated evidence below retains its original source, version and result; it does not qualify changed v0.2.3 bytes. Unchanged constraints remain subject to the [current documentation index](../V0.2.3-DOCUMENTATION-INDEX.md) and [SPEC-0005](../specs/SPEC-0005.md). The owner authorises a v0.2.3 PR and GitHub checks only, not merge, release or signing. Final-source, package, native and live qualification require independent evidence in [LOCAL-V023-0001](../qa/LOCAL-V023-0001.md). No retired provider, historical grant or old PASS is reactivated.

**Current-status navigation — 8 October 2026:** see [the reconciliation record](../qa/OCT8-RECONCILIATION-20261008.md) for actual PR/dependency/source/gate outcomes and the later owner-authorised conditional integration. This dated paragraph and the original evidence below are preserved history, not current authority, a new execution receipt or a reactivation of retired providers. Manual/native/live/production results do not transfer between sources.


**v0.2.2 applicability, 3 October 2026:** retained requirements/contracts apply only where unchanged; retired providers, disabled capabilities and older dated results are not current authority or new-source PASS. [SPEC-0004](../specs/SPEC-0004.md), [ADR-0017](../adr/ADR-0017.md) and [LOCAL-V022-0001](../qa/LOCAL-V022-0001.md) govern active zoom/startup changes and fresh evidence. Earlier evidence remains source-bound history; the later owner-reported v0.2.1 startup flash is recorded separately. The entire original v0.2.1 checkout/installer is unchanged.

Date: 2026-09-30 UTC. Classification: **source/unsigned-engineering candidate preparation**, not a production release or completed Windows 11 manual acceptance.

## Exact source and environment

Tested source: [`198db2111907d7ea548ccb365f4d0b42c80ea50a`](https://github.com/comzulca/Aster-desktop/commit/198db2111907d7ea548ccb365f4d0b42c80ea50a), tree `e5812b390d0f460a7203045da3f7ba7d1dc52220`, based on `31a82bf50fc46ea4fbb91c50c59cc02a65da4c71`. All 19 changed UTF-8 blobs were independently matched to local Git-blob hashes after upload; the authenticated recursive tree contains exactly 69 source files, with no missing baseline file. Dependencies, lockfiles, CI workflows, original JPG and IPC permission file are unchanged. The commit author/committer is Rene Alvarez, `258826763+renenehru@users.noreply.github.com`; the source commit itself is unsigned (no commit-signature claim).

Final offline run: **2026-09-30T15:57:31.7563233Z–2026-09-30T15:57:58.8576264Z**. Microsoft Windows 11 Pro `10.0.26300`, build `26300`; existing owner account `QUARK2\\Rene Nehru Alvarez`, `elevated_token=False`. The runner explicitly checked that identity and denied any other/elevated token. Rust/cargo 1.97.0, Node 24.19.0, pnpm 11.19.0, Python 3.12.14. Working source is a verified snapshot, not a Git clone. Temporary fixture roots and credential targets are uniquely generated, synthetic and cleaned by their fixture owners; real credentials and project files are not used.

The owner reported personally enabling Developer Mode, and the run read `AllowDevelopmentWithoutDevLicense=1`. This is an explicit owner-made baseline exception, not an agent change. No new/guest account, security exclusion, certificate, trust-store change, automatic elevation or further Windows protection change was made. Actual protection/ESET UI inspection remains manual NOT RUN. `ASTER_DISABLE_LIVE=1` was set only in the test process. Loopback fake servers are fixture traffic, not live NVIDIA requests.

## Commands and results

Working directory: `C:/Users/Rene Nehru Alvarez/Documents/ChatGPT/Verificaciones Github/issues-11-12-work`, the reviewed source snapshot. The commands below used absolute pinned executables: cargo/rustc in `C:/Users/Rene Nehru Alvarez/.rustup/toolchains/1.97.0-x86_64-pc-windows-msvc/bin`, pnpm in `C:/Users/Rene Nehru Alvarez/.cache/codex-runtimes/codex-primary-runtime/dependencies/bin/fallback/pnpm.cmd`, and Python in that runtime's `dependencies/python/python.exe`. `PATH` included the pinned Rust/Git directories and `RUSTC` named the pinned rustc.exe for this process only. No user/system environment setting was modified.

| Command | Result |
| --- | --- |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS |
| `cargo check --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets` | PASS |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets -- -D warnings` | PASS |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked --offline` | PASS: 59 library fixtures; 0 failed/ignored/filtered; main/doc test targets 0 tests |
| `pnpm typecheck` | PASS |
| `pnpm test` | PASS: 23 tests in 3 files |
| `pnpm build` | PASS: TypeScript and Vite renderer build, not an installer |
| `python scripts/ci/scan_secrets.py --self-test` | PASS: synthetic positive/placeholder checks |
| `python scripts/ci/scan_secrets.py` | PASS: local bootstrap scan, 76 UTF-8 files, 0 findings/errors; not proof of absence of every secret |
| `python "C:/Users/Rene Nehru Alvarez/.agents/skills/aster-builder/scripts/harness_gate.py" check --root . --through-phase 2 --json` | PASS: `{"issues": [], "ok": true}`; ledger structure only, not runtime activation |

The tool-output references are command start `fd76be` and completion `91a1bc`; source-hash verification is independent of those display identifiers. The native symlink/hardlink/reparse fixture passed on this non-elevated owner-context run. Synthetic credential APIs, DPAPI, parser, exact-review encoding/bounds, same-handle new-file verification, injected interruption states, conservative recovery, competing-thread creator/writer denial, grant revocation/replay/expiry, post-review disclosure freshness and ephemeral-history tests all passed. T-029/T-032 are credited only as these internal fixtures; they do not enable file IPC.

After creating the immutable tested-source commit, the evidence-only ledger was reconciled to that source and checked again under the same non-elevated owner account at **2026-09-30T16:09:16.2003399Z–2026-09-30T16:09:16.2965781Z**: PASS, `{"issues": [], "ok": true}`, exit 0. This validates current declarations and references, not native consent, provider availability or release readiness. The preceding source commit retains its historical ledger; this follow-up evidence commit does not change runtime bytes.

## Retained prior failure and rerun

At **2026-09-30T15:32:00.3541035Z–15:32:13.4360010Z**, the prior unpublished intermediate source ran 58 Rust fixtures under the same existing non-elevated owner account: **57 PASS, 1 FAIL, 0 ignored**. `workspace::tests::hardlinks_and_reparse_points_are_denied` failed while creating its synthetic symlink with Windows error **1314**, required privilege not held. That intermediate `workspace.rs` has SHA-256 `77395f14446253ee4b5dbe1f7d3d6331bd5416c5da007e6fcd8fe33c3838dce8` (Git blob `dd84c3659500954df60950d5b24f973ef1969cf2`); it has no published commit acceptance claim. It is not represented as a PASS or as the final tested revision.

After the owner enabled Developer Mode, a separate intermediate rerun at **15:42:16.5009328Z–15:42:22.0022344Z** passed all 58 fixtures, with the owner identity/non-elevated token/Developer Mode readback explicitly checked. The final revision added cross-operation candidate revocation and its 59th fixture, then reran the complete commands above. Neither rerun erases the earlier failure or proves interactive native review.

## Immutable runtime bytes tested

| Path | SHA-256 |
| --- | --- |
| `src-tauri/src/credential.rs` | `0223cb6373318993e471a2d0bf922eb7ddf61116868793aa10f4ff90ebca47be` |
| `src-tauri/src/lib.rs` | `02862898dd8aae3aec4e1fe7534ae33901dc7238b2a31821bf5457f63ad0aca9` |
| `src-tauri/src/native_review.rs` | `1476a0a2afc55fc429080f9c8219c1ae3763f2ee65c6b94ac567a8c1ea4cd518` |
| `src-tauri/src/new_file.rs` | `d428f347f68610d7b4e7d3c2cebc8945e0728fab410bba458831a2c00829fea1` |
| `src-tauri/src/policy.rs` | `bb20ba9e597c9e4984ea03a516aa269adf52174944a98598be30cf3fd7b25f3c` |
| `src-tauri/src/workspace.rs` | `1b79b0f28136c427882954604803c2e9956943de7ef2e8422c41673389b87cfd` |
| `src/App.test.tsx` | `71a97447b0b76453fd4fdcad0d0b0832174cc093d282c12eae251b6a3d04f418` |
| `src/App.tsx` | `3fa793380b1976fb47b9084955d67e3f1c92cfb44a8cb432aaa3a130ec5cb033` |

Unchanged lockfiles: pnpm `9fd9a0695646d0c19bb6b3e64f99fc50b1d41616f13eabf3aa7b7fd508189d18`; Cargo `5937e08df5ab91373f6e53a45723fcbad5a83ccc66c5d8d0678ef0909fcdd2d0`. Original JPG remains `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887`.

## Static scope/design review and remaining gates

Authenticated repository GETs on 2026-09-30 confirmed private `comzulca/Aster-desktop`, default `main`, connector role with admin/maintain/push/pull access, and active ruleset `23958546`, **Aster main CI gates**. It requires PRs and strict source-policy/frontend/windows-host checks, denies deletion/non-fast-forward updates and has no bypass actor. No ruleset/permission change was made. This is not an exhaustive sole-owner or inherited-organization-access audit.

Phase 0 inventories the exact baseline/candidate and authority (unsigned engineering only, no new accounts, owner-entered Credential Manager key, exactly two authorized live requests). Phase 1 reviews the unchanged stable requirement IDs and new T-028–T-032 procedures, one combined candidate per conversation/global four, and disabled conditional scope. Phase 2 reviews the source-bound spec/threat/ADR chain for secret custody, native consent, exact egress, NTFS identity, metadata journal and conservative recovery. Structural validation only checks the ledger declarations; it does not prove these runtime invariants.

Full exact-head GitHub CI/advisory/license/SBOM evidence and a newly built installer are pending this source's PR/main run. The old main installer cannot certify these changes. All four shipping workspace flags remain false and the six-command IPC allowlist is unchanged. Existing-file apply always remains unavailable/proposal-only. No NativeApproval fixture is human consent.

Interactive native small/near-limit review, DPI/Narrator/focus, authenticated normal-app owner HWND, manual Credential Manager GUI interoperability/secret-boundary capture, installation/launch/uninstall and no-history restart remain **NOT RUN**. Journal DACL/privacy, independent competing-process tests, actual process-kill/restart and power-loss durability remain **NOT RUN**; injected stage returns do not pass them. Actual project disclosure integration is disabled. Live NVIDIA calls remain **NOT RUN**: exactly two plain-text canaries are authorized, no project upload, retry, third call or additional live cancel/error case. Current payload/license/notice/owner exceptions and signed/public production promotion are not claimed.

Use [WINDOWS11-ACCEPTANCE-0001.md](WINDOWS11-ACCEPTANCE-0001.md) and [WORKSPACE-CANDIDATE-0001.md](WORKSPACE-CANDIDATE-0001.md) for owner manual evidence. Issues #11/#12 remain open until their independently required evidence is complete. A later evidence-only commit may refer to this earlier tested source without changing its runtime bytes; any behavior change requires fresh validation.

