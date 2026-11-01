# React runtime cohort repair — 1 November 2026

## Before-change scope and observed failure

The owner's branch-problem request also covers the newly generated failing React update. The original PRs #41/#44 and retirement of their old branches are separate completed operations. New PR #60 starts at `d2d811848c99db7026acef2d66829345ddce5104`, tree `920e5c59b0f28c9829ce20a377d8529f8723f581`, parent/current main `f629482a7907c4657c5eb0969e1f73cd2d7fe6ef`. Its run `41323162719` fails frontend job `124897531104`: eleven suites cannot import because React 19.3.1 and React DOM 19.3.0 must have exactly matching versions. Source-policy and codec fixtures pass; Windows and installer are skipped. Preserve this FAIL; it is not a native result or a reason to disable tests.

This implements existing SPEC-0001 NFR-003 / SEC-009 / AC-019 and SPEC-0006 NFR-011 through the new T-097 structural regression family. No product authority, provider, credential, IPC, history, startup timing or private-data disclosure changes.

Before dependent edits, the intended correction is to align both manifest requirements and every resolved runtime/peer edge to React/React DOM 19.3.1, keep scheduler 0.28.0, group only these two runtime packages in scheduled Dependabot updates, and add a standard-library-only early guard for their reviewed pnpm lock representation. Type packages are independent and must remain unchanged. No unrelated dependency is included.

T-097 requires an aligned cohort PASS and denial for the observed split pair, wrong peer, mismatched manifest/importer, missing/duplicate package resolution and stale snapshot dependency. This narrow structural guard is not a general YAML parser or an execution/paint test. Frozen registry installation, the full frontend suite/build/audit and all unchanged native/package gates remain required against the new head.

## Metadata and integrity provenance

The [official React DOM v19.3.1 manifest](https://github.com/react/react/blob/v19.3.1/packages/react-dom/package.json) declares React `^19.3.1` and scheduler `^0.28.0`; no new runtime dependency is needed. Direct npm registry metadata could not be retrieved in this session. The tarball SRI candidate `sha512-Y0bARz53lhEC1NMCASs0/j+faB/zqt3R8hpvg1e8+61ZVJ4UL0UdC4cBbXfZMcIuhrfgeILTLe+ZeBCavQPtKQ==` is independently identical in the published [Backlog.md lock](https://github.com/MrLesk/Backlog.md/blob/3a296c0f6f949b31e458b849b9617702c56c1023/bun.nix) and [TickerQ dependency inventory](https://github.com/Arcenox-co/TickerQ/blob/f366cf7fa32575be12e3638546d04ed6d0708c63/src/TickerQ.Dashboard/DASHBOARD-THIRD-PARTY-NOTICES.md). These are candidate integrity records, not proof of installation. Mandatory frozen pnpm installation must validate the actual registry bytes against that SRI before tests. No checksum is invented, integrity check disabled or mutable download substituted.

All new-source checks are NOT RUN at this before-change checkpoint. The ongoing actual-main engineering build at `f629482` does not include or qualify this repair. Record subsequent PR results against their exact head/run; do not transfer the earlier 177/404/package PASS. Existing unresolved upstream warnings, unavailable capabilities and owner/native/live/signing/production obligations remain separate.
