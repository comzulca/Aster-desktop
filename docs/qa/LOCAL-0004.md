# LOCAL-0004 — dormant journal privacy candidate verification

Started: 2026-09-30 UTC; continued: 2026-10-01 UTC. Status: **PASS for the reviewed repaired original-owner offline fixture suite; immutable source binding pending**. The initial native FAIL remains separately recorded below. These observations cannot transfer to changed runtime bytes. Trace: ADR-0006 / FR-014 / SEC-011 / TM-014 / AC-027 / T-035. No capability activation or release promotion is claimed.

## Source and authority

Authenticated baseline main `50837e157e009b5d1135fbe8fa18d29e9fe2975a`, tree `1799e08bb3313a8417ad78d2d692325e3d9407d5`, is merged PR15. The requested Git clone is clean on this baseline. Its required CI run63 passed 80 Rust tests, but neither that CI nor main run64 certifies the later private-journal changes. The candidate is separate and **UNPINNED** until final review, original-owner offline tests and immutable publication/source-byte binding.

Only `journal_privacy.rs`, `new_file.rs`, internal journal callers/fixtures in `workspace.rs` and one Windows-only module declaration in `lib.rs` change runtime behavior. No dependency/lockfile, provider, credential diagnostic, renderer, IPC permission, workflow or shipping capability changes are intended. All four workspace flags remain false. The existing real NVIDIA credential remains preserved; no agent real-key read/export/log, GUI or live request occurs here. Owner native fixture execution, when reviewed, uses only uniquely generated disposable synthetic objects under the existing owner account with a normal/non-elevated token. No account, privilege, unrelated ACL, parent/project/repository permission or global Windows security policy changes are allowed.

## Retained intermediate observations

- Initial technical offline Clippy returned **FAIL** for a needless borrow in `new_file.rs:321` after the journal directory became `&Path` (raw output reference `0aea69`, intermediate unpinned source). The corrected command returned PASS (`125ce9`). This is not final native evidence; do not erase the failure or fabricate an immutable revision for it.
- The first source-ready snapshot at `2026-09-30T22:52:16.7376266Z` had SHA256 `526552ed43c7f94931a10b2ee35044bda4ad0d0fb743b951f31553dfc4ab35a9` for `journal_privacy.rs`, `3c74721fb433304875ff9f46fb226020229d2f3cab75d93b4e9cc72667f8b843` for `new_file.rs`, `8342ccc65eed220ea970b8b59014ac5d69b5be04cdaed57a7911d34210fd2816` for `workspace.rs` and `617686b02fe8e7085d8a57994177e60f85648f06d77194ecbc406e8b4654f82d` for `lib.rs`. Technical `QUARK2\CodexSandboxOffline` fmt/check/Clippy and four pure decoder tests passed, with 85 other tests filtered. **Zero native privacy/credential/GUI fixtures were executed in that technical run.**
- Independent static review rejected that snapshot before owner native execution: directory-only revalidation missed already-written record privacy drift at the Intent/IdentityRecorded sampled publication boundaries. The repair requires bounded existing-record admission before subsequent publication and final recovery success, plus record-only negative fixtures. This is a source review finding, not an executed native test result. The same-user non-CAS residual remains explicit.
- A frontend command invocation failed at `2026-09-30T22:54:56.0074002Z` because the summarized pnpm path omitted `bin` (output `0d204b`). No frontend test launched in that attempt. The installed runtime path was inspected, not installed or reconfigured.
- Correct bundled pnpm execution under `QUARK2\Rene Nehru Alvarez`, non-elevated, ASTER_DISABLE_LIVE=1, from `2026-09-30T22:55:43.1921701Z` through `2026-09-30T22:55:54.6982396Z`: `pnpm typecheck`, `pnpm test` and `pnpm build` **PASS**; 23 tests across three files passed. Frontend and locked inputs are unchanged, but final source binding remains pending.

## Original-owner first native run — retained FAIL

