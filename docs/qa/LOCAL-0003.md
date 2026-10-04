# LOCAL-0003 — isolated credential diagnostic candidate

**v0.2.3 applicability — 4 October 2026, Edinburgh / Europe/London:** dated evidence below retains its original source, version and result; it does not qualify changed v0.2.3 bytes. Unchanged constraints remain subject to the [current documentation index](../V0.2.3-DOCUMENTATION-INDEX.md) and [SPEC-0005](../specs/SPEC-0005.md). The owner authorises a v0.2.3 PR and GitHub checks only, not merge, release or signing. Final-source, package, native and live qualification require independent evidence in [LOCAL-V023-0001](../qa/LOCAL-V023-0001.md). No retired provider, historical grant or old PASS is reactivated.

**v0.2.2 applicability, 3 October 2026:** retained requirements/contracts apply only where unchanged; retired providers, disabled capabilities and older dated results are not current authority or new-source PASS. [SPEC-0004](../specs/SPEC-0004.md), [ADR-0017](../adr/ADR-0017.md) and [LOCAL-V022-0001](../qa/LOCAL-V022-0001.md) govern active zoom/startup changes and fresh evidence. Earlier evidence remains source-bound history; the later owner-reported v0.2.1 startup flash is recorded separately. The entire original v0.2.1 checkout/installer is unchanged.

Date: 2026-09-30 UTC. Classification: **source candidate / unsigned engineering preparation**, not completed Windows 11 acceptance or production promotion.

## Immutable source and custody

Tested source: `f6f47a9bdf18edfee33bc742aba282542881a78e`, tree `be846dfd30dc5d6b75e6484f7b98adf13de1294c`, based on main `4e4cd6c6208d955782d8cbe8aea2df6aada3d8c7`. Local branch `codex/isolated-credential-gui-gate`. All 74 committed source files independently matched the tested working copy's Git blobs at **2026-09-30T22:10:14.2635473Z**. The 11-file implementation/specification change leaves dependencies, both lockfiles, workflows, the original JPG, the six-command IPC manifest and all four false workspace capability flags unchanged.

The real requested checkout is now a clean authenticated Git clone with origin `https://github.com/comzulca/Aster-desktop.git`. The owner manually completed the guarded renames after normally closing Codex. The clone's 71 baseline blobs/main/tree were verified at 21:41 UTC. At 21:43 UTC only the exact temporary duplicate was removed after no-unique-file/no-reparse checks: 71 files and 1,043,035 logical bytes. All removed bytes remain in the active clone; earlier separately named historical backups were not touched. No permanent replacement backup, safe.directory, ACL or Windows protection change was made.

Installed Git `C:/Program Files/Git/cmd/git.exe`, version 2.56.0.windows.1, verified the HTTPS main ref at 22:02 UTC. The new local source commit uses author and committer `Rene Nehru Alvarez <258826763+renenehru@users.noreply.github.com>`. Commit signing is not claimed. This is the existing owner account with a normal/non-elevated token, not a new account or UAC-elevated build.

## Owner-context offline gates

Microsoft Windows 11 Pro `10.0.26300`, build `26300`; identity `QUARK2\\Rene Nehru Alvarez`, `elevated=False`. Developer Mode was previously enabled by the owner; no further configuration exception was made. Rust/cargo 1.97.0, Node 24.19.0, pnpm 11.19.0, Python 3.12.14. Runtime paths and process-only PATH/RUSTC follow LOCAL-0002; Git uses the installed executable above. `ASTER_DISABLE_LIVE=1` was set in the Rust test process only. No live NVIDIA request was executed.

Rust run: **2026-09-30T22:06:18.2763434Z–2026-09-30T22:06:40.4284964Z** in the reviewed working copy, all of whose source blobs were then bound to the immutable commit above.

