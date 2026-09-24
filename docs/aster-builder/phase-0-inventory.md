# Phase 0 — project inventory and provenance

Status: initial source inventory recorded, implementation baseline pending. Date: 2026-09-24 UTC. This document records evidence available before new behavior is implemented. Recheck the Git revision, ownership and configured app permissions at every promotion gate.

| Item | Evidence and current status |
| --- | --- |
| Goal | New private `comzulca/Aster-desktop` Windows 11 app, version 0.1.0; secure deterministic host policy, two NVIDIA models, original Aster visual reference, bounded project reading and per-operation approved file changes. |
| GitHub ownership | User states they are sole owner of `comzulca`. On 2026-09-24 the authenticated GitHub organization UI showed one member (`renenehru`), and the newly created `comzulca/Aster-desktop` repository displayed **Private**. Initial repository commit: `c5abde1861e75836f57dc4d643eb0175b9bd2368` (README only). The ChatGPT Codex Connector has not yet been installed on `comzulca`; source publication and connector verification remain pending. |
| Original repository | New project; no existing local Git checkout was present at initial inspection. A local Git working baseline was subsequently initialized with remote `https://github.com/comzulca/Aster-desktop.git`. No application source commit or tested revision exists yet. Do not reuse source, migrations, licenses or release evidence from historical Aster projects. |
| Windows target | User requested `C:\Users\Rene Nehru Alvarez\Documents\comzulca\aster-desktop`. This exact path is a deployment/checkout request, never a runtime constant. Current execution workspace is `/workspace/scratch/925163fe19f8` on Linux; direct access to the user's Windows filesystem and Credential Manager has not been demonstrated. |
| UI reference | `upload/01-Aster_UI2.jpg`, 1917 x 1137 px, 179000 bytes, SHA-256 `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887`. `upload/Aster_UI2.jpg` is byte-identical. The planned app asset must retain the original bytes. The image is a design fixture, not proof of functionality. |
| Nemotron source | `upload/01-nemotron-3.5-lightning-30b-a3b.txt`, 779 bytes, SHA-256 `826261f2c1ae3ebba7f365b7ad8a865ad46f2e0b3ecb347a35c1e22482db63dc`. NVIDIA `https://integrate.api.nvidia.com/v1`, exact model `nvidia/nemotron-3.5-lightning-30b-a3b`; sample uses streaming and separate `reasoning_content`. Literal `$NVIDIA_API_KEY` is a placeholder, not a credential. |
| Muse source | `upload/02-muse-glimmer-30b.txt`, 414 bytes, SHA-256 `9a44533e93b5fdd645e174e6a6f4f397216e1090d0ff4c1c1d8220fbb0562407`. Same NVIDIA base URL, exact model `meta/muse-glimmer-30b`; supplied sample uses non-streamed JSON. User-facing follow-up chose streaming for both; Muse streaming requires its own verified and tested contract. |
| Secret | No NVIDIA key has been supplied and none should be pasted into chat or committed. Design calls for Windows Credential Manager generic credential via native host-only entry. Windows key storage and live calls are NOT RUN. |
| Capability decisions | User explicitly authorizes new-file and existing-file changes only after separate approval per operation. Allow bounded project reads after an explicit root selection. Disable shell, browser, web/MCP, external filesystem and automatic model tool privilege. No persistent chat history decision has been supplied; use ephemeral in-memory conversations as the default. |
| Development toolchain | Current execution workspace is Linux; platform build, installer behavior, ESET behavior, Credential Manager and Windows file identity/collision checks are NOT RUN. Reconfirm availability of Rust, Node/pnpm and Windows CI on their actual targets. |

## Phase progression

Phase 0 describes the inputs and unknowns above. Phase 1 is `docs/specs/SPEC-0001.md`. Phase 2 is `docs/security/THREAT-MODEL-0001.md` and `docs/adr/ADR-0001.md` through `ADR-0003.md`. The provider agent owns `docs/providers/`; record the dated official provider contract and request/response fixtures before any live adapter is promoted. Review Phase 0 through 2 as design artifacts, not as evidence that an app runs.

## Open facts and dependencies

1. The organization UI currently shows one member and the repository is private. Verify the scoped ChatGPT Codex Connector installation and its effective repository access before publishing source. If another organization owner exists later, GitHub grants that owner access by organization design.
2. Choose and verify a Windows native file publication strategy before enabling existing-file replacement. Without a passing Windows race/crash/recovery suite, expose proposals only for existing files. New-file creation is separately gated.
3. Validate exact NVIDIA provider protocol and any current costs/data-use terms before live use. The attached examples are API inputs, not proof of a successful canary or current availability.
4. Verify installed app against the user's Windows checkout, native credential storage and both live streaming models. Neither Linux tests nor a source build substitutes for those checks.
