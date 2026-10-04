# LOCAL-0007 — corrected-NSIS exact-source validation

**v0.2.2 applicability, 3 October 2026:** retained requirements/contracts apply only where unchanged; retired providers, disabled capabilities and older dated results are not current authority or new-source PASS. [SPEC-0004](../specs/SPEC-0004.md), [ADR-0017](../adr/ADR-0017.md) and [LOCAL-V022-0001](../qa/LOCAL-V022-0001.md) govern active zoom/startup changes and fresh evidence. Earlier evidence remains source-bound history; the later owner-reported v0.2.1 startup flash is recorded separately. The entire original v0.2.1 checkout/installer is unchanged.

Observed 2026-10-01 UTC. Current classification: **complete local automatic source/frontend/native/dependency gates PASS for immutable source `9796d75a98b14d247d3f86e1803300727b3d0229`, tree `405c61c22f65df81d56fac02dcffeb1a3fd2861e`; final-main package and owner manual gates remain independent and incomplete**. Parent main is `dff78ec355056754eb99132716cd03b30c2b9ce8`. The primary external record was read in full from `C:\Users\Rene Nehru Alvarez\Documents\aster-desktop-premium\engineering-tools\FINAL-SOURCE-CHECKS-9796d75-20261001.md` (10,236 bytes, SHA-256 `9d5478879cff81151ff5a2463817927aced236b64075c55f9f8f565b3836733f`). This in-repository summary does not self-hash or transfer its PASS results to the later documentation-reconciliation revision.

## Narrow source scope and retained package failures

This source changes five files relative to its parent: Tauri bundle configuration, CI workflow, local artifact verifier, README and packaging QA documentation. It selects the supported NSIS compression value `none`, retains the strict advertised numeric-size/path/link/count/member/aggregate preflight and adds post-extraction regular-file bounds of 256 MiB each plus 512 MiB aggregate. These checks are not a generic pre-write disk-quota guarantee. Protocol-2 runtime/IPC behavior is unchanged.

PR run `36905811883` and main run `36908990412` each built a 3.61 MiB solid NSIS and then **FAIL**ed because the 7-Zip NSIS handler omitted the final uninstaller `Size`; neither artifact is deliverable or installed. Main job `110535239979` failed at `2026-10-01T19:19:14Z`. Independent NSIS 3.11 fixture `9121a0` (`19:07:31.9640734Z`–`19:07:32.6251906Z`) produced:

- compression-none fixture: 47,930 bytes, SHA-256 `38c18ae9cfde129aa569b510c0c33e517ab6d8061236c8300a5359ed6f203110`; strict parser **PASS**, two numeric advertised entries totaling 1,390 bytes;
- solid fixture: 39,394 bytes, SHA-256 `2c26bd160c7a8290b4a0d3677c3ee77b5280bbfd9723d89368921add0bb9c0a6`; expected strict-parser denial **PASS** because the final size is blank;
- none-mode advertised uninstaller metadata: 784 bytes; bounded reconstructed output: 42,288 bytes; all extracted fixture files: 42,894 bytes.

Corrected-verifier fixture `b7973a` (`19:16:09.0472774Z`–`19:16:09.6540781Z`) **PASS**ed under owner Windows PowerShell 5.1.26100.9549: none preflight admitted, solid preflight denied and eight invalid manifest bindings rejected before outputs. Verifier SHA-256 at this source is `a87287f7597356350317a297ffd4de06ad23738fac4111cadb3d0a33c783b40c`. A synthetic extracted-uninstaller hash read was denied (`687c76`, `19:12:03Z`); preserve that **FAIL** and its subsequent hash as **NOT RUN**. No fixture installer was executed and no protection was bypassed.

Exact Git synchronization retains signed-serialization attempt `6b33c0` as **FAIL** before import or HEAD change. The subsequent signature-byte reconstruction matched signed commit SHA `30c99f`; exact parent/new objects were imported and compare-and-swap HEAD update `2e2f9e` completed at `19:25:07.1449105Z`. No signature was fabricated and no force push, reset or global Git setting was used.

## Complete local suites

| Suite | Actual result |
| --- | --- |
| Native Rust, owner principal `QUARK2\Rene Nehru Alvarez` | **PASS**; `19:26:04.3275150Z`–`19:26:23.9508866Z`; outputs `7cd9fc`, `ea508b`; format, locked/offline all-target check, Clippy `-D warnings`, **100/100** library tests, zero failures, empty main/doc harnesses; exit 0. |
| Frontend, isolated `QUARK2\CodexSandboxOffline` | **PASS**; `19:26:01.2593931Z`–`19:26:16.4730458Z`; outputs `bebb4a`, `e33be7`; typecheck, **25/25** tests, Vite build and moderate npm audit with zero known vulnerabilities; exit 0. |
| Guards/compatibility/notices/SBOM/ledger, isolated sandbox | **PASS**; `19:27:27.7814431Z`–`19:27:35.2190325Z`; output `a369fe`; **91 UTF-8 files/zero findings**, 73 compatibility fixtures plus actual aligned comparison, notice self-tests, full/Windows locked offline metadata, notices, 475-dependency SBOM and structural ledger through phase 2; exit 0. |
| Complete native RustSec audit | **PASS with two disclosed warnings**; `19:27:32.2385561Z`–`19:27:35.6525008Z`; output `ef4f3f`; 1,278 advisories fetched, crates.io index updated, 476 dependencies scanned, no advisory ignore; exit 0. |

