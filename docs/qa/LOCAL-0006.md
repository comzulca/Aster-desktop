# LOCAL-0006 — final lock-corrected exact-source validation

> Current-source applicability — 8 October 2026, P9 preparation: the [dated reconciliation](OCT8-RECONCILIATION-20261008.md) supersedes current-status claims below. PR #38 and #31–#34 are normally merged after four source-bound gates; PR #35–#37 and PR #30 are not complete. P8's native Windows access violation remains FAIL, its installer SKIPPED. The [P9 contract](PDF-CRASH-DIAGNOSTIC-20261008.md) admits only test-only diagnosis after the unchanged complete suite fails; 112 counted pure checks and static review are not native qualification or an installer PASS. The dependency auxiliary test is an isolation lease test, not native credential execution. Original dated results, owner v0.2.2 flicker/Code Integrity FAIL, credential custody and manual/live/production obligations remain unchanged.

**Retained dated v0.2.3 applicability — 4 October 2026, Edinburgh / Europe/London:** dated evidence below retains its original source, version and result; it does not qualify changed v0.2.3 bytes. Unchanged constraints remain subject to the [current documentation index](../V0.2.3-DOCUMENTATION-INDEX.md) and [SPEC-0005](../specs/SPEC-0005.md). The owner authorises a v0.2.3 PR and GitHub checks only, not merge, release or signing. Final-source, package, native and live qualification require independent evidence in [LOCAL-V023-0001](../qa/LOCAL-V023-0001.md). No retired provider, historical grant or old PASS is reactivated.

**Current-status navigation — 8 October 2026:** see [the reconciliation record](../qa/OCT8-RECONCILIATION-20261008.md) for actual PR/dependency/source/gate outcomes and the later owner-authorised conditional integration. This dated paragraph and the original evidence below are preserved history, not current authority, a new execution receipt or a reactivation of retired providers. Manual/native/live/production results do not transfer between sources.


**v0.2.2 applicability, 3 October 2026:** retained requirements/contracts apply only where unchanged; retired providers, disabled capabilities and older dated results are not current authority or new-source PASS. [SPEC-0004](../specs/SPEC-0004.md), [ADR-0017](../adr/ADR-0017.md) and [LOCAL-V022-0001](../qa/LOCAL-V022-0001.md) govern active zoom/startup changes and fresh evidence. Earlier evidence remains source-bound history; the later owner-reported v0.2.1 startup flash is recorded separately. The entire original v0.2.1 checkout/installer is unchanged.

Observed 2026-10-01 UTC. Current classification: **complete local automatic source/frontend/native/dependency gates PASS for immutable source `2bcacd7c6f2496eda233d4a069ec7b65a93d47b3`, tree `7c1c567faa9fb9df9bb9c6c62e40491fde07c084`; hosted package and principal-laptop manual gates remain independent and incomplete**. Parent `5b9355368bbdc08854a68eab5c130da499eb57de` differs only in two `src-tauri/Cargo.lock` values: `yoke-derive 0.8.3` becomes `0.8.4`, and its checksum becomes `ec8ebde2db3681e8c9980cc27822030e68752690ddfa9473e739aeb4dbde6d71`. Application/runtime/IPC source is unchanged. [LOCAL-0005](LOCAL-0005.md) remains the unmodified historical record for `c718e2c`; none of its PASS results are silently rebound to this source.

## Source and dependency decision

Read-only live-index review found `yoke-derive 0.8.4` published and not yanked, with the same dependency list as `0.8.3`; the locked `yoke` requirement remains compatible at `^0.8.2`. The correction removes the withdrawn Windows build macro without a broad dependency update. No GTK/off-target fork or manifest relaxation is authorized.

The complete audit still reports two warnings in the broad locked graph: `proc-macro-error 1.0.4` / RUSTSEC-2024-0370 and `glib 0.18.5` / RUSTSEC-2024-0429. Target-filtered Windows metadata contains 311 packages and contains neither package; it does contain `yoke-derive 0.8.4`. This is a scoped engineering observation, **not** zero warnings, owner warning acceptance, legal approval, production clearance or authority to change off-target dependencies.

## Exact post-freeze results

