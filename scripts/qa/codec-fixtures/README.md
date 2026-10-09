<!-- OCT9 CPU AND GLIB OWNER DECISION BEGIN -->
> Owner decision and current applicability — 2026-10-09 17:14:45 UTC: PR #30 is normally merged; its distinct actual-main package run remains FAIL after two CPU-probe failures, not a qualified main installer. The [separate CPU repair contract](../../../docs/qa/CPU-CONTAINMENT-20261009.md) preserves the 10-second user-CPU and 15-second wall budgets and all mandatory gates; changed-source execution remains NOT RUN until fresh evidence. [GLib 0.18.5 remains OPEN and UNRESOLVED](../../../docs/qa/GLIB-UPSTREAM-DEFERRAL-20261009.md), deferred to a complete compatible stable upstream chain; no backport, dependency remediation, dismissal or ignore is authorised. GTK4/Tauri 3 is a future owner-preferred architecture review, not an approved migration or promised fix. Earlier bodies below are retained dated source-bound history; no historical PASS transfers to changed source or owner/manual/live/signing/production acceptance.
<!-- OCT9 CPU AND GLIB OWNER DECISION END -->

# Exact-source synthetic codec fixtures

This CI-only Linux crate includes the actual production `attachment_processing.rs` and its inline tests. It uses locked matching image/PNG/GIF/SHA/JPEG versions and the same explicitly locally modified JPEG path source. It does not duplicate or repair the production parser in a test shim.

The only worker symbols are panic-only stubs: entering host/worker admission is a test failure, not a simulated successful isolation. Windows PDF APIs, restricted token/Job/IO cleanup, native source identity, review, stores, provider, application and installer are not exercised or qualified here. Every input is a closed synthetic inline fixture; no owner file, key, request or store is involved. The Windows host suite remains mandatory and depends on this early check, not vice versa.

Run on the hosted Linux runner:

```sh
cargo +1.97.0 test --manifest-path scripts/qa/codec-fixtures/Cargo.toml --locked
cargo +1.97.0 test --manifest-path scripts/qa/zune-unit-fixtures/Cargo.toml --locked --lib --no-default-features --features std
```

The second command uses a test-only manifest whose library target is the **actual vendored `src/lib.rs`**, not a copied or mocked library. Its exact std-only feature declarations and `zune-core 0.5.3` match production. Dependency compilation alone would not execute internal unit tests. Both fixture manifests use their own test-only locked graph; the preserved upstream vendor `Cargo.lock` is provenance, not the selected test resolution. Only offline Cargo metadata was inspected locally; compilation/execution is not claimed. The new job and locally modified decoder need their own actual successor run. Local OS4551 and prior P1/P2/P3 failures remain preserved. An attempted external workspace-member metadata arrangement was refused because of Cargo's hierarchy rule and removed before publication; no tests were run under it. Fixture locks are not production dependencies or substitutes for the packaged application's SBOM.
