# Aster Desktop v0.2.0

Aster Desktop is a Windows 11 x64, local-first desktop harness in the private `comzulca/Aster-desktop` repository. The current implementation profile uses only **Meta Model API / `muse-spark-1.3-contributor`**. A Tauri/Rust host owns admission, native review, credentials, transport and protected local records; React renders untrusted text without granting authority. The model runs externally, not on this laptop.

This is an implementation-stage private engineering profile, **not an accepted v0.2.0 build or production release**. Baseline source: `35314591ff997f0a660460917d55c48423433544` (PR #27). Decisions and provider documentation were reviewed on 2 October 2026, Edinburgh / `Europe/London` (BST). See [SPEC-0002](docs/specs/SPEC-0002.md), [THREAT-MODEL-0002](docs/security/THREAT-MODEL-0002.md) and [the current transport contract](docs/providers/transport.md).

## Current profile and verification boundary

| Area | v0.2.0 contract | Evidence boundary |
| --- | --- | --- |
| Provider | One exact Contributor model and fixed Meta HTTPS Chat Completions endpoint; no alternate tier/model, selector, retry or fallback. | Dated official contract review; offline fixture implementation. Account entitlement and live canary: **NOT RUN**. |
| Text and continuation | Synthetic or rights-established public-eligible text only; every complete outgoing body is re-admitted and separately reviewed in an owner-window native dialogue. | Local classification is not universal DLP; owner acknowledgement is not independent legal/account verification. |
| History | Protected host-authored conversations retained across restarts, with a scrolling sidebar; restored incomplete requests are interrupted, not resumed. | Pure and isolated native-store fixtures, including post-review append/restart regressions, passed at the 3 October working-source checkpoint; installed Windows acceptance remains pending. |
| Project capabilities | `projectRead`, `proposals`, `applyNewFile` and `applyExistingFile` are all false. | No project/attachment/file-write invocation is admitted. Historical recovery code/evidence remains inactive. |
| Interface | British English, supplied grey Aster artwork, borderless local menus, Union Jack locale indicator, `Ready when you are.`, scrolling composer, named Send/Stop. | **38 offline frontend cases PASS** at the earlier headed 3 October checkpoint. Subsequent unsent-draft/late-receipt correction requires frontend/build rerun; installed visual/performance acceptance is NOT RUN. |
| Financial admission | £89.00 monthly ceiling; warning at £80.10 including admitted exposure; durable reservations and checked integer accounting. | Local accounting and reviewed-month/expiry regressions passed on 3 October. No account-global billing guarantee, balance reset or fabricated FX/tax; owner-native configuration remains a separate gate. |
| Release | Separate exact-source suite, dependency/security/notices/SBOM and unsigned installer/payload qualification required. | 3 October Rust suite: **142 PASS, two ignored credential fixtures**, format/check/Clippy PASS. Exact-source pinning, CI, v0.2.0 package/payload and installed acceptance remain **NOT RUN / pending**. |

These are implementation contracts, not a declaration that every gate has passed. The [3 October evidence reconciliation](docs/qa/LOCAL-V020-0001.md) records headed local suites and retains earlier checkpoints, preliminary failures and provenance limits. A later review found unsent draft loss on native denial: the corrective UI/late-receipt fixtures and affected frontend/build/package reruns are pending, not covered by the old 38-case result. Source remains uncommitted and `UNPINNED` until exact-revision evidence is recorded. A v0.1.0 PASS is never transferred to Meta, protected history, the new visual profile or a future executable.

## Contributor privacy, access and credentials

Meta designates this Contributor model as Discounted Services: submitted prompts and completions may be used for training, and personal, sensitive or confidential information—including code that must remain confidential—must not be submitted. `store:false` does not override those terms or promise zero retention. A public page can still contain prohibited personal data. Only eligible public material with the necessary rights, or synthetic examples, is admitted. [Meta Terms, updated 2 October 2026](https://dev.meta.ai/legal/terms-of-service)

The United Kingdom is not listed in the reviewed restricted-territory list, but Meta controls technical availability and account/model access. This is **not proof that the owner's account is entitled or that UK access is currently enabled**; no account dashboard, real key or live request was inspected. Never bypass geographic or security controls. [Geographic policy, updated 18 September 2026](https://dev.meta.ai/legal/geographic-use-policy)

Use a separately authorised direct Model API key, not a coding-harness subscription credential restricted to its own harness. The fixed Windows Generic Credential target is `AsterDesktop/MetaModelAPI/v1`; native syntax validation does not verify account access. Secret entry/read/delete stays in trusted Windows paths. No key belongs in chat, source control, renderer IPC, prompts, logs, environment files, CI or command arguments. Migration never reads, renames, reuses or deletes the historical provider credential. [Authentication](https://dev.meta.ai/docs/authentication), [key restrictions](https://dev.meta.ai/legal/terms-of-service)

## Protected history and money

The owner's explicit request to retain conversations between restarts supersedes the attached plan's memory-only proposal. Saved schema-1 conversations use a separate current-user DPAPI/protected-owner-DACL store: at most 128 conversations, 512 KiB per conversation, 20 completed continuation turns and a 64 MiB domain payload. Capacity exhaustion denies rather than silently truncating or evicting.

Only completed eligible turns may be restored as candidate continuation; the host re-admits every prior prompt/answer and reviews the full exact body before the next send. Restoration cannot recreate active requests, approvals, grants, credentials or financial authority. No legacy provider conversation import, browser `localStorage`, arbitrary transcript upload or history import/export is supported. Native-confirmed removal hides a conversation using a durable tombstone; encrypted recovery records and used quota remain. This is not secure erasure or physical compaction. [ADR-0015](docs/adr/ADR-0015.md)

The £89 ceiling follows calendar months in `Europe/London`. The £80.10 threshold is a 90% warning, **not another allowance**. Sending requires separately native-confirmed, dated integer GBP-per-USD FX plus safety-margin, tax and payment-charge assumptions; no automatic estimate is invented. Configuration lasts at most 31 days and cannot outlive the current dated contract policy, which expires at **2026-11-02T00:00:00Z** (2 November, Edinburgh/London). Missing, expired or ambiguous state blocks dispatch.

Reserve before network and durably mark possible dispatch; cancellation or failure does not establish a zero charge. Only a proven-unsent failure may release the reservation automatically. Missing, duplicate or impossible usage leaves the reservation unresolved, including after restart/month rollover; **any unresolved reservation blocks fresh dispatch**. A separately native-confirmed statement/dashboard amount may reconcile one exact request against its original account and quote and is labelled owner-confirmed, not API-measured. Conversation deletion, credential change or reinstall cannot reset spending. The journal cannot prevent same-user rollback or charges outside Aster. [ADR-0014](docs/adr/ADR-0014.md)

## Resource and authority limits

Policy and IPC protocol are version 3. Host limits include 32 KiB UTF-8 input, 512 KiB exact request/context, 20 complete turns, 32 admitted conversation IDs and 64 request IDs per process, one active request globally, 8,192 completion tokens including reasoning, bounded SSE and fixed deadlines. Excess is rejected rather than summarised silently.

The host sends the immutable reviewed byte allocation once. Refusal, truncation, cancellation, failure and interruption are distinct from completion; partial text may remain visible but is not automatic continuation. Token usage and financial settlement are independent of visible completion. There are no model tools, shell, browser, web/MCP, voice, image-generation, file attachments or background agents. Menus expose implemented local actions only. Determinism means reproducible admissions/transitions under the same validated input, state and versioned policy—not identical model prose, remote timing or binary bytes.

## Source checks and Windows build

Read [AGENTS.md](AGENTS.md) before changes. Required build inputs are Node 24 (`>=24.15.0 <25`), Corepack/pnpm 11.19.0, Rust 1.97.0, Microsoft C++ build tools and WebView2. Use a normal, non-elevated Windows session; leave Windows and ESET protections enabled.

```powershell
corepack pnpm install --frozen-lockfile
corepack pnpm typecheck
corepack pnpm test
corepack pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml --locked
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
```

These commands are instructions, not evidence of execution. Tests that touch a disposable native Credential Manager target remain explicitly ignored until separately authorised. Source checks do not authorise paid Meta requests, native credential changes, installation, publication or weakening protections.

Before packaging, generate target-filtered Cargo metadata, SBOM and schema-2 third-party notices using the repository scripts. Repeat current dependency/security/license/source checks and use the existing installer verifier to bind source, manifest, supplemental evidence, exact payload resources, executable hash/size and Authenticode status. Do not install or deliver intermediate PR packages. No v0.2.0 installer has been accepted at this checkpoint.

## Retained v0.1.0 engineering evidence

The earlier accepted unsigned build is historical, not the current active profile: source `1782d3375db47466f89be6bd3cd3c6affbfa95b7`, tree `c7e116d9bbba63f5b0b2ace7cf0757cb16a30b3b`; [main run 94](https://github.com/comzulca/Aster-desktop/actions/runs/37040168700), attempt 1, artifact `11243338033`. Its 100 Rust/25 frontend tests and package qualification belong only to that source.

The delivered `Aster Desktop_0.1.0_x64-setup.exe` and byte-identical `Aster_0.1.0_x64-setup.exe` alias were 19,017,189 bytes, SHA-256 `0357a0798714922dd6bc6dc612eb6ed0a5c9c6224197328e1f186112c0256bcb`, Authenticode `NotSigned`. The installed application matched extracted payload SHA-256 `f05eed42074f2e1b4b37d5d3be23b7227e0930726cac58f8cf65285f7b1bba62` at `2026-10-02T19:25:00.4822961Z`; that later identity check was not the owner's test time.

[OWNER-ACCEPTANCE-0001](docs/qa/OWNER-ACCEPTANCE-0001.md) retains all six category-level owner-reported PASS results, their scope and absence of per-case measurements. [LOCAL-0007](docs/qa/LOCAL-0007.md), [LOCAL-0006](docs/qa/LOCAL-0006.md), [PACKAGING-0001](docs/qa/PACKAGING-0001.md) and [the follow-up record](docs/qa/V0.2.0-FOLLOW-UP-0001.md) retain earlier qualification, failures, corrected packaging and deferred specifics. Historical source audits contained two broad-lock warnings, not a warning-free certification.

The [retired transport appendix](docs/providers/transport.md#retired-v010-transport-evidence) and retired provider pages preserve provenance only. Old credentials, creation/recovery journals, financial records and historical release evidence are not deleted during migration. This record neither enables retired capabilities nor proves v0.2.0 acceptance.
