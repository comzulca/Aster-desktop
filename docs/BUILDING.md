# Source checks and Windows engineering builds

Reconciled 5 November 2026 against software source `57339a6`. Read [AGENTS.md](../AGENTS.md) and the [current status](ENGINEERING-STATUS.md) before changes. The pinned declarations and frozen locks are authoritative; these procedures are instructions, not proof they ran in an owner's session.

## Build cohort

| Input | Current declaration |
| --- | --- |
| Target | Windows x64 / MSVC; Windows 11 owner target, hosted Windows Server image for CI |
| Node | 24, `>=24.15.0 <25` |
| Package manager | Corepack / `pnpm@11.19.0` from [package.json](../package.json) |
| Rust | 1.97.0 with rustfmt and Clippy from [rust-toolchain.toml](../rust-toolchain.toml) |
| Native prerequisites | Microsoft C++ build tools and WebView2; Tauri requirements |
| Reproducible inputs | [pnpm-lock.yaml](../pnpm-lock.yaml), [Cargo.lock](../src-tauri/Cargo.lock), reviewed vendored sources and resource identities |

A Linux frontend build is not a Windows native/package result. Keep Windows, GPU and security settings unchanged. Do not use protection exclusions or a failed local compiler result as a reason to bypass a gate.

## Source validation

In a dedicated checkout with the reviewed toolchain:

```powershell
corepack pnpm install --frozen-lockfile
corepack pnpm typecheck
corepack pnpm test
corepack pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --locked --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
```

The [source guard scripts](../scripts/ci) additionally check probable secrets, vendored provenance, Tauri/React/WinRT cohorts and CPU receipts. [Run-LocalEngineeringChecks.ps1](../scripts/qa/Run-LocalEngineeringChecks.ps1) is an existing reviewed procedure; inspect its parameters and scope before use. Native/disposable credential fixtures need their specific authorised scope and must not touch a real owner key or production target.

## Mandatory hosted/package gates

The exact implementation is in [ci.yml](../.github/workflows/ci.yml). Preserve all five jobs: `source-policy`, `frontend`, `codec-source-fixtures`, `windows-host` and `engineering-installer`. A documentation change still needs its own current-source checks before normal integration; an earlier green run does not qualify changed source.

The workflow generates target-filtered Cargo metadata, schema-2 third-party notices, licence inventories and frontend/Rust SBOM evidence before packaging. It requires the unchanged locked inputs, twelve resources including the ten byte-identified inert Aster Builder files, complete Windows Rust suite, strict Clippy and advisory checks. The default suite excludes two native Credential Manager cases; the workflow separately executes the exact two authorised disposable cases and records their scope.

After building the unsigned NSIS package, the actual release executable must pass all fifteen codec and six resource probes. Validate causal CPU evidence independently with [verify_cpu_receipt.py](../scripts/ci/verify_cpu_receipt.py). A diagnostic running after full-suite failure cannot turn the failed job into PASS. A compiled setup without the mandatory probes is not a qualified delivery.

## Stage without installing

Use the existing [passive verifier](../scripts/qa/Test-EngineeringInstallerArtifact.ps1) with independently observed expected ZIP SHA-256, source SHA, run and attempt. Default final staging requires actual-main `push`/`workflow_dispatch` provenance; a separately selected PR diagnostic must retain its candidate identity and must not become a final alias.

Cross-platform passive checks must distinguish Windows text checkout bytes from the ten immutable inert resources. The November installer contains the exact CRLF checkout representation of root LICENSE; no licence text or resource identity was changed. Do not claim a Linux static certificate-table check executed Windows Authenticode. The receipt records the native CI signature result separately from local static verification.

Do not launch the installer/application to verify an archive. Bind the manifest, supplemental evidence, all payload resources, notices, architecture, installer bytes/hash and signature status. An optional delivery alias must be proven byte-identical. Preserve `qualificationComplete=false`, owner-installed/live/signing/production gaps, historical failures and the unresolved GLib record. See the [delivered engineering receipt](qa/FINAL-ENGINEERING-V024-20261105.md) for an observed example.