| Area | Result and boundary |
| --- | --- |
| Native aggregate | **PASS**, `2026-10-01T17:40:50.0312171Z`–`17:42:30.4589499Z`, outputs `3e79d6`, `87058b`, `f4650d`: locked/offline format, all-target check, all-target Clippy with warnings denied and full test suite. Result: **100/100 library tests**, empty main/documentation harnesses, all exit 0. |
| Frontend aggregate | **PASS**, `2026-10-01T17:41:00.9668288Z`–`17:41:18.9121690Z`, outputs `2e4e3d`, `5efe7f`: exact-source typecheck, **25/25 tests**, production build and moderate npm audit with zero known vulnerabilities. No live provider request. |
| Source/compatibility/notices/SBOM | **PASS**, `2026-10-01T17:44:36.5401101Z`–`17:44:42.1338226Z`, output `a332e0`: isolated Python guards, **90 UTF-8 files/zero likely-secret findings**, 73 compatibility fixtures plus actual locked comparison, notice negative self-tests, offline complete and Windows-target Cargo metadata, and SBOM generation. Git source remained clean. |
| Schema-2 notices | **PASS source-graph generation**: 575 total packages; 291 required; 45 direct missing texts; 10 repository-peer covered; four overlay covered, including one canonical steward text; 31 non-required missing; zero required missing; zero required `UNKNOWN`. Generated notice: **4,577,083 bytes**, SHA-256 `45c54b9a9c2ff83296ec2357be44dbb973c8109fa60963cc87754fe842a803ca`. Installer extraction/hash proof remains separate. |
| Complete RustSec audit | **PASS with two allowed warnings**, `2026-10-01T17:49:58.2293465Z`–`17:50:01.1287681Z`, output `a12965`, exit 0. It fetched 1,278 advisories, updated the crates.io index and scanned 476 packages using the valid owner advisory database at `C:\Users\Rene Nehru Alvarez\.cargo\advisory-db` and the pinned Rust toolchain path. No advisory ignore was used. |

The command bodies below were executed in separate aggregates from the reviewed checkout. Native Rust and the complete audit used the normal non-elevated owner account; frontend and isolated Python checks used the technical sandbox. These are not one combined owner run. Tool variables show the resolved process-only paths; no persistent user/system environment or global Git configuration was changed:

```powershell
$python = 'C:\Users\Rene Nehru Alvarez\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$pnpm = 'C:\Users\Rene Nehru Alvarez\.cache\codex-runtimes\codex-primary-runtime\dependencies\bin\fallback\pnpm.cmd'
$rustBin = 'C:\Users\Rene Nehru Alvarez\.rustup\toolchains\1.97.0-x86_64-pc-windows-msvc\bin'
$cargo = Join-Path $rustBin 'cargo.exe'
$env:RUSTC = Join-Path $rustBin 'rustc.exe'
$env:CARGO_HOME = 'C:\Users\Rene Nehru Alvarez\Documents\aster-desktop-premium\.cargo-home'
$env:PATH = "$rustBin;C:\Users\Rene Nehru Alvarez\.cargo\bin;C:\Program Files\Git\cmd;$env:PATH"
$env:ASTER_DISABLE_LIVE = '1'

& $cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
& $cargo check --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets
& $cargo clippy --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets -- -D warnings
& $cargo test --manifest-path src-tauri/Cargo.toml --locked --offline
& $pnpm typecheck
& $pnpm test
& $pnpm build
& $pnpm audit --audit-level moderate

& $python -I -B scripts/ci/scan_secrets.py --self-test
& $python -I -B scripts/ci/scan_secrets.py
& $python -I -B scripts/ci/check_tauri_versions.py --self-test
& $python -I -B scripts/ci/check_tauri_versions.py --root .
& $python -I -B scripts/ci/third_party_notices.py --self-test
& $cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked --offline | Set-Content reports/packaging-metadata.json
& $cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked --offline --filter-platform x86_64-pc-windows-msvc | Set-Content reports/packaging-metadata-windows.json
& $python -I -B scripts/ci/third_party_notices.py reports/packaging-metadata.json reports/packaging-metadata-windows.json . reports/THIRD-PARTY-NOTICES.txt reports/third-party-notice-inventory.json x86_64-pc-windows-msvc
& $python -I -B scripts/ci/cargo_sbom.py reports/packaging-metadata.json reports/rust.cdx.json reports/rust-licenses.json
& 'C:\Users\Rene Nehru Alvarez\.cargo\bin\cargo-audit.exe' audit --db 'C:\Users\Rene Nehru Alvarez\.cargo\advisory-db' --file src-tauri/Cargo.lock
```

The automatic suite covers protocol-2 capability-schema, forged receipt/status, exact-body, bounds, early revocation, private-journal, containment and independent helper-process termination/restart fixtures. It supports the same code-level portions identified in the ledger; installed owner-HWND positive T-039/T-040 journeys, physical power loss, loopback/proposal injection and additional project/live canaries remain NOT RUN.

## Retained incomplete attempts

- `330c00`: aggregate provenance **FAIL** because Git rejected repository ownership before the process-local exact-checkout `safe.directory` argument; no global Git configuration changed.
- `0fc115`: audit **INCOMPLETE/NOT RUN** because the crates.io index update reported `program not found`; it is not a complete audit result.
- `e024bb`: audit **INCOMPLETE/NOT RUN** because a workspace-cached advisory database lacked a valid origin. It was not repaired or treated as owner evidence.

