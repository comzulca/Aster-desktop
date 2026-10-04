# LOCAL-0005 — protocol-2 exact-source validation

**v0.2.3 applicability — 4 October 2026, Edinburgh / Europe/London:** dated evidence below retains its original source, version and result; it does not qualify changed v0.2.3 bytes. Unchanged constraints remain subject to the [current documentation index](../V0.2.3-DOCUMENTATION-INDEX.md) and [SPEC-0005](../specs/SPEC-0005.md). The owner authorises a v0.2.3 PR and GitHub checks only, not merge, release or signing. Final-source, package, native and live qualification require independent evidence in [LOCAL-V023-0001](../qa/LOCAL-V023-0001.md). No retired provider, historical grant or old PASS is reactivated.

**v0.2.2 applicability, 3 October 2026:** retained requirements/contracts apply only where unchanged; retired providers, disabled capabilities and older dated results are not current authority or new-source PASS. [SPEC-0004](../specs/SPEC-0004.md), [ADR-0017](../adr/ADR-0017.md) and [LOCAL-V022-0001](../qa/LOCAL-V022-0001.md) govern active zoom/startup changes and fresh evidence. Earlier evidence remains source-bound history; the later owner-reported v0.2.1 startup flash is recorded separately. The entire original v0.2.1 checkout/installer is unchanged.

Observed 2026-10-01 UTC. Current classification: **post-freeze local source/frontend/native gates PASS for immutable runtime source `c718e2c75a6c94a77020eb7e047c07503945b388`, tree `24e48836a421e099c47d94593b49409749096088`; hosted CI/package/manual gates remain independent**. The evidence-only documentation that cites this source may be a later commit; it does not change or re-identify the tested runtime bytes.

## Environments and authority

- Native Windows execution: `QUARK2\Rene Nehru Alvarez`, Windows NT `10.0.26300.0`, normal existing owner environment. A process-local `git -c safe.directory=<exact checkout>` was used only for provenance checks; no global Git, Windows or ESET setting changed. No live provider request or real-credential read was part of this run.
- Frontend execution: `QUARK2\CodexSandboxOffline` on the same Windows version, bundled Node `24.19` and pnpm `11.19`, with live provider access disabled.
- Source under test: commit `c718e2c75a6c94a77020eb7e047c07503945b388`, tree `24e48836a421e099c47d94593b49409749096088`. `git rev-parse HEAD`, the Git tree and the authenticated GitHub commit/tree matched; `git status --porcelain --untracked-files=all` was empty after every successful aggregate.

## Exact post-freeze results

