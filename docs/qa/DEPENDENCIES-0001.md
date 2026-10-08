# DEPENDENCIES-0001 — dependency and declared-license inventory

## v0.2.3 dependency change and new inventory requirement

The new development manifest pins direct `base64 = 0.22.1`, `image = 0.25.10` with default features disabled and only PNG/JPEG/GIF enabled, and Windows `windows = 0.62.2` features for in-memory PDF/stream/render APIs. Windows-native Job/token/pipe/identity features are explicit in windows-sys. These are source declarations, not a completed fresh graph audit, platform runtime PASS or legal approval. Do not transfer predecessor package counts or notice/SBOM hashes to the changed graph. No Office/LibreOffice/internal-converter dependency is added. [ADR-0019](../adr/ADR-0019.md) governs the worker/resource/remaining containment boundary; [ADR-0020](../adr/ADR-0020.md) governs media history.

Fresh locked metadata, separately filtered Windows target closure, notices self-test/generation, Rust/frontend licence and CycloneDX outputs, advisory gates and source secret guards must bind the final clean revision. Report actual warnings and required notice gaps; no zero-warning, no-network, licence distribution or account-entitlement claim follows from a pin or old exit 0. Independently bind the generated required licence/notices resources to the new NSIS payload. Original inventories and exact historical observations below remain unchanged in content and cannot qualify v0.2.3. See [LOCAL-V023-0001](LOCAL-V023-0001.md) and the [documentation index](../V0.2.3-DOCUMENTATION-INDEX.md).

**Retained dated v0.2.3 applicability — 4 October 2026, Edinburgh / Europe/London:** dated evidence below retains its original source, version and result; it does not qualify changed v0.2.3 bytes. Unchanged constraints remain subject to the [current documentation index](../V0.2.3-DOCUMENTATION-INDEX.md) and [SPEC-0005](../specs/SPEC-0005.md). The owner authorises a v0.2.3 PR and GitHub checks only, not merge, release or signing. Final-source, package, native and live qualification require independent evidence in [LOCAL-V023-0001](../qa/LOCAL-V023-0001.md). No retired provider, historical grant or old PASS is reactivated.

**Current-status navigation — 8 October 2026:** see [the reconciliation record](../qa/OCT8-RECONCILIATION-20261008.md) for actual PR/dependency/source/gate outcomes and the later owner-authorised conditional integration. This dated paragraph and the original evidence below are preserved history, not current authority, a new execution receipt or a reactivation of retired providers. Manual/native/live/production results do not transfer between sources.


**v0.2.2 applicability, 3 October 2026:** retained requirements/contracts apply only where unchanged; retired providers, disabled capabilities and older dated results are not current authority or new-source PASS. [SPEC-0004](../specs/SPEC-0004.md), [ADR-0017](../adr/ADR-0017.md) and [LOCAL-V022-0001](../qa/LOCAL-V022-0001.md) govern active zoom/startup changes and fresh evidence. Earlier evidence remains source-bound history; the later owner-reported v0.2.1 startup flash is recorded separately. The entire original v0.2.1 checkout/installer is unchanged.

**Historical applicability, reconciled 3 October 2026:** the dated record/procedures below retain their original version, identifiers, authority limits and PASS/FAIL observations. They do not qualify or authorise changed v0.2.1 source, current key activation, retired providers or disabled project capabilities. Active scope is [SPEC-0003](../specs/SPEC-0003.md) and fresh verification is [LOCAL-V021-0001](LOCAL-V021-0001.md); no old PASS transfers.

## Current engineering disposition — 2026-10-02

Final source `1782d3375db47466f89be6bd3cd3c6affbfa95b7` / main run `37040168700` attempt 1 / artifact `11243338033` passed full audit/SBOM/schema-2 notices and independent extracted NSIS-resource qualification. It reproduced the counts and notice hash recorded below. The owner reports engineering-scope acceptance PASS for internal unsigned evaluation dated 2026-10-02, Edinburgh/BST; [OWNER-ACCEPTANCE-0001](OWNER-ACCEPTANCE-0001.md) binds that decision. Detailed notice/advisory dispositions not supplied are explicitly deferred under V02-006 in [v0.2.0 follow-up](V0.2.0-FOLLOW-UP-0001.md). The two disclosed off-Windows broad-lock warnings remain; no ignore, warning-free audit, external-distribution clearance or legal/production approval is inferred.

