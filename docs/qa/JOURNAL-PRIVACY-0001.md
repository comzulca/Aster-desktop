# JOURNAL-PRIVACY-0001 — planned dormant native privacy fixtures

Procedure written: 2026-09-30 UTC, before behavioral implementation. Latest observation: 2026-10-01 UTC, **PASS for the repaired original-owner offline privacy fixtures, source `fc88d94b63b5d80d91a77f68796a2ba83b23a896`**; exact-byte binding and initial native FAIL remain in [LOCAL-0004.md](LOCAL-0004.md). Trace: ADR-0006 / FR-014 / SEC-011 / TM-014 / AC-027 / T-035. This is only the privacy subsection of T-031, not the whole capability gate. The steps below remain the bounded procedure, not a claim of GUI/process-kill/power-loss acceptance.

## Fixture authority and execution boundary

Run only reviewed source with locked/offline inputs on Windows 11 local NTFS, using the existing original owner administrator account with a normal/non-elevated token and ASTER_DISABLE_LIVE=1. Record source/tree, OS/build/account/token, Rust/tool versions, exact command, UTC and sanitized classifications. Use uniquely generated disposable synthetic roots and metadata. No real key, GUI, external/live network, guest/new account, policy/exclusion/registry/trust change, parent/project/repository ACL repair or privilege adjustment is authorized. Changes to fixture ACLs are limited to objects the fixture itself freshly created and owns; never name existing user objects.

Pure decoder/admission tests may run under the technical sandbox and must be labeled separately from original-owner native execution. The root will review compiler/source output before running the owner's full suite:

`cargo test --manifest-path src-tauri/Cargo.toml --locked --offline`

## Required positive and negative observations

1. Create one new host-selected private journal leaf with creation-time explicit protected current-process-user descriptor. By retained handles observe exact directory identity/owner/closed ACE policy, not merely path-based printed icacls output. Create bounded DPAPI metadata records with explicit protected record descriptors, CREATE_NEW/exclusive sharing, then validate before decryption/after flush/read-back. No project content is stored.
2. Pure bounded descriptor fixtures reject missing/NULL/empty/unprotected DACL, malformed sizes/SIDs/ACE layouts, wrong owner, broad/additional/inherited/object/callback/deny ACE, wrong access mask or object-kind inheritance flags. No privileged request, target or record write may occur on rejection.
3. A freshly fixture-created weak/inherited directory presented as an existing journal must be denied without ownership/descriptor/content change. A private-creation collision must preserve its descriptor/content unchanged. Never use a real weak user directory to obtain this evidence.
4. Change only a newly created fixture directory/record ACL to an invalid privacy state. Admission/enumeration/read/publication/recovery must deny, preserve prior bytes and never classify Verified. Restore only fixture-owned test data if needed for controlled teardown; the shipping helper itself never repairs or deletes anything.
5. Reparse/hardlink, path-versus-handle/directory identity mismatch, parent/root drift and journal-target overlap remain denied. Retain outside sentinel identity/hash and ensure lifetime pins remain held until operation completion.
6. Before durable intent, a privacy failure creates zero records/target. After intent or target creation, preserve ambiguity and block follow-up work; do not infer atomicity or rollback.

After each run record PASS/FAIL/NOT RUN per observation with exact source/fixture/UTC and evidence. Retain every failed intermediate source/command separately. Existing LOCAL-0003's 80 tests/run62/PR15 artifact cannot certify this later behavior. Full fresh quality/CI/package evidence remains required. Independent process-kill/restart/competition, normal-app native consent/disclosure/proposal/apply and power-loss remain NOT RUN; no physical-host reboot/power cut or VM provisioning is part of this procedure. All four shipping flags stay false; issue #12 remains open.
