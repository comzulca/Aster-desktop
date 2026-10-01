# LOCAL-0005 — protocol-2 working-tree validation

Observed 2026-10-01 UTC. Current classification: **latest unpinned Rust format/Clippy/test observation PASS; complete frozen-source aggregate pending**. An initial 100-test observation completed before `16:01:33Z` (native tool output `60695b`). Independent review then enforced an explicit 32-KiB project-composer boundary with exact-limit/+1 coverage and pending-state revocation on early project-send guard/compose/reservation errors. A complete post-fix native rerun passed from `2026-10-01T16:11:08.9529732Z` through `2026-10-01T16:11:12.9313913Z` (output `ec32f1`). Other rows below were observed before the last Rust edits and must also be repeated after source freeze. No observation may be attached to a later commit by assertion.

## Environments and authority

- Native Windows execution: `QUARK2\Rene Nehru Alvarez`, Windows NT `10.0.26300.0`, normal existing owner environment. No Windows or ESET protection was disabled and no live provider request or real-credential read was part of this run.
- Frontend execution: `QUARK2\CodexSandboxOffline` on the same Windows version, bundled Node `24.19` and pnpm `11.19`, with live provider access disabled.
- Source under test: the integrated protocol-2 working tree. It was still changed/uncommitted at this checkpoint, so a Git commit/tree, raw-blob manifest and exact-source rerun remain required before these observations can identify release bytes.

## Earlier integrated working-tree results

| Area | Local observation |
| --- | --- |
| Rust quality | Before the final two Rust edits, `cargo fmt` check, locked `cargo check` and locked all-target Clippy with warnings denied: **PASS**. The later scoped rerun is recorded below. |
| Rust suite | **100/100 library tests PASS**, with empty main and documentation harnesses passing. The suite includes exact reviewed-body dispatch, one-use/revalidation/revocation behavior, strict proposal/new-file journal behavior, independent-process termination/restart recovery and the journal-container-as-project containment regression. |
| Frontend | Frozen install, typecheck, **25/25 tests PASS** and production build PASS. Protocol-2 renderer tests include metadata-only project receipts and denial of forged receipts. |
| Source policy | **84 UTF-8 files scanned; zero likely-secret findings**. |
| Dependency compatibility | **73/73 pure T-037 fixtures PASS** and the actual configured Tauri minor-pair comparison PASS. |
| Frontend advisory scan | Exit success with zero known npm vulnerabilities in the installed dependency graph. |
| RustSec | Exit success with exactly three allowed warnings: unmaintained `proc-macro-error 1.0.4` / RUSTSEC-2024-0370; unmaintained and unsound `glib 0.18.5` / RUSTSEC-2024-0429; yanked `yoke-derive 0.8.3`. Target-graph review excludes the first two from the packaged Windows closure and retains `yoke-derive` as a build-time procedural macro; the full current Rust suite passed. Retaining this locked graph is acceptable only for the private engineering test classification, not a zero-warning, owner/legal-risk or production claim. |
| Notices | The current dual-metadata generator exited successfully: schema 2, 575 total/291 required, 45 direct missing, 10 peer-covered, four overlay-covered, 31 non-required missing, zero required missing/unknown; output 4,577,083 bytes, SHA-256 `b181b1e7d4aa59018bd23a828112a72adbc9a2b8d76764132159ba924bcb1c92`. This unpinned source-graph observation is not packaged-payload or legal evidence. |

The post-fix automatic fixtures support the code-level portions of T-038 through T-040 on the latest observed working tree, but do not transfer to later changed bytes and do not substitute for normal installed-app native dialog observation. In particular, no result here claims that the owner saw the exact disclosure in the final packaged application, that a real provider accepted it, or that the setup behaves acceptably on the test laptop.

## Post-review native rerun

The latest source retains **100/100 library tests**—the exact 32-KiB/+1 case extends an existing test rather than adding a new test function. Rust formatting, all-target Clippy with warnings denied, the full library suite and the empty main/documentation harnesses all completed with exit 0 in output `ec32f1`. This rerun covers the later project-boundary and early-revocation edits. An exact command transcript and commit/tree will be recorded with the frozen-source rerun; no `cargo check` claim is newly inferred from this output.

## Engineering handoff boundary

After the integrated bytes are frozen, rerun and record their commit/tree and exact command timestamps; do not rebind the uncommitted observation by assertion. Engineering handoff additionally requires fresh hosted CI, schema-2 notice/SBOM evidence with zero required missing/unknown entries, reproduction of the bounded private-engineering RustSec disposition, plus the five-entry unsigned package and independent verifier extraction/hash of both bundle resources. These structural checks do not approve external distribution, owner/legal risk acceptance or production suitability.

The following are deliberately **post-handoff owner-laptop acceptance**, not prerequisites that can be fabricated before delivering the engineering setup: normal Windows 11 install/launch, interactive credential/native-review/secret-boundary checks, accessibility, restart/no-history, T-041 performance observations, uninstall and the two separately authorized live plain-text canaries. Existing-file replacement remains proposal-only with `applyExistingFile=false`; safe publication proof remains unresolved. Issues #11/#12 therefore remain open.

## Evidence fields to complete after source freeze

| Field | Current value |
| --- | --- |
| Source commit/tree | UNBOUND working tree; do not attach a later commit without rerun |
| Exact UTC command window | Post-fix native rerun `2026-10-01T16:11:08.9529732Z`–`16:11:12.9313913Z`; frozen-source window PENDING |
| Raw command/output references | Initial native aggregate `60695b`; post-fix native rerun `ec32f1`; remaining exact references PENDING |
| Current post-review Rust aggregate | PASS on working tree: 100 library, 0 main, 0 documentation failures |
| Hosted workflow run/attempt | NOT RUN for this snapshot |
| Installer ZIP/manifest/setup hashes | NOT RUN for this snapshot |
| Owner-laptop acceptance and live calls | NOT RUN |

Do not silently replace a PENDING/NOT RUN field with evidence from historical run 85 or any earlier candidate.
