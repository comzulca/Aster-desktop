# Frontend dependency maintenance — 17 November 2026

## Before-change scope and acceptance

The owner requests the necessary repository adjustments followed by a documentation cross-review. Baseline is main `c70126d27fab5134d83ddb9610be063e9975321e`, clean Linux checkout, Node 24.22.1 and the declared Corepack pnpm 11.19.0. The Windows target remains Windows 11 x64; this environment has no Rust toolchain. Existing requirements apply: SPEC-0001 NFR-003 / SEC-009 / AC-019 and SPEC-0006 NFR-012 / AC-085 / T-095. No new runtime behaviour or authority crossing is intended, so no new product requirement, IPC or threat model is introduced.

Consolidate the existing Dependabot proposals in one candidate: jsdom 30.1.3 → 31.0.0 (#71, `cfbd56b32d528dab605ffd7e18b7a7be5cad74bd`), Vitest 5.0.5 → 5.1.0 (#72, `a0919a97d68ee637783eb8485a0c7ecf4787f1be`) and Vite 8.3.5 → 8.3.6 (#73, `7dccc5cf8b5d20cf44809623ec4e84adf1eac84f`). Preserve their original PRs and failure evidence. The combined candidate needs its own checks; separate green runs cannot qualify the combination.

PR #71 [run 43191316639](https://github.com/comzulca/Aster-desktop/actions/runs/43191316639), frontend job 131091135961, reports 182 PASS / two FAIL. `main-window.test.tsx` compares CSS grid serialisation literally (`44px minmax(0,1fr)` versus `44px minmax(0px, 1fr)`). `zoom.test.tsx` expects the global fake timer count to reach zero after unmount but observes one. Windows-host and engineering-installer were SKIPPED. These observations are not proof of a native product regression.

Intended corrections: accept only the two equivalent CSSOM serialisations of the same exact two-track CSS layout; verify the actual zoom timer identity, replacement, expiry and cancellation independently of unrelated DOM timers. Keep the 1,000-ms notice, focus preservation, event-listener cleanup, negative authority assertions and all existing tests. Do not alter production UI to accommodate a test environment or clear unrelated timers to hide a leak.

Acceptance: frozen install; typecheck; complete frontend suite and production build; moderate advisory audit; source-policy guards; fresh unchanged five-job hosted workflow, Windows checks and exactly fifteen codec/six resource probes before integration. Keep the exact application version, policy 4 / IPC 5, toolchains, native dependencies, workflow, immutable resources and historical ledgers. New candidate results are NOT RUN at this pre-change checkpoint. Owner GUI/live/provider/signing/production acceptance and the unresolved GLib record remain separate.

## Documentation cross-review scope

Cross-check current README, engineering status, building guide, version index and release notes against manifest/lock inputs and host declarations. Correct current facts without rewriting dated receipts, archives or historical implementation notes. Preserve the exact README disclaimer and supplied preview. Record final hosted outcomes against their exact source in the PR receipt, avoiding a new source commit solely to record CI success.

## Additional advisory gate failure (before lock follow-up)

The first local combination passes all 184 frontend tests and the production build, but `corepack pnpm audit --audit-level moderate` FAILS on high-severity [GHSA-fc5q-39xc-vjpf](https://github.com/advisories/GHSA-fc5q-39xc-vjpf), picomatch <=4.0.7. The registry audit reports >=4.0.8 as patched. The proposed bounded follow-up was a transitive refresh to 4.0.8 within existing parent ranges, without an override, new root dependency or audit suppression. However, registry lookup `corepack pnpm view picomatch@4.0.8 version dist.integrity --json` returns `ERR_PNPM_PACKAGE_NOT_FOUND`; the available-version list ends at 4.0.7. The live GitHub advisory, reviewed/updated 17 November, lists no patched release. Therefore no invented version/checksum, vendored patch or audit bypass is applied; picomatch remains 4.0.7 and integration is BLOCKED. The original failed audit remains recorded; passing older Dependabot checks cannot qualify the current advisory database.


## Local source verification and cross-review

Observed 17 November 2026, 18:06–18:11 UTC, Linux technical runner (not the owner Windows session), Node 24.22.1 / Corepack pnpm 11.19.0, changes relative to baseline `c70126d` in this maintenance commit:

| Check / command | Result and scope |
| --- | --- |
| Original two fixtures with the new locked cohort, `vitest run src/main-window.test.tsx src/zoom.test.tsx` | Expected FAIL reproduced: two original assertions fail, nine cases pass. |
| First revised full suite | FAIL: three following zoom cases time out because restoring spies after real timers reinstalls fake timers. Retained finding; teardown now restores spies before returning to real timers. |
| Deliberately omit only the production zoom unmount clear, run the named notice fixture, restore exact original source | Expected FAIL at cancellation of the actual pending timer. The revised test still detects a real cleanup defect; no production change is retained. |
| `corepack pnpm install --frozen-lockfile` | PASS, matching final manifest/lock graph. |
| `corepack pnpm typecheck`, `corepack pnpm test`, `corepack pnpm build` | PASS; 184 tests / 15 files and production renderer build. |
| `corepack pnpm audit --audit-level moderate --json` | FAIL; one high picomatch advisory, twelve dependency paths, no suppression. |
| Existing secret, vendor, Tauri, React, WinRT and causal CPU guard commands/self-tests from `ci.yml` | PASS; no changed native cohort or guard. |
| First-party Markdown link/anchor review | PASS; 153 pages / 2,238 relative references, zero missing paths/anchors before this additive outcome section. |
| README disclaimer/preview, historical archives, frozen ledgers, native sources and packaged resources against baseline; `git diff --check` | PASS; original bytes retained. |
| Local Rust, Windows native and package probes | NOT RUN; no local Rust toolchain or Windows environment. Fresh hosted outcomes belong to the maintenance PR and its exact head/tested-merge/run; do not inherit earlier package PASS. |

The lockfile was regenerated by the declared package manager against npm, preserving direct React/Tauri/TypeScript inputs and resolving the selected development packages and their transitive ranges with registry integrity fields. These are actual resolved inputs, not a claim that the combined lock is byte-identical to any individual Dependabot proposal.

Cross-review: README, engineering status, building, release notes and the version index now describe the selected cohort, strict layout and timer checks, and the audit blocker. README's stale combined policy/IPC-4 statement is corrected against `policy.rs` (`POLICY_VERSION = 4`), `lib.rs` (`protocol_version: 5`) and `types.ts` (`protocolVersion: 5`). Setup/use and the current provider, transport and attachment entry pages were checked and need no behaviour changes for this development-only maintenance. Dated earlier implementation notes and engineering receipts retain their original dependency/test/artifact statements. The final PR receipt binds the published revision and hosted results; it must remain unmerged while the unchanged advisory gate fails. The original Dependabot PRs remain open rather than being described as resolved/integrated.