| Exact command | Result |
| --- | --- |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | PASS |
| `cargo check --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets` | PASS |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets -- -D warnings` | PASS |
| `cargo test --manifest-path src-tauri/Cargo.toml --locked --offline` | PASS: 80 library tests; 0 failed/ignored/filtered; main/doc targets 0 tests |
| `pnpm typecheck`, `pnpm test`, `pnpm build` | PASS at 21:51:18.4401265Z–21:51:33.8119473Z; 23 tests / 3 files; Vite build only. The unchanged frontend/locked inputs are independently included in the 74-blob source binding; this is not an installer. |
| `python scripts/ci/scan_secrets.py --self-test` | PASS: synthetic positive/placeholder checks |
| `python scripts/ci/scan_secrets.py` | PASS at 22:10:15.0342035Z: 71 Git-visible UTF-8 source files, 0 findings/errors; not proof of absence of every possible secret |

The 21 new pure diagnostic tests cover generated target isolation, exact/unknown/extra/invalid-Unicode argument admission, fake-backend lifecycle and no error cleanup, bounded actual decoder/fake-only admission, stack-backed credential metadata admission, deletion denial and changed/foreign/same-value-rewritten state. These new tests do not call Credential Manager or open native GUI. The full suite additionally runs the existing uniquely named synthetic Windows Credential Manager API, DPAPI/NTFS and loopback transport fixtures; none uses the production credential or a live NVIDIA destination.

## Independent source review and retained intermediate failure

A separate read-only agent reviewed the finalized three runtime files, specifications, threats, ADR and manual proof boundaries. Reported draft issues were repaired: invalid-Unicode startup arguments now fail closed; credential read-back additionally validates bounded exact metadata and non-secret generation snapshots before deletion. Final review found no remaining actionable issue. This is source review, not GUI acceptance or proof of every runtime invariant.

Technical sandbox identity `QUARK2\\CodexSandboxOffline` separately passed the 21 filtered pure tests, fmt and Clippy by **2026-09-30T22:04:54.0746269Z**. Retain a preceding unpublished intermediate **FAIL**: one filtered non-`--lib` test command's library test passed, but Windows Application Control blocked launching the empty main test executable with Windows error **4551**. No protection was disabled, bypassed or reconfigured. Subsequent deliberately filtered pure `--lib` tests passed, and the separate original-owner full run above successfully executed both library and empty main/doc targets. Neither later observation erases the technical sandbox failure or proves that policy changed.

| Reviewed runtime path | SHA-256 |
| --- | --- |
| `src-tauri/src/credential.rs` | `099426f35b052e96bdc22886d87e21b110a058bd89ade4eaa209d3bc5499261b` |
| `src-tauri/src/credential_fixture.rs` | `8a9eb04a1944a20597c0248875222e0091e2bc3df27cad949ab8cf1b4e49da95` |
| `src-tauri/src/lib.rs` | `df9f563f62febc32e361f714b6aab07240b337c8e91222dd42a7906c67ce4baf` |

## Scope/spec/threat review and remaining acceptance

Phase 0 reviews the authenticated baseline/new source inventory, unchanged owner authority, genuine clone, precise temporary cleanup and real-key preservation. Phase 1 reviews FR-013/SEC-010/AC-026 mapped separately to automated T-033 and owner GUI T-034. Phase 2 reviews TM-013/ADR-0005 typed target isolation, raw absence, zeroizing fake-only admission, exact metadata/generation checks, default-cancel consent and no persisted-credential cleanup. The source-bound static reviews are complete. After evidence-only ledger reconciliation, `python "C:/Users/Rene Nehru Alvarez/.agents/skills/aster-builder/scripts/harness_gate.py" check --root . --through-phase 2 --json` passed at **2026-09-30T22:12:41.1974306Z**, returning `{"issues": [], "ok": true}` under the technical sandbox. A separate original-owner non-elevated check in the actual clone returned the same successful result at **2026-09-30T22:14:21.8481425Z** before the evidence-only commit. Its tracked-source secret scan then passed with 72 UTF-8 files, 0 findings/errors, at 22:14:22.2937361Z. This is ledger/source consistency, not runtime, owner GUI or promotion proof; phases 3–9 and T-034 stay NOT RUN.

The diagnostic exits before Host/Tauri/provider startup and cannot use the fixed production store, a renderer, project files or network. Shared native primitives and isolated MessageBox consent do **not** certify the whole production Tauri removal handler. Generic Credential operations have no malicious same-user atomic compare-and-write/delete guarantee. Unsupported GUI metadata/representation fails closed, not silently repaired.

Full exact-head hosted CI/audit/license/SBOM evidence and a **new** unsigned installer are pending. Main run 62 and its verified NotSigned installer bind the preceding LICENSE-only source; they do not contain the new diagnostic and must not be used as its acceptance artifact. Owner installation/launch/UI/restart/uninstall, Control Panel/native GUI journey T-034, secret-boundary capture, accessibility, third-party notices/risk acceptance and both minimal live canaries remain **NOT RUN**. Existing NVIDIA credential must remain unchanged; live calls performed: **0 of 2 authorized**, no project upload/retry/third call.

Issue #12 still lacks normal-app authenticated owner HWND/disclosure/proposal/new-file integration, explicit private journal DACL enforcement, independent competing-process/process-kill/restart proof and safe existing replacement. No VM, host power cut, forced host reboot or new account is authorized. All four shipping workspace flags remain false, existing replacement remains unavailable/proposal-only, and both tracked issues remain open. Use CREDENTIAL-GUI-FIXTURE-0001 only on the reviewed newly packaged diagnostic; it is a manual procedure, not a PASS record.