| Area | Result and boundary |
| --- | --- |
| Native aggregate | **PASS**, `2026-10-01T16:46:46.2806428Z`–`16:46:49.2001051Z`, output `9f332a`: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`; `cargo check --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets`; `cargo clippy --manifest-path src-tauri/Cargo.toml --locked --offline --all-targets -- -D warnings`; `cargo test --manifest-path src-tauri/Cargo.toml --locked --offline`. Result: 100/100 library tests, empty main and documentation harnesses, all exit 0. |
| Frontend aggregate | **PASS**, `2026-10-01T16:44:37.8958197Z`–`16:44:48.6676378Z`, outputs `a031df` and `b54ee4`: exact-SHA checks, `pnpm typecheck`, `pnpm test` (**25/25**) and `pnpm build`. This post-freeze aggregate did not perform a new dependency resolution. |
| Source, compatibility, notice self-test and npm audit | **PASS**, `2026-10-01T16:47:20.2940425Z`–`16:47:21.6228521Z`, output `72d5cb`: Python `-I -B` secret-guard self-test/scan (**90 UTF-8 files, zero likely-secret findings**), T-037 self-test and actual locked pair (**73/73 fixtures plus actual comparison**), notice-generator negative self-test, and `pnpm audit --audit-level moderate` with zero known vulnerabilities. This aggregate did not generate the canonical notice output. |
| Canonical notice observation | Earlier output `6f7658` used the lock/script inputs later frozen in `c718e2c`: schema 2, 575 total/291 required, 45 direct missing, 10 peer-covered, four overlay-covered, 31 non-required missing, zero required missing and zero required unknown. `THIRD-PARTY-NOTICES.txt`: 4,577,083 bytes, SHA-256 `b181b1e7d4aa59018bd23a828112a72adbc9a2b8d76764132159ba924bcb1c92`. This was not rerun in post-freeze aggregate `72d5cb`; the hosted package job must reproduce it. It is source-graph evidence, not extracted installer-payload or legal evidence. |
| Original Aster JPG | **PASS** byte check at `2026-10-01T16:57:58Z`, output `75839e`: `public/assets/Aster_UI2.jpg` SHA-256 `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887`. Rendering/layout/accessibility remain manual. |
| RustSec disposition | Exactly three allowed warnings remain: unmaintained `proc-macro-error 1.0.4` / RUSTSEC-2024-0370; unmaintained and unsound `glib 0.18.5` / RUSTSEC-2024-0429; yanked `yoke-derive 0.8.3`. Target-graph review excludes the first two from the packaged Windows closure and retains `yoke-derive` as a build-time procedural macro; the full exact-source Rust suite passed. Retaining this locked graph is acceptable only for the private engineering test classification, not a zero-warning, owner/legal-risk or production claim. |

The suite covers the exact 32-KiB/+1 project-composer boundary, early pending-state revocation, metadata-only receipts, exact reviewed-body dispatch, one-use/revalidation behavior, ephemeral continuation, strict proposal parsing, private journal/new-file behavior, independent helper-process termination/restart recovery and the journal-container-as-project containment regression. It supports the code-level portions of T-038 through T-040 but does not substitute for normal installed-app native dialog observation. T-039 and T-040 remain aggregate `NOT RUN` in the ledger where their full procedures require installed owner-HWND interaction; T-031 also retains physical power-loss work.

## Retained failed aggregate attempt

The first post-freeze wrapper attempts, outputs `dcde1c` and `d1f7e6`, are retained as **FAIL at the wrapper provenance gate**: the repository ownership check rejected Git access before a process-local exact-checkout `safe.directory` argument was supplied. Their 100 native tests happened to pass, but the aggregate cannot be relabeled PASS because its provenance precondition failed. No global safe-directory entry or other protection change was made. The corrected complete aggregate is `9f332a` above.

## Engineering build and pre-delivery boundary

Fresh GitHub PR run [36894176872](https://github.com/comzulca/Aster-desktop/actions/runs/36894176872) checked synthetic merge `f0c432f4f882e065932fb453d79663f1ee60058b`, as shown by source-policy logs. Git API verification confirmed that merge has exact tree `24e48836a421e099c47d94593b49409749096088`, identical to local source `c718e2c`. At this checkpoint the run was still in progress; tree equality and partial job success are not an aggregate hosted PASS and no installer identity is inferred. Internal test readiness additionally requires all required hosted jobs, schema-2/SBOM outputs, the five-entry unsigned package and independent verifier extraction/hash of both bundle resources.

After internal artifact verification and before final delivery, execute every feasible manual row on this current primary Windows 11 laptop under the owner's existing principal account only; do not create/use a guest/new account. This includes normal install/launch, credential/native-review/secret-boundary checks, accessibility, restart/no-history, feasible T-041 observations, uninstall and the two separately authorized live plain-text canaries. The shipping app has no fake-provider or proposal-injection diagnostic; installed steps requiring those test-only seams remain `NOT RUN` and are supported only by exact-source fixtures unless a separately reviewed diagnostic is added. Existing-file replacement remains proposal-only with `applyExistingFile=false`; safe publication proof remains unresolved. Issues #11/#12 therefore remain open.

## Evidence fields

| Field | Current value |
| --- | --- |
| Runtime source commit/tree | `c718e2c75a6c94a77020eb7e047c07503945b388` / `24e48836a421e099c47d94593b49409749096088` |
| Native command window/output | `2026-10-01T16:46:46.2806428Z`–`16:46:49.2001051Z`; `9f332a`; PASS, 100 library/0 main/0 documentation failures |
| Frontend command window/output | `2026-10-01T16:44:37.8958197Z`–`16:44:48.6676378Z`; `a031df`, `b54ee4`; PASS, 25/25 |
| Policy/compatibility/notice-self-test/audit window/output | `2026-10-01T16:47:20.2940425Z`–`16:47:21.6228521Z`; `72d5cb`; PASS |
| Canonical pre-freeze notice output | `6f7658`; 4,577,083 bytes; SHA-256 `b181b1e7d4aa59018bd23a828112a72adbc9a2b8d76764132159ba924bcb1c92`; hosted reproduction pending |
| Retained failed attempt | `dcde1c`, `d1f7e6`; FAIL at wrapper provenance gate, superseded only by the complete corrected aggregate |
| Hosted workflow run | `36894176872`; synthetic PR merge `f0c432f4f882e065932fb453d79663f1ee60058b`; tree `24e48836a421e099c47d94593b49409749096088` verified identical; IN PROGRESS, not credited as PASS |
| Installer ZIP/manifest/setup hashes | NOT RUN at this checkpoint |
| Owner-laptop acceptance and live calls | NOT RUN |

Do not silently replace a pending/NOT RUN field with evidence from historical run 85 or any earlier candidate.
