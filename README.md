# Aster Desktop v0.1.0

Aster Desktop is a Windows 11, local-first agentic harness in the private `comzulca/Aster-desktop` repository. A Tauri/Rust host owns provider transport, Windows credential access, project-file authority and native review; React renders inert conversation text and the user's original Aster reference image. The registered providers are exactly `nvidia/nemotron-3.5-lightning-30b-a3b` and `meta/muse-glimmer-30b`. The model receives no shell, browser, MCP, arbitrary URL or generic file-write tool.

## Current engineering boundary

The **final unsigned engineering build** at source `1782d3375db47466f89be6bd3cd3c6affbfa95b7` has passed its complete automated and artifact qualification and was accepted by the owner for internal engineering evaluation on 2026-10-02. It is not a signed or production-certified release. Pull-request packages remain `unsigned-engineering-candidate` evidence and must not be installed or delivered; only a fully gated package from `refs/heads/main` may be classified `unsigned-engineering-build`. [OWNER-ACCEPTANCE-0001](docs/qa/OWNER-ACCEPTANCE-0001.md) separates verified build evidence from the owner's category-level PASS reports and the [v0.2.0 follow-up](docs/qa/V0.2.0-FOLLOW-UP-0001.md). Protocol 2 exposes these host-owned capabilities:

| Capability | Current behavior | Evidence boundary |
| --- | --- | --- |
| Ordinary chat | Fixed NVIDIA endpoint/model registry, bounded streaming parser, cancellation and ephemeral in-memory continuation. | Exact-source automatic fixtures and final-main CI/package qualification PASS; providers/history category PASS (owner-reported). Detailed per-model canary records are deferred, not inferred. |
| Project selection | Native root and file pickers; the renderer receives relative path, request ID, SHA-256 and byte count, never selected project plaintext. Files may be read up to the workspace limit, but a project-bearing composed prompt is capped at 32 KiB and is denied rather than truncated. | Exact-source automatic T-038/T-039 portions, including exact-limit/+1 coverage, and final-main CI/package qualification PASS; projects/files category PASS (owner-reported). Unreported installed-case details are deferred. |
| Project disclosure | Host serializes the exact provider body, shows those exact bytes in an owner-HWND native review, consumes a one-use approval, revalidates the snapshot and transmits those bytes once. Project prompts and derived answers do not enter continuation history. Failed/cancelled egress revokes the snapshot; completion retains only the five-minute proposal right, with 60-second expiry cleanup. | Exact-source automatic T-039 portions, including early-error revocation, and final-main CI/package qualification PASS; projects/files category PASS (owner-reported). Positive installed loopback/exact-body proof is deferred because no diagnostic seam ships. |
| Proposal review | One strict, bounded typed proposal is parsed and shown in a native host review. Renderer/model text cannot approve it. | Exact-source automatic T-040 portions and final-main CI/package qualification PASS; projects/files category PASS (owner-reported). Positive installed diagnostic cases are deferred, not certified by the category report. |
| Create new file | After native approval, the host uses a private current-user DPAPI journal, revalidates identities and creates with Windows `CREATE_NEW`; ambiguous interruption becomes `ATTENTION_REQUIRED`, with conservative safe-helper recovery. | Exact-source process termination/restart fixtures and final-main CI/package qualification PASS; projects/files category PASS (owner-reported). Unreported installed mutation/recovery details and physical power-loss proof are deferred. |
| Replace existing file | Proposal and review only. A second native file choice must match the proposed relative path and old SHA-256; confirmation consumes the candidate but never mutates. `applyExistingFile` is hard false. | No safe publication primitive/proof has been accepted; this remains deliberately unavailable. |

Protocol 2 status contains exactly four Boolean fields: `projectRead`, `proposals`, `applyNewFile` and `applyExistingFile`. An unknown or incompatible shape denies. Capability reporting describes code paths; it is not release evidence.