The repaired static-review snapshot was independently reviewed without further actionable findings, with exact runtime SHA256 values `c4b085eaeb6f4f9bf9841960ee67454a16a8c2d1e551d477b6c04e33a2c7c0e8` (`journal_privacy.rs`), `147961f9cfe71014b278618b30f3fabf98e65eaeae647ad2d1335600662f1c3b` (`new_file.rs`), `18e519133cfe3d064a43f6aa9fbee8b9b685bdd82960b8de8e38af74511bc6c8` (`workspace.rs`) and `617686b02fe8e7085d8a57994177e60f85648f06d77194ecbc406e8b4654f82d` (`lib.rs`). These hashes were checked before execution and again after the failure; no immutable Git source revision is asserted for that intermediate snapshot.

Original owner `QUARK2\Rene Nehru Alvarez`, normal/non-elevated token, Windows 11 Pro `10.0.26300` build `26300`, Rust 1.97.0, ASTER_DISABLE_LIVE=1, locked/offline inputs:

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo check --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets`, `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets -- -D warnings`: **PASS**, `2026-10-01T09:35:56.9786188Z`–`2026-10-01T09:36:01.1582352Z` (raw output `b63da7`). Native tests were NOT RUN in this command.
- `cargo test --manifest-path src-tauri/Cargo.toml --locked --offline`: **FAIL**, started `2026-10-01T09:36:38.9684645Z`, completed and source hashes rechecked by `2026-10-01T09:37:23Z` (raw outputs `fd64a5`, `897d60`, `5d77c3`). **89 passed, 1 failed, 0 ignored, 0 filtered** out of 90 Rust library tests. Failure: `workspace::tests::journal_privacy_rejects_weak_existing_objects_and_preserves_creation_collisions`, `workspace.rs:2483`, reading the synthetic weak-journal sentinel returned Windows error 5 / PermissionDenied. Main/doc test execution did not occur after the failed library suite. The other five native privacy tests and all four pure decoder tests passed in this run, but the aggregate privacy/full-suite gate is not credited.

The failure is being diagnosed as test-fixture setup/ACL propagation; no production-policy relaxation or Windows security change is authorized to suppress it. A reviewed fixture correction and fresh complete exact-source owner run are required. No real credential, GUI, external/provider request, account change or global security policy change occurred. Native ACL mutations were confined to freshly generated synthetic fixture objects. Preserve this failed record even after a later successful rerun.

## Test-only setup repair — native rerun pending

Primary [Microsoft inheritance contract](https://learn.microsoft.com/en-us/windows/win32/secauthz/automatic-propagation-of-inheritable-aces), inspected 2026-10-01 UTC, states that removing inheritable parent ACEs removes inherited copies from children and can leave an empty deny-all child DACL. This supports the fixture diagnosis: its sentinel was initially created with inherited permissions before the fixture's own directory-ACL mutation. No production admission/collision helper was observed changing it.

The scoped repair creates that synthetic sentinel **after** deliberate weak-parent setup, using the existing CREATE_NEW helper with an explicit protected actual-owner record descriptor; it writes/flushes/closes the new object and establishes its baseline before exercising production rejection/collision. The test now compares both directory/sentinel descriptor bytes, identities and sentinel content unchanged after the denied calls. The private-directory collision also checks unchanged identity. No production-prefix or other runtime-file changes were made; only the cfg(test) test in `workspace.rs` changes, new SHA256 `c536113a228d85eefb08a7c8ff092156829aedebf0f0b8cd89df1c2e5ab9e291`. The prior failed fixture was not repaired or deleted by the repairing agent.

Retain the technical formatting **FAIL** at `2026-10-01T09:41:38.9800816Z` (output `e9214b`) before a scoped one-line format correction. Technical `QUARK2\CodexSandboxOffline` fmt/check/Clippy and four filtered pure decoder tests subsequently **PASS** from `2026-10-01T09:42:02.1309815Z` through `2026-10-01T09:42:16.5572026Z` (outputs `543f80`, `25d5a5`), with 86 other tests filtered and no native/ACL/credential/GUI/network fixture execution. These passes are not an original-owner complete native-suite result.

## Remaining evidence after the failed run

This historical checkpoint's NOT RUN statuses are superseded only by the separate exact-byte observations below. The earlier failure is retained unchanged.

## Reviewed repaired original-owner run — PASS

After a read-only independent review found no remaining actionable fixture repair defect, the root ran the following commands in the original owner account `QUARK2\Rene Nehru Alvarez`, normal/non-elevated token, Windows 11 Pro `10.0.26300` build `26300`, Rust 1.97.0, ASTER_DISABLE_LIVE=1 and locked/offline inputs:

```text
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline
```

All four commands **PASS**, `2026-10-01T09:46:54.1151259Z`–`2026-10-01T09:47:05.1365348Z` (raw outputs `39c4ca`, `862790`). Rust library suite: **90 passed, 0 failed, 0 ignored, 0 filtered**. Empty main and doc test binaries also completed successfully. Final runtime hashes were guarded before and after execution: journal privacy `c4b085eaeb6f4f9bf9841960ee67454a16a8c2d1e551d477b6c04e33a2c7c0e8`; new-file journal `147961f9cfe71014b278618b30f3fabf98e65eaeae647ad2d1335600662f1c3b`; workspace `c536113a228d85eefb08a7c8ff092156829aedebf0f0b8cd89df1c2e5ab9e291`; lib `617686b02fe8e7085d8a57994177e60f85648f06d77194ecbc406e8b4654f82d`. The fixture correction changed no production prefix or other runtime file.

The new four pure decoder tests and all six native privacy tests passed: weak/colliding objects unchanged; path/identity mismatch and retained lifetime pins; directory drift before intent with zero publication; record drift blocks read/recovery/follow-up; directory drift at four sampled stages preserves ambiguity/bytes/descriptors; intent-record-only drift denies target creation at Intent and content writes at IdentityRecorded, with no complete record, preserved existing ciphertext/descriptor and denied recovery/actual follow-up. These are bounded in-process sampled fixtures, not external process termination, power-loss, human consent or an atomic-CAS proof.

At `2026-10-01T09:46:11.6472732Z`, an original-owner read-only inventory of the generated `aster-workspace-test-*` namespace in the two eligible fixture bases found zero residual directories created during the failed run's `09:36:38Z`–`09:37:23Z` window. No cleanup, repair, unrelated deletion or ACL change was performed by that inventory. A prior technical sandbox OS CIM read was AccessDenied; the later owner read independently observed the Windows build above. Do not transfer one execution identity's result to the other.

## Current independent gates

Fresh original-owner frontend check, ASTER_DISABLE_LIVE=1, bundled Node/pnpm and unchanged locked frontend inputs: `pnpm typecheck`, `pnpm test`, `pnpm build` **PASS**, `2026-10-01T09:48:40.4319045Z`–`2026-10-01T09:48:57.1320943Z` (outputs `d02381`, `9a4aa5`), **23 tests across three files**. This is separate from the prior day's frontend observation and the preserved failed tool invocation.

Current source-byte/secret/ledger binding, hosted required CI/audit/SBOM and installer verification are pending for this later candidate. T-035 can clear only the privacy subsection once its immutable byte binding is established; aggregate T-031, normal-app owner-HWND/native consent/disclosure/proposal/new-file integration, independent-process competition/termination/restart, existing-file replacement and power-loss remain separate. No disposable VM is available and no physical-host power cut or reboot is authorized. Issues 11 and 12 remain open; manual Windows 11 GUI/secret-boundary/no-history/accessibility acceptance and the two separately authorized live canaries remain pending. All four shipping workspace flags remain false, with zero live NVIDIA requests and the existing real credential preserved.
