# Exact-source synthetic codec fixtures

This CI-only Linux crate includes the actual production `attachment_processing.rs` and its inline tests. It uses locked matching image/PNG/GIF/SHA/JPEG versions and the same explicitly locally modified JPEG path source. It does not duplicate or repair the production parser in a test shim.

The only worker symbols are panic-only stubs: entering host/worker admission is a test failure, not a simulated successful isolation. Windows PDF APIs, restricted token/Job/IO cleanup, native source identity, review, stores, provider, application and installer are not exercised or qualified here. Every input is a closed synthetic inline fixture; no owner file, key, request or store is involved. The Windows host suite remains mandatory and depends on this early check, not vice versa.

Run on the hosted Linux runner:

```sh
cargo +1.97.0 test --manifest-path scripts/qa/codec-fixtures/Cargo.toml --locked
cargo +1.97.0 test --manifest-path scripts/qa/zune-unit-fixtures/Cargo.toml --locked --lib --no-default-features --features std
```

The second command uses a test-only manifest whose library target is the **actual vendored `src/lib.rs`**, not a copied or mocked library. Its exact std-only feature declarations and `zune-core 0.5.3` match production. Dependency compilation alone would not execute internal unit tests. Both fixture manifests use their own test-only locked graph; the preserved upstream vendor `Cargo.lock` is provenance, not the selected test resolution. Only offline Cargo metadata was inspected locally; compilation/execution is not claimed. The new job and locally modified decoder need their own actual successor run. Local OS4551 and prior P1/P2/P3 failures remain preserved. An attempted external workspace-member metadata arrangement was refused because of Cargo's hierarchy rule and removed before publication; no tests were run under it. Fixture locks are not production dependencies or substitutes for the packaged application's SBOM.
