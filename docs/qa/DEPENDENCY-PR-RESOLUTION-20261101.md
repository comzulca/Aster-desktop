# Dependency PR resolution — 1 November 2026

## Before-change scope and authority

The owner requests resolving every open PR and verifying that no active branch has pending conflicts. This authorises the scoped dependency integration and non-forced maintenance below. It supersedes older check-only restrictions for this task. Source, native and package gates remain mandatory; production release, signing, installed-owner acceptance and live provider calls remain separate.

Observed main is `f629482a7907c4657c5eb0969e1f73cd2d7fe6ef`. The nine open PRs each have five successful mandatory CI jobs against their individual sources. Those results do not qualify their combined dependency graph.

| PR | Reviewed head | Intended update |
| --- | --- | --- |
| #57 | `7596a2279e58b6c7fb7beeed93000532aa8eb7c9` | tauri-plugin-dialog 2.8.4 and its reviewed lock changes |
| #58 | `dcaddc3b48a5daebb11dbda38654062a441edfaf` | tauri-build 2.7.2 |
| #59 | `a0f21de6e2b597df5a054878f185e948d6325875` | tauri 2.12.2 |
| #60 | `e14144d3765aa45e86efa6e099d9295b1c7811e8` | aligned React/React DOM 19.3.1 and its existing cohort guard |
| #61 | `12f6721f0bb71981a02c4a7619e213491d8087db` | tokio 1.54.1 |
| #62 | `634813f2e91836c82b9ea305d08120b9f36abfea` | sha2 0.11.1 |
| #63 | `612dd929dbb35ab0886401566570147735d317cd` | jsdom 30.1.3 and its reviewed lock changes |
| #64 | `ada84d3f0e1bfc04e02d4c19efdf68396d109f27` | Vite 8.3.5 and its reviewed lock changes |
| #65 | `856eb9f5cb6e667409a5e144c3d3a30426cdcb4e` | Tauri CLI 2.12.4 |

This fulfils existing SPEC-0001 NFR-003 / SEC-009 / AC-019 and SPEC-0006 NFR-011 / NFR-012. It introduces no new product authority, IPC, provider, credential, storage or startup behaviour. T-097 and the existing WinRT/Tauri guards retain their full assertions. The exact windows 0.62.2 / windows-future 0.3.2 cohort remains unchanged.

## Composition and acceptance

Use a separate integration branch with current main and all nine reviewed PR heads as parents. Preserve the existing PRs and their histories; a normal merge commit must retain those ancestors, allowing GitHub to mark the original PRs integrated. Do not force-push or close an unintegrated PR merely to empty the list. The shared pnpm lock has overlapping Vitest context changes: resolve both the importer and snapshot key to jsdom 30.1.3 plus Vite 8.3.5, retaining all existing integrity checks and each PR's dependency updates.

Before integration, require fresh exact-source source-policy, frontend, codec-source-fixtures, windows-host and engineering-installer success for the complete combination. Keep frozen installation, advisory gates, strict Clippy, full suites, SBOM/notices, exact fifteen codec and six resource probes and the causal CPU receipt verifier. Verify actual-main CI again after integration, and inspect every remaining remote branch for unique commits or conflicts. Preserve unrelated work.

The local Linux checkout verifies every available textual baseline blob against the GitHub tree. Nine absent binary image/icon blobs remain unchanged through the remote base tree; local checks do not claim a complete build. Native and package execution is performed by the existing hosted Windows workflow. All new combined-source and actual-main results are NOT RUN at this before-change checkpoint; record subsequent observations against their exact revisions and runs in the integration PR metadata.

## Retained limitations

Partial v0.2.4 engineering scope, unavailable runtime GitHub mutations and automatic full-window compaction, owner-installed/native/live acceptance gaps, all 42 compound obligations, `qualificationComplete=false`, original budgets and unresolved GLib remain unchanged. Preserve historical failure receipts and the earlier React split-cohort failure. Successful maintenance CI is not production qualification or evidence of an owner-installed flicker cure.