## Retained protocol-2 inventory checkpoint — 2026-10-01 UTC

The counts/provenance below remain source-bound inventory evidence. Its then-pending final-package and owner-decision wording is historical and superseded by the current disposition above, not a request to repeat the completed final-build checks.

The schema-2 generator combines complete installed-source Cargo metadata with separately target-filtered `x86_64-pc-windows-msvc` metadata and production npm dependencies. Current exact local-lock counts are **575 installed source packages**, **291 required for the packaged-app graph**, **45 without package-owned standalone text**, **10 covered by exact installed repository/license peers**, **four covered by reviewed overlays**, and **31 with no selected text**. One of the four overlays is the canonical license-steward MPL text described below; the other three are exact pinned upstream-repository MIT text. All 31 remaining gaps are non-required: 30 other-target/build Cargo packages and npm dev dependency `saxes@6.0.0`. `requiredMissingTextCount=0` and `requiredUnknownLicenseCount=0`. The generator fails if either becomes nonzero.

Peer reuse is allowed only when normalized repository and exact declared-license identity match, with the donor recorded. Reviewed target-required gaps use pinned official WebView2 MIT texts—with the original copyright and permission notice preserved—and the canonical Mozilla MPL 2.0 text for `selectors`, each explicitly labeled as an overlay rather than repository-original and bound to its URL/VCS/hash.

The local Rust package points `license-file` to repository `LICENSE`; the SBOM generator represents it as `LicenseRef-Aster-Proprietary`, and Tauri installs it as `LICENSE.txt`. The notice output and proprietary license are then independently extracted from the built NSIS and hashed against manifest inputs. These controls are source-graph and two-resource payload evidence, not an exact runtime binary SBOM, proof that every inventoried package ships, external-distribution clearance or legal advice. Remaining missing-text packages outside the target-required closure stay visible for production legal review.

Reviewed overlays are closed to these exact identities; the generator and verifier reject any other mapping or changed VCS/text fingerprint:

