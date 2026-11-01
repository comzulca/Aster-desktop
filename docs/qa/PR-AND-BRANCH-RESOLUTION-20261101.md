# PR and branch resolution — 1 November 2026

## Authority, baseline and intended outcome (before changes)

The owner asks to resolve the two open product PRs and then inspect and resolve problems in active branches. This authorises scoped corrections, branch synchronisation and normal PR integration after all mandatory current-source checks pass. It supersedes the 10 October PR/check-only restriction for this resolution task. It does not authorise a production release, signing, live Meta/OAuth calls, local credential/history changes, installed application execution or relaxed security controls.

Observed main is `40b938b1d0abcaa3b6ad160ed09aa8d29ae17bee`; PR #41 head is `b459be9830b26c3ba3a7c4cc0d0c9f16d0469f9c`; PR #44 head is `dd525a7bce44d86bf09d75cfd3712f945b7ce1ab`. GitHub's conflict-free synthetic merge `26c6d8703240a33098520778b21d17e7fd23e7f9` has ordered parents [main, PR #41] and tree `dfb0e4ba7d9f39a9d8ba6cb829d018b79ffba63d`. It combines the v0.2.4 engineering sources with PR #56's already-integrated security maintenance. A synthetic merge is not actual integration or new-source PASS evidence.

The source snapshot is retrieved through the authorised GitHub connection. Local textual files are checked against their Git blob SHA-1 identities. This Linux environment has no Rust toolchain or matching installed project dependencies; Windows/native/package checks must be observed on hosted CI. The snapshot is not the owner's Windows installation.

## Existing requirement and regression contract

This maintenance implements SPEC-0001 NFR-003 / SEC-009 / AC-019 and SPEC-0006 NFR-011 / NFR-012. No new product IPC, tool, credential, persistence, provider or network capability is introduced by these corrections.

PR #44's completed run `41248035057`, Windows job `124675672705`, fails all-targets compilation with E0277/E0599 in the PDF factory path. Its direct `windows-future 0.100.0` uses `windows-core 0.100.0`, whereas `windows 0.62.2` and its transitive `windows-future 0.3.2` use `windows-core 0.62.2`. Identically named Rust types from those different crates do not implement each other's traits. The complete suite and installer are SKIPPED, not PASS. Retain that failure.

Before implementation, the scoped correction is:

1. Preserve the reviewed direct `windows = '=0.62.2'` / `windows-future = '=0.3.2', default-features = false` pair and all security maintenance from current main. Do not cast between incompatible WinRT types or broaden runtime authority.
2. Exclude only the known incompatible `windows-future 0.100.x` series from scheduled Dependabot version proposals. Other dependencies and advisory checks remain enabled. A coordinated WinRT migration requires a new review and updated cohort evidence; this exclusion is not security remediation.
3. Add a standard-library-only early source-policy guard for the reviewed manifest pair and resolved root/windows/future/core dependency edges. A forged manifest, split core/future cohort, missing dependency, ambiguous duplicate or unreviewed default-feature change must fail before Windows compilation. Synthetic negative controls must cover the observed #44 failure, not merely a version string in isolation.
4. Synchronise PR #41 by a non-forced descendant commit with both old head and current main as parents, retaining all historical source and receipts. Keep the existing five mandatory jobs, exact fifteen codec/six resource probes, strict Clippy, audit, SBOM/notices and causal CPU verification.
5. Reconcile the current PR scope as partial **v0.2.4 engineering**, then observe fresh current-source checks before integration. The successful 10 October run does not qualify these changed sources. Original expired artifacts cannot be presented as a current download.

Guard acceptance: reviewed cohort PASS; real repository manifest/lock PASS; isolated incompatible direct future FAIL; split/ambiguous core and future graphs FAIL; missing root/WinRT edges FAIL; non-exact manifest pins and changed feature policy FAIL. These are policy regression fixtures, not native PDF or ABI execution proof.

## Engineering boundary and branch preservation

The existing v0.2.4 slice remains partial. Runtime GitHub mutations and full-window automatic compaction are unavailable; pure mutation/storage/capacity foundations remain test-only. Owner-installed flicker correction, native/live account/key qualification, all 42 compound obligations and unresolved GLib are separate. Successful source/package CI cannot mark them complete. Original evidence and the 6,000/10,000-ms startup bounds remain unchanged.

Inspect each active branch against its exact closed/open PR, current tip and main. Preserve unique or unreviewed work and historical failed-run evidence. An old failed run on a superseded closed branch is not a failure of current main. Do not force-push or manufacture passing checks on abandoned sources. Any retirement decision must identify its exact preserved PR/tip and distinguish merged, superseded and still-active work.

At this before-change checkpoint, new guard execution, refreshed PR CI, integration and actual-main package checks are **NOT RUN**. Record later observed outcomes in the PR conversation against their exact revisions and runs; do not commit a new source solely to record successful CI.
