# Aster-desktop v0.1.0

Windows 11 desktop harness under the private `comzulca/Aster-desktop` repository. A Tauri/Rust host owns provider transport, the Windows credential, project grants and operating-system dialogs. React renders the conversation and the user's original Aster reference image. Nemotron 3.5 Lightning 30B A3B is the default; Muse Glimmer 30B is the other registered NVIDIA model. Both adapters request streaming, with separate fixtures. The model receives no shell, browser, MCP, arbitrary URL or file-write tool.

## Evidence and current scope

This is an **implementation candidate**, not a signed or production-certified Windows release. Read [`docs/qa/STATUS-0001.md`](docs/qa/STATUS-0001.md) for the exact verified/blocked capabilities and [`docs/specs/SPEC-0001.md`](docs/specs/SPEC-0001.md) for acceptance conditions. Ordinary typed chat can be exercised after the Windows build and credential entry; availability of each NVIDIA model requires a live canary. A project root can be selected. Native guarded read and proposal code remains internal and unexposed while the exact-content viewer and Windows evidence are incomplete. Project excerpts and file changes cannot be sent or applied by the renderer. The UI must report these unavailable capabilities honestly.

The original image is [`public/assets/Aster_UI2.jpg`](public/assets/Aster_UI2.jpg); its SHA-256 is `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887`. No NVIDIA API key belongs in this repository, chat, a command line or an environment file. The app's trusted Windows credential dialog stores the key with Windows Credential Manager for the signed-in user. Another process running as that same user may read a generic credential.

## Source and architecture

| Area | Entry point | Ownership |
| --- | --- | --- |
| Contract and threat model | `docs/specs/`, `docs/security/`, `docs/adr/` | Versioned requirement and security decisions before capabilities |
| Host | `src-tauri/src/` | Fixed provider registry, credential, window-scoped IPC, conversation policy, project access |
| Renderer | `src/` | Aster interface, inert text rendering, status and user intent |
| CI and local setup | `.github/workflows/`, `scripts/` | Fail-closed tests, source checks and optional Windows clone helper |
| Gate evidence | `docs/aster-builder/contract.json`, `docs/qa/` | Per-phase proof with `NOT RUN` until evidence exists |

`python3 scripts/ci/publish_preflight.py .` inventories Git-visible files with their SHA-256 and Git object IDs, rejecting a changed original JPG. Keep the manifest beside the publication evidence and compare it with the published commit; the helper does not upload or modify files.

Determinism covers host admissions, denials, grants and terminal transitions under the same versioned state and fixture. It does not promise identical generated text or timing from an external model.

## Windows checkout and candidate build

After source has been published and reviewed, use **PowerShell** in your ordinary Windows account to clone the private repository into the requested location. Git for Windows with Git Credential Manager should prompt for GitHub authentication as needed. Leave Windows and ESET protection enabled.

```powershell
$company = Join-Path $env:USERPROFILE 'Documents\comzulca'
New-Item -ItemType Directory -Force -Path $company | Out-Null
git clone --branch main --single-branch https://github.com/comzulca/Aster-desktop.git (Join-Path $company 'aster-desktop')
cd (Join-Path $company 'aster-desktop')
git remote -v
git rev-parse HEAD
```

If that destination already exists, inspect it and update it deliberately; the optional `scripts/windows/bootstrap.ps1` refuses to overwrite it. No code in this source hardcodes a person's profile name. To build the candidate, install Node 22, Corepack/pnpm, Rust 1.97.0 (pinned in `rust-toolchain.toml`), Microsoft C++ build tools and WebView2; once the reviewed lockfiles are committed, run:

```powershell
corepack pnpm install --frozen-lockfile
corepack pnpm typecheck
corepack pnpm test
corepack pnpm tauri:dev
```

The CI Windows job also runs `cargo fmt`, `cargo check --locked`, `cargo clippy --locked`, `cargo test --locked`, dependency audit and inventories. Do not start a live NVIDIA request until the Windows checks and exact-model canaries have passed. No credential is needed for fixture tests. If either lockfile is absent, use the read-only bootstrap workflows in `.github/workflows/` and review the candidates before committing them; ordinary CI intentionally fails without both.

## Security and contributions

Read [`AGENTS.md`](AGENTS.md) before changes. Treat model output, project text and renderer events as untrusted. Update the requirement, threat mapping, negative test and gate evidence together when crossing a new boundary. File modification requires an independently reviewed exact operation; an on-screen button does not grant host authority. Keep private reports in the GitHub Actions artifacts, not the source tree.