Only complete result `a12965` credits the current audit. Passing subcommands inside an incomplete wrapper do not relabel that wrapper PASS.

## Hosted and principal-laptop boundary

The root reviewed the pending combined source snapshot and reran the Phase-2 structural ledger, source-guard self-test/90-file scan, 73 compatibility fixtures/actual comparison, whitespace check and unchanged-LOCAL-0005 check: output `bc0df0`, `2026-10-01T18:12:30.1261262Z`–`18:12:30.9325192Z`, all exit 0. This is a working-snapshot check, not a final CI/package PASS.

The subsequent, not-yet-committed packaging/evidence patch does not change the `2bcacd7` application behavior: it changes workflow/verifier evidence metadata and corrects obsolete Rust documentation/comments without changing a statement or function. Fresh native-owner PowerShell 5.1 child-process fixtures (`2a7247`, `2026-10-01T18:02:32.1224230Z`–`18:02:37.6505361Z`) reject eight invalid manifest bindings before destination creation and admit only the two allowed main `push`/`workflow_dispatch` shapes before deliberately rejecting their invalid inventory. Current verifier SHA-256 is `1489440934e7cda69fa3821b1840c048047446b849ffc207a9798d02f8974799`. These are verifier-negative/schema fixtures, **not** positive proof of an actual ZIP/installer. Earlier compression-type failures `4fc258`/`20bda5` and sandbox execution-policy denial `f6b46b` remain retained; no execution-policy change or bypass occurred. The final combined commit is nevertheless a new source identity and requires its own complete CI/package evidence.

GitHub PR #24 run [36901019822](https://github.com/comzulca/Aster-desktop/actions/runs/36901019822), run number 88, was still pending at this checkpoint. Source-policy and frontend jobs passed, and Windows Rust tests had passed so far; the Windows audit/evidence job and engineering package were not yet complete. Partial jobs are not an aggregate CI/package PASS, and no installer/hash is inferred.

The owner manually removed the unrelated Aster 0.10.0 installation. Read-only preflight `f72ce7` at `2026-10-01T17:19:53.8091386Z` found zero old uninstall entries, old executable/directory/processes absent and no new `Aster Desktop` directory. It did not install or launch anything and did not inspect/export the NVIDIA credential. The external preflight record is `C:\Users\Rene Nehru Alvarez\Documents\aster-desktop-premium\engineering-tools\PRIMARY-LAPTOP-PREFLIGHT-20261001.md`.

The reviewed external passive sampler is `C:\Users\Rene Nehru Alvarez\Documents\aster-desktop-premium\engineering-tools\Measure-AsterEngineeringProcess.ps1`, SHA-256 `292a2fa642a738d776e8f985383430fedaecb97109700e55c5244fd2d38b9a3d`. Its parser/denial/self-process fixtures pass, but **no Aster process has been sampled**; it provides bounded CPU/memory/process observations only, not UI-latency or user-perceived-performance proof.

Final engineering installation remains blocked until the exact final hosted run, five-entry artifact, manifest/supplemental/ZIP/EXE/alias hashes, `NotSigned` status and independently extracted NSIS resources pass verification. Only then will the owner manually install and exercise the reviewed build on this primary Windows 11 laptop/principal account and supply non-sensitive screenshots and observations. The owner superseded the Computer Use setup request with this manual workflow; no region-availability or regulation assertion was independently verified. No installed feature/UI/accessibility PASS is claimed. Credential Manager and other security-sensitive dialogs remain owner-only. Live requests remain **0/2**.

## Evidence summary

| Field | Current value |
| --- | --- |
| Source commit/tree/parent | `2bcacd7c6f2496eda233d4a069ec7b65a93d47b3` / `7c1c567faa9fb9df9bb9c6c62e40491fde07c084` / `5b9355368bbdc08854a68eab5c130da499eb57de` |
| Native | PASS; `17:40:50.0312171Z`–`17:42:30.4589499Z`; `3e79d6`, `87058b`, `f4650d`; 100/100 |
| Frontend | PASS; `17:41:00.9668288Z`–`17:41:18.9121690Z`; `2e4e3d`, `5efe7f`; 25/25 |
| Guards/metadata/notices/SBOM | PASS; `17:44:36.5401101Z`–`17:44:42.1338226Z`; `a332e0` |
| Complete audit | PASS with two warnings; `17:49:58.2293465Z`–`17:50:01.1287681Z`; `a12965` |
| Hosted workflow | Run `36901019822` / number 88; aggregate/package pending at this checkpoint |
| Installer identity | NOT RUN at this checkpoint |
| Principal-laptop install/manual UI/live | NOT RUN; live 0/2 |

Do not replace these pending fields with historical run 85, run 86, `LOCAL-0005`, partial run-88 jobs or an intermediate executable.