| Package | Provenance | Notice bytes / SHA-256 |
| --- | --- | --- |
| `selectors 0.38.0` | `servo/stylo` commit `572ecba2d1600e7c3d490586692a209faf703baa`, path `selectors`; canonical steward text from [Mozilla MPL 2.0](https://www.mozilla.org/media/MPL/2.0/index.txt), explicitly **not** represented as repository-original. | 16,726 / `3f3d9e0024b1921b067d6f7f88deb4a60cbe7a78e76c64e3f1d7fc3b779b9d04` |
| `webview2-com 0.39.1` | `wravery/webview2-rs` commit `edc2caf886175ccaebe86078c9cfe1ae2a187328`, path `crates/webview2-com`; exact upstream root MIT file. | 1,067 / `0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73` |
| `webview2-com-macros 0.8.1` | `wravery/webview2-rs` commit `dffa41a8a46d3f5565eefbff2de57d38d399f158`, path `crates/callback-macros`; exact upstream root MIT file. | 1,067 / `0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73` |
| `webview2-com-sys 0.39.1` | `wravery/webview2-rs` commit `edc2caf886175ccaebe86078c9cfe1ae2a187328`, path `crates/bindings`; exact upstream root MIT file. | 1,067 / `0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73` |

The authoritative machine-readable bindings are `scripts/ci/notices/manifest.json` and the generator's compiled expected mapping. Changing a loose text file or manifest alone cannot authorize another overlay. Exact-source offline generation at `2bcacd7c6f2496eda233d4a069ec7b65a93d47b3` produced `THIRD-PARTY-NOTICES.txt` at **4,577,083 bytes**, SHA-256 `45c54b9a9c2ff83296ec2357be44dbb973c8109fa60963cc87754fe842a803ca`. Schema 2 records 575 total packages, 291 Windows-required, 45 direct missing texts, 10 peer-covered, four overlay-covered, one canonical-steward text, 31 non-required missing, zero required missing and zero required `UNKNOWN`. The final hosted package job must reproduce the output; this is not extracted installer-payload evidence or legal approval.

The complete exact-source RustSec scan observed exactly two warnings in the broad lock graph: `proc-macro-error 1.0.4` and `glib 0.18.5`; the frontend audit reported no known vulnerabilities. The 311-package Windows-target metadata excludes both and includes non-yanked `yoke-derive 0.8.4`. No ignore or owner risk acceptance exists. For this **private internal engineering test only**, retain the locked graph and disclose those facts instead of claiming zero warnings. This is the engineering-builder disposition, not an owner/legal risk acceptance, production approval or external-distribution decision. The final packaging/evidence source still needs complete main-branch CI SBOM/audit/schema-2 outputs and independently extracted NSIS-resource evidence. Do not inherit the historical inventory below or treat generated notices/audit exit 0 as legal or risk approval.

## Historical inventory — source `e35dcf7`

Reviewed 2026-09-24 UTC against private `comzulca/Aster-desktop` source commit `e35dcf743831875c782d874313d1108919502cc4` and Git tree `e4cdf33858ed7bd4c23e2bde3b5d4a37f055df68`. This is a read-only inspection of the frontend and Windows host evidence from [workflow run 36043069614](https://github.com/comzulca/Aster-desktop/actions/runs/36043069614), plus the RustSec findings recorded in [CI-0005.md](CI-0005.md). It is a package-declaration inventory, not a legal opinion or approval to distribute an installer.

| Evidence source | Artifact ID | Independently downloaded ZIP | Inventory result |
| --- | --- | --- | --- |
| `frontend-evidence-36043069614-1` | `10828150808` | 36,903 bytes; SHA-256 `c1edcd6232e13bd867b013e1f26385c4dee50daef34fd930ccc5f355d0ca1595` | `frontend-licenses.json`: **107 package records** with declared licenses across seven expression groups. The CycloneDX file has 167 components; it uses a different counting basis. |
| `windows-host-evidence-36043069614-1` | `10828113704` | 71,276 bytes; SHA-256 `4b191f0e277eec221075ad3240ed7947b585829f3c67862762354e732305b97e` | `rust-licenses.json`: **476 package records**: 475 registry dependencies with declarations and the local `aster-desktop` package with `UNKNOWN`. The CycloneDX file has 475 resolved dependency components. |

The `source-revision.txt` inside **each** downloaded ZIP names exactly `e35dcf743831875c782d874313d1108919502cc4`. The ZIP digests above are of the downloaded archives, not the separate unsigned NSIS setup executable or its manifest in the engineering installer artifact. The frontend and Rust inventory files and SBOM JSON were parsed successfully; their package counts were checked from the downloaded bytes.

## License-expression screening

The frontend inventory groups its 107 package records as MIT (91), MIT-0 (1), `Apache-2.0 OR MIT` (3), Apache-2.0 (4), BSD-2-Clause (2), ISC (4), and BSD-3-Clause (2). This records declared strings only; it does not verify the upstream source or license files.

The Rust inventory includes five records declaring `MPL-2.0`: `cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`, and `selectors`. The local `aster-desktop` package declares `UNKNOWN` in this generated inventory. No package record in either inventory declares GPL, AGPL or LGPL **as its exclusive license expression**. Two versions of `r-efi` declare `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; the presence of an alternative in an expression does not by itself establish which option is used or what distribution notices are required.

The RustSec scan exited 0 but reported **seven allowed warnings**. [CI-0005.md](CI-0005.md) lists the exact advisory IDs and classifications: `glib` and `proc-macro-error` are assessed as Linux-only dependencies, while five unmaintained `unic-*` packages occur in dependencies compiled for Windows. Audit exit status must not be restated as zero advisories or full dependency clearance.

## Release boundary and open review

The CycloneDX files describe resolved dependency graphs from the locked source build. They do **not** enumerate the exact files embedded in the unsigned NSIS installer, nor prove which package assets, license texts or notices are redistributed in that binary. A component count in one inventory cannot be substituted for a component count in another.

**Legal suitability, license choice, notice obligations and exact installer-payload review are NOT RUN.** Before a release decision, the owner should inspect the package license files and actual installer payload for the selected source/artifact revision, resolve the local package's `UNKNOWN` declaration, document handling of the five `MPL-2.0` records and the `r-efi` alternatives, and review the seven RustSec warnings with dependency owners. Record the resulting decisions and notices against the exact executable hash. A later source commit, lockfile change or installer requires a new inventory and review.
