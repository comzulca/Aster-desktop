# STATUS-0001 — implementation evidence and release blockers

Date: 2026-09-24 UTC. Candidate revision: **uncommitted / NOT RUN** at first draft; record the immutable Git SHA and CI run links after publication. This status reports actual proof; the specification describes intended product behavior. Do not infer capability merely from code presence.

| Capability/gate | Current result | Required next evidence |
| --- | --- | --- |
| Private GitHub repository | **PASS** for private visibility, scoped app installation and first source publication: `185ad6db621700eed7be980d030ad918415d00b0`, Git tree `9ef2e6047e337f958120fccdd0baba3b72c5adbd` (57 files) | Review other organization/enterprise privileges and branch protection before release |
| Original Aster JPG | Byte copy matches uploaded image, SHA-256 `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887` | Render Windows UI at full and compact sizes |
| Requirements / threat / ADR | Written and structurally validated; behavioral gates **NOT RUN** | Link gate evidence to tested source revision |
| Key in Windows Credential Manager | Host-only source implementation; **NOT RUN** on Windows | Native entry/read/delete using a synthetic key; inspect IPC and local files |
| Fixed NVIDIA model pairs and streaming | Two strict provider contracts and parser fixtures in source; real calls **NOT RUN** | Windows fixture tests and one separately recorded live canary for each model, with user-entered key |
| Bounded root selection and guarded file reads | Root-selection source implemented; internal native reads and negative tests drafted. File-read IPC **DISABLED**; native tests **NOT RUN** | Native identity/alias tests, trusted full-content viewer and malicious renderer tests |
| Project excerpt disclosure | **DISABLED** pending native review of complete exact excerpt | Trusted full-content consent, request binding, denied spoof/replay tests |
| File proposal and apply | Internal proposal parsing source candidate; renderer proposal and existing/new file apply **DISABLED** | Full exact native diff and specific approval plus race, interruption, recovery and replay proof on Windows |
| Renderer typecheck/tests/build | **PASS** for hosted Ubuntu frontend job on source commit `185ad6d`: frozen install, typecheck, 9/9 Vitest tests, Vite build, audit, license and SBOM artifact. Windows UI acceptance **NOT RUN** | Inspect frontend evidence and run native Windows 11 UI tests |
| Rust build/native tests | `Cargo.lock` generated and inspected from read-only Windows bootstrap; compiled Rust/native tests **NOT RUN** | Commit reviewed lock and formatter patch; run locked Windows CI and manual Windows 11 acceptance |
| CI/audit/SBOM | Hosted `source-policy` and `frontend` **PASS** on `185ad6d`; `windows-host` **FAIL** on the intentional missing `Cargo.lock` gate. Frontend audit reports no known advisories; license/SBOM artifact exists. See `CI-0002.md` | Commit reviewed Cargo lock and Rust formatting patch, rerun all CI jobs on the new SHA, review licenses |
| Local laptop install | **NOT RUN**; execution workspace is Linux, not user Windows disk | User clones reviewed commit and runs Windows checks/install locally |
| Installer signature and production promotion | **NOT RUN** | Defined signing/verification process and all release acceptance evidence |

The host exposes capability flags to the UI; unsupported actions must be disabled. Source validation on Linux cannot prove Windows API, ESET, NVIDIA availability, native dialogs, or file publication safety. Preserve a failing or missing test as FAIL or NOT RUN until independently rerun with its source revision, environment, exact command, UTC time and evidence. See `CI-0001.md` and `docs/aster-builder/verification-plan.md`.
