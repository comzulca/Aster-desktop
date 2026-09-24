# DEPENDENCIES-0001 — dependency and declared-license inventory

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
