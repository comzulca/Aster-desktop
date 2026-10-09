# Reviewed notice overlays

**v0.2.3 applicability — 4 October 2026, Edinburgh / Europe/London:** dated evidence below retains its original source, version and result; it does not qualify changed v0.2.3 bytes. Unchanged constraints remain subject to the [current documentation index](../../../docs/V0.2.3-DOCUMENTATION-INDEX.md) and [SPEC-0005](../../../docs/specs/SPEC-0005.md). The owner authorises a v0.2.3 PR and GitHub checks only, not merge, release or signing. Final-source, package, native and live qualification require independent evidence in [LOCAL-V023-0001](../../../docs/qa/LOCAL-V023-0001.md). No retired provider, historical grant or old PASS is reactivated.

These files close a bounded packaging-evidence gap when an exact locked crate omits the standalone license file that exists at its upstream VCS root, or when its source headers direct recipients to the canonical license steward.

`manifest.json` binds every use to the package ecosystem, exact name and version, declared license, normalized repository, `.cargo_vcs_info.json` commit and path, local filename, byte count, SHA-256, origin classification, primary-source URL, and package-source access URL. The generator also contains the reviewed expected mappings; changing only this manifest cannot authorize a new overlay.

- `webview2-rs-MIT.txt` is the original root `LICENSE` from the exact `webview2-rs` commits recorded by the three locked crates. The copyright and permission notice are preserved verbatim.
- `Mozilla-MPL-2.0.txt` is the canonical MPL 2.0 text from Mozilla, the license steward. It is not represented as an original file from the Servo/Stylo repository. The locked `selectors` source headers identify MPL 2.0 and direct recipients to Mozilla when the text was not distributed with the source archive.

The source package trees remain unmodified. These overlays are engineering notice-completeness evidence only: they are not a binary SBOM, legal advice, legal approval, or production risk acceptance. The generated inventory retains the original direct-missing count, distinguishes runtime-target dependencies from build/development source inventory, and fails if a required package is unmapped or any package identity, VCS provenance, file size, or hash changes.
