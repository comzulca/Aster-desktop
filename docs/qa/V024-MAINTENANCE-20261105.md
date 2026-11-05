# v0.2.4 maintenance implementation checkpoint — 5 November 2026

Scope: [SPEC-0007](../specs/SPEC-0007.md), on baseline main `0448445813211783086cb7f4a44a4ec6a564da5f`. This is the pre-publication checkpoint, not a hosted/package/installed acceptance receipt. Later exact source/run/artifact results belong to the maintenance PR and delivery receipt, without transferring earlier PASS to changed source.

## Observed local verification

Linux container; Node v24.22.1, pnpm 11.25.0; project dependencies installed from the unchanged frozen lock. `pnpm test`: **178 PASS across 14 files**. `pnpm build`: TypeScript and Vite **PASS**. Before the attachment implementation, the changed regressions reproduced five failures on the inherited implementation (172 other cases passed). Cancellation/pending selection and atomic rejection then pass with the correction.

Pinned Rust 1.97.0 distribution components were downloaded from the official Rust distribution and checked against the manifest/SHA-256 before using its formatter. `rustfmt 1.9.0-stable (2d8144b788 2026-07-07)` formatting/check of the five changed Rust files **PASS**. No local Rust/native execution is claimed.

Source-policy self-tests and repository checks for secret scanning, vendored provenance, Tauri, React and WinRT cohorts **PASS**; CPU receipt verifier self-tests **PASS**. Secret scan after staging covered 320 UTF-8 files with zero findings/errors. `git diff --cached --check` **PASS**. One local guard invocation initially omitted its required `--root` argument; it was corrected and the complete remaining guard commands passed. No source or assertion was weakened to correct that command.

## Authored Rust regression scope

T-097/098: current review boundaries, unreviewed gap, legacy acceptance replay followed by fresh append, unknown/out-of-window records and rollback denial. T-099: preserved historical charges/quotes, unresolved reservation blocking after renewal, original-period reconciliation and expiry. T-101: transactional additions/cancellation, duplicate IDs, aggregate count/text/source limits, exact snapshot/conversation isolation, consumed deadlines and expired allocation non-restoration. These require fresh Windows CI execution; formatting is not their execution receipt.

## Delivery gate

At this checkpoint hosted source-policy/frontend/codec/windows/package jobs, actual fifteen codec/six resource probes and changed-source installer are **NOT RUN**. Normal integration and actual-main unsigned engineering delivery require all existing gates. No owner app/setup launch, live inference or local owner credential/history/financial mutation was performed. Owner account access, real-key/live requests, installed UI, all 42 compound obligations, unfinished runtime features, GLib and production/signing remain separately qualified.
