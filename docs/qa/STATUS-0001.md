# STATUS-0001 — implementation evidence and release blockers

Date: 2026-09-24 UTC. Candidate revision: **uncommitted / NOT RUN** at first draft; record the immutable Git SHA and CI run links after publication. This status reports actual proof; the specification describes intended product behavior. Do not infer capability merely from code presence.

| Capability/gate | Current result | Required next evidence |
| --- | --- | --- |
| Private GitHub repository | Observed private under `comzulca`, initialized with README at `c5abde1861e75836f57dc4d643eb0175b9bd2368`; application source **not yet published** | Verify exact source commit and scoped app access; review organization privileges and branch protection |
| Original Aster JPG | Byte copy matches uploaded image, SHA-256 `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887` | Render Windows UI at full and compact sizes |
| Requirements / threat / ADR | Written and structurally validated; behavioral gates **NOT RUN** | Link gate evidence to tested source revision |
| Key in Windows Credential Manager | Host-only source implementation; **NOT RUN** on Windows | Native entry/read/delete using a synthetic key; inspect IPC and local files |
| Fixed NVIDIA model pairs and streaming | Two strict provider contracts and parser fixtures in source; real calls **NOT RUN** | Windows fixture tests and one separately recorded live canary for each model, with user-entered key |
| Bounded root selection and guarded file reads | Root-selection source implemented; internal native reads and negative tests drafted. File-read IPC **DISABLED**; native tests **NOT RUN** | Native identity/alias tests, trusted full-content viewer and malicious renderer tests |
| Project excerpt disclosure | **DISABLED** pending native review of complete exact excerpt | Trusted full-content consent, request binding, denied spoof/replay tests |
| File proposal and apply | Internal proposal parsing source candidate; renderer proposal and existing/new file apply **DISABLED** | Full exact native diff and specific approval plus race, interruption, recovery and replay proof on Windows |
| Renderer typecheck/tests/build | Local Linux engineering checks **PASS**: frozen offline install, typecheck, 9/9 Vitest tests, Vite build. The exact-revision hosted gate remains **NOT RUN** | Commit reviewed `pnpm-lock.yaml`, repeat frozen checks in CI on published SHA and inspect Windows UI |
| Rust build/native tests | **NOT RUN**, Linux scratch lacks Rust toolchain and Windows | Commit reviewed `Cargo.lock`; run locked Windows CI and manual Windows 11 acceptance |
| CI/audit/SBOM | YAML and local secret scanner verified; local `pnpm audit --audit-level moderate` reports zero known advisories, frontend license/SBOM commands produce parseable JSON. Hosted workflow **NOT RUN** | Run all jobs against published source with both committed lockfiles; review licenses |
| Local laptop install | **NOT RUN**; execution workspace is Linux, not user Windows disk | User clones reviewed commit and runs Windows checks/install locally |
| Installer signature and production promotion | **NOT RUN** | Defined signing/verification process and all release acceptance evidence |

The host exposes capability flags to the UI; unsupported actions must be disabled. Source validation on Linux cannot prove Windows API, ESET, NVIDIA availability, native dialogs, or file publication safety. Preserve a failing or missing test as FAIL or NOT RUN until independently rerun with its source revision, environment, exact command, UTC time and evidence. See `CI-0001.md` and `docs/aster-builder/verification-plan.md`.