The two broad-lock warnings are `proc-macro-error 1.0.4` / RUSTSEC-2024-0370 and `glib 0.18.5` / RUSTSEC-2024-0429. The 311-package Windows metadata contains neither warned package. Schema-2 notices contain 575 total packages, 291 required, zero required missing and zero required unknown; generated text is 4,577,083 bytes, SHA-256 `45c54b9a9c2ff83296ec2357be44dbb973c8109fa60963cc87754fe842a803ca` (independent read `1e5e55`). These are source-graph observations, not zero warnings, an exact binary SBOM, legal approval or production clearance.

## Commands and authority

Native Rust and the complete audit ran under the normal non-elevated owner principal. Frontend and isolated Python checks ran under the technical sandbox. Process-local Rust 1.97.0 paths, `RUSTC`, repository-scoped `CARGO_HOME` and `ASTER_DISABLE_LIVE=1` were used; no persistent environment or global Git configuration changed.

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline
pnpm typecheck
pnpm test
pnpm build
pnpm audit --audit-level moderate

python -I -B scripts/ci/scan_secrets.py --self-test
python -I -B scripts/ci/scan_secrets.py
python -I -B scripts/ci/check_tauri_versions.py --self-test
python -I -B scripts/ci/check_tauri_versions.py --root .
python -I -B scripts/ci/third_party_notices.py --self-test
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked --offline
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked --offline --filter-platform x86_64-pc-windows-msvc
python -I -B scripts/ci/third_party_notices.py reports/packaging-metadata.json reports/packaging-metadata-windows.json . reports/THIRD-PARTY-NOTICES.txt reports/third-party-notice-inventory.json x86_64-pc-windows-msvc
python -I -B scripts/ci/cargo_sbom.py reports/packaging-metadata.json reports/rust.cdx.json reports/rust-licenses.json
python -I -B 'C:\Users\Rene Nehru Alvarez\.agents\skills\aster-builder\scripts\harness_gate.py' check --root . --through-phase 2
& 'C:\Users\Rene Nehru Alvarez\.cargo\bin\cargo-audit.exe' audit --db 'C:\Users\Rene Nehru Alvarez\.cargo\advisory-db' --file src-tauri/Cargo.lock
```

The automatic suite covers protocol-2 schema, forged receipt/status, exact-body, bounds, early revocation, private journal, containment and independent helper-process termination/restart fixtures. Installed owner-HWND positive T-039/T-040 journeys, physical power loss, loopback/proposal injection, owner UI/accessibility/performance and live provider canaries remain **NOT RUN**.

## Hosted and handoff boundary

PR #25 run `36913594210`, number 91, ultimately completed its candidate packaging job. Its candidate executable was 19,017,189 bytes/SHA-256 `ad48367cdd4750d0be0f4dbc4795c5f508be45e5553376121bd85a4588088d83`, `NotSigned`; artifact `11191296049` was 5,677,853 bytes with GitHub digest `abf9a9f28031f15c2dae4cce093ce7e1eae8cace39051b5e82124a6e83252627`. It was not downloaded, executed, installed or delivered, and printed logs do not substitute for independent supplemental payload inspection.

Normal main integration produced revision `a72cd6ba52929bfe85cff4798fff7de84290955c`, tree `405c61c22f65df81d56fac02dcffeb1a3fd2861e`, matching this locally validated tree. At the latest recorded checkpoint, main run `36916971838`, number 92, had the three prerequisite code jobs PASS and package job `110562611197` in progress. That run is intermediate evidence for this pre-reconciliation tree and cannot qualify the later documentation revision. All four successful jobs plus independent five-entry ZIP/manifest/EXE/NSIS resource/payload/delivery-alias verification remain required against a fresh final-main identity.

No installer or app has been executed. Owner installation/screenshots, installed hash comparison, keyboard/accessibility/performance/provider observations and uninstall remain **NOT RUN**; live calls remain **0/2**. Existing-file replacement remains disabled. Physical power-loss and absent installed loopback/proposal-injection procedures remain unresolved/NOT RUN. Issues #11/#12 stay open. This documentation reconciliation changes source identity after `9796d75`; its complete local and final-main qualification must be recorded separately before engineering handoff.