Final main source `1782d3375db47466f89be6bd3cd3c6affbfa95b7`, tree `c7e116d9bbba63f5b0b2ace7cf0757cb16a30b3b`, passed complete local qualification and all four jobs in [main run 94](https://github.com/comzulca/Aster-desktop/actions/runs/37040168700), attempt 1, completed `2026-10-02T18:04:48Z`: Rust formatting/locked check/all-target Clippy with warnings denied and 100 library tests; typecheck, 25 frontend tests, build and npm audit; 92-file source guard with zero findings; 73 T-037 fixtures plus actual comparison; metadata/notices/SBOM, structural ledger through phase 2 and complete RustSec audit. Actual artifact `11243338033` and its five-entry ZIP, manifest, unsigned installer, extracted NSIS resources and delivery alias were independently verified. [OWNER-ACCEPTANCE-0001](docs/qa/OWNER-ACCEPTANCE-0001.md) records the exact build identity and evidence boundaries.

The broad lock audit retains exactly two warnings—`proc-macro-error 1.0.4` and `glib 0.18.5`—both absent from the 311-package Windows target metadata. The owner's internal-engineering scope acceptance does not make the audit warning-free, constitute legal approval or authorize production distribution. [LOCAL-0007](docs/qa/LOCAL-0007.md) preserves the earlier corrected-NSIS source `9796d75`; [LOCAL-0006](docs/qa/LOCAL-0006.md) remains historical for runtime/lock source `2bcacd7`. [PACKAGING-0001](docs/qa/PACKAGING-0001.md) retains the solid-NSIS failures and subsequent correction. Pre-reconciliation main run 92, historical run 85 and all PR packages are not substitutes for the delivered final-source artifact. This documentation reconciliation does not change the delivered runtime bytes or transfer acceptance to a future binary.

## Security and data ownership

The original image is [`public/assets/Aster_UI2.jpg`](public/assets/Aster_UI2.jpg), SHA-256 `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887`. No NVIDIA API key belongs in the repository, chat, command line, environment file or renderer. The host reads one fixed Windows Generic Credential target for the signed-in user. The owner may use Windows Credential Manager or the trusted native entry dialog after the synthetic compatibility checks; neither route sends secret bytes through renderer IPC. Another process running as the same user may read a Generic Credential, which is a documented residual risk.

Project text is untrusted. A selected file is not authority to disclose or modify it. Disclosure and file creation each require their own exact, host-owned, one-use native review. Existing-file replacement remains proposal-only. Generated project prompts, answers and proposal content are not written to ordinary chat history. No code path may turn a model response, renderer Boolean or project `AGENTS.md` into authority.

## Source and architecture

| Area | Entry point | Ownership |
| --- | --- | --- |
| Requirements and threats | `docs/specs/`, `docs/security/`, `docs/adr/` | Versioned behavior, trust boundaries and decisions |
| Host | `src-tauri/src/` | Provider registry, credential, window-scoped IPC, project access and native review |
| Renderer | `src/` | Aster interface, inert text rendering, status and explicit user intent |
| CI and local setup | `.github/workflows/`, `scripts/` | Fail-closed tests, inventories, notices, payload evidence and unsigned packaging |
| Evidence | `docs/aster-builder/contract.json`, `docs/qa/` | Source-bound PASS/FAIL/NOT RUN records |

`python scripts/ci/publish_preflight.py .` inventories Git-visible files with SHA-256 and Git object IDs and rejects a changed original JPG. Keep its manifest with publication evidence and bind it to the reviewed commit; it does not upload or modify files.

Determinism covers host admissions, denials, grants and terminal transitions under the same policy version and fixture. It does not promise identical generated prose or remote timing.

## Windows build

Use an ordinary, non-elevated Windows account with existing Windows and ESET protections left enabled. Build only a reviewed immutable revision. Required inputs are Node 24 (`>=24.15.0 <25`), Corepack/pnpm 11.19.0, Rust 1.97.0 from `rust-toolchain.toml`, Microsoft C++ build tools and WebView2.

```powershell
corepack pnpm install --frozen-lockfile
corepack pnpm typecheck
corepack pnpm test
corepack pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
New-Item -ItemType Directory -Force -Path reports | Out-Null
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked |
  Set-Content -LiteralPath reports/packaging-metadata.json
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 --locked `
  --filter-platform x86_64-pc-windows-msvc |
  Set-Content -LiteralPath reports/packaging-metadata-windows.json
python scripts/ci/third_party_notices.py reports/packaging-metadata.json `
  reports/packaging-metadata-windows.json . reports/THIRD-PARTY-NOTICES.txt `
  reports/third-party-notice-inventory.json `
  x86_64-pc-windows-msvc
corepack pnpm tauri:build -- --no-sign
```

CI additionally runs source policy, compatibility fixtures, audits, inventories, SBOM/notices generation, executable hash/size and `NotSigned` checks. Both the Windows-host and packaging jobs generate schema-2 notices from complete plus target-filtered Cargo metadata; a development build must do the same before Tauri bundling. The inventory distinguishes all installed source packages from the `requiredForPackagedApp` Cargo normal-dependency closure and production npm graph, and fails unless that required subset has zero missing text and zero unknown license declarations. This is still source-graph evidence, not an exact binary SBOM or legal approval.

Generated `THIRD-PARTY-NOTICES.txt` and proprietary `LICENSE.txt` are bundled at the installed application root. CI uses 7-Zip to extract the actual NSIS payload, requires exactly one of each resource and hashes those bytes against their source inputs. The fail-closed installer gate separately bounds advertised pre-extraction metadata and actual expanded files: every advertised member must have a numeric size no greater than 256 MiB, its advertised aggregate must be no greater than 512 MiB, then every extracted regular file and the actual aggregate must satisfy the same respective limits before acceptance. The private packaging artifact has exactly five entries: manifest, setup executable, notices, notice inventory and installer-payload listing. The manifest binds all three supplemental-evidence hashes plus `bundle_resource_sha256` for the two extracted resources; payload evidence is classified `verified-extracted-nsis-payload`. Exact-source schema 2 contains 575 installed source packages and 291 in the target-required graph: 45 direct text gaps, 10 peer-covered, four overlay-covered (one canonical steward text), and 31 explicitly missing but non-required (30 other-target/build Cargo packages plus npm development dependency `saxes@6.0.0`). Required missing and required unknown counts are both zero. The final-main package independently reproduced and bound 4,577,083 notice bytes with SHA-256 `45c54b9a9c2ff83296ec2357be44dbb973c8109fa60963cc87754fe842a803ca` to its extracted resources. None of this is external-distribution or legal approval. Hosted Windows Server evidence remains separate from the owner's Windows 11 category acceptance.

The product identity remains **Aster Desktop**. The delivered manifest-named `Aster Desktop_0.1.0_x64-setup.exe` and requested byte-identical alias `Aster_0.1.0_x64-setup.exe` are both 19,017,189 bytes, SHA-256 `0357a0798714922dd6bc6dc612eb6ed0a5c9c6224197328e1f186112c0256bcb`, Authenticode `NotSigned`. The alias is not a different product or build. The installed application independently matched the extracted payload SHA-256 `f05eed42074f2e1b4b37d5d3be23b7227e0930726cac58f8cf65285f7b1bba62` in a read-only check at `2026-10-02T19:25:00.4822961Z`; that is a later identity check, not the owner's test time.

## Engineering build handoff and owner laptop acceptance

The delivered unsigned engineering setup passed these build-time gates against the immutable final-main source above. Every future build must establish its own evidence:

1. Bind the source to one commit/tree and repeat the complete source, frontend and Windows Rust suites, including T-038–T-040, without inheriting run-85 evidence.
2. Generate the SBOM and schema-2 notice inventory; require zero missing/unknown entries in the target-filtered packaged-app closure, reproduce the exact two broad-lock warnings and confirm neither is present in target-filtered Windows metadata, and retain provenance for peer/official license-text overlays. This is not owner warning acceptance; source inventory alone is not payload or legal proof.
3. Build the unsigned installer and use the repository verifier plus 7-Zip to bind source/manifest/supplemental evidence/ZIP/EXE hashes and byte counts, independently extract/hash both bundle resources, require Authenticode `NotSigned`, and confirm the requested delivery alias is byte-identical.

On 2026-10-02 the owner accepted the installed build in Edinburgh, display location `Edinburgh`, timezone `Europe/London` (BST, UTC+01:00), using the existing Windows 11 administrator account with normal, non-elevated Aster launch. No exact test clock time was supplied. All six categories are **PASS (owner-reported)**: interface; performance; providers/history; credentials/uninstall; projects/files; scope acceptance. These are category-level engineering results, not fabricated per-case measurements or an independent agent-observed GUI run. See [OWNER-ACCEPTANCE-0001](docs/qa/OWNER-ACCEPTANCE-0001.md).

The owner explicitly transferred all unreported specifics to the [v0.2.0 follow-up](docs/qa/V0.2.0-FOLLOW-UP-0001.md), including detailed live-call records, measurements, special credential/security cases and unavailable installed diagnostic journeys. No request counts, timings, keyboard actions or transport/secret-boundary results are inferred. Do not create/use a guest account, bypass protections, disclose private project content or infer authority for extra live calls from this acceptance decision. Historical failures remain retained.

Existing-file replacement remains an explicit proposal-only engineering limitation: the safe publication primitive/proof is unresolved and `applyExistingFile` stays false. It and all deferred specifics are tracked for v0.2.0 rather than claimed PASS. With the owner's dated v0.1.0 engineering acceptance and explicit scope transfer, issues #11/#12 are eligible for scoped closure while preserving their history; closure does not establish production readiness or enable replacement. Remote issue state must be verified separately.

Read [`AGENTS.md`](AGENTS.md) before changes. Update the requirement, threat mapping, negative test and source-bound evidence together whenever authority crosses a new boundary.
