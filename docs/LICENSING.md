# Licensing and redistribution

## COMZULCA's Apache 2.0 grant

Effective 2 November 2026, and at the copyright owner's express direction, COMZULCA licenses its original material in this repository under the Apache License, Version 2.0. This includes COMZULCA-owned source, documentation and supplied artwork, and COMZULCA-owned contributions retained in earlier Git revisions. The complete, unmodified terms are in [LICENSE](../LICENSE); the project attribution is in [NOTICE](../NOTICE).

The earlier Aster proprietary licence is superseded for that COMZULCA-owned material by this grant. Old commits, dated engineering receipts and old artefacts retain their original bytes as historical evidence. They are not silently rebuilt, re-signed or requalified. The grant does not transfer ownership of third-party material or change third-party terms.

Use, modification, redistribution and commercial use are permitted without COMZULCA licence fees or separate commercial permission, subject to Apache 2.0. Redistributors must supply the licence, identify modified files and retain applicable attribution and NOTICE information. Apache 2.0's patent and trademark provisions, warranty disclaimer and liability limits continue to apply. The [COMZULCA disclaimer](../COMZULCA_Disclaimer_EN.md) supplements the licence without adding restrictions or excluding liability that cannot lawfully be excluded.

This repository's change applies to Aster Desktop. It does not alter files or settings in other COMZULCA repositories. Access to AI models, GitHub and other third-party services remains governed by their own terms, availability and fees.

## Dependencies retain their licences

| Material or declaration | Treatment |
| --- | --- |
| COMZULCA's npm application and original Rust packages | `Apache-2.0`; `private: true` / `publish = false` remain accidental package-publication guards. |
| Upstream `MIT OR Apache-2.0`, or another expression offering Apache 2.0 | Apache 2.0 may be selected where the upstream terms permit; the upstream declaration, copyright and alternative licence files are preserved. |
| MIT, BSD, ISC, Zlib, Unicode and other permissive third-party material | Retain its original conditions and required notices. It is not relabelled as COMZULCA-owned Apache-only code. |
| Expressions containing `AND` | Meet all applicable licence obligations, including component-specific notices. |
| MPL-2.0 components | Keep MPL-2.0 and its file-level obligations. When distributing executable forms of covered software, provide the required source-availability information; modified covered files remain subject to MPL. |
| `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | The permissive Apache alternative is available; retaining the upstream expression does not mean selecting every alternative. |
| Vendored `zune-jpeg 0.5.15` | Keep `MIT OR Apache-2.0 OR Zlib`, all three original licence files and the modification/provenance records. The fixture package that compiles upstream source retains its matching upstream alternatives. |

The [dependency licence inventory](licensing/THIRD_PARTY_LICENSES.md) records the locked baseline's declared terms and source identities. It includes build/development dependencies and distinguishes the Windows packaged-application graph; it is not a claim that every listed package ships in the executable. Fresh inventories are generated for each exact build. No dependency version, lockfile or upstream licence is changed by this transition.

## Packaged notices

The existing Tauri resource mapping continues to package the complete root `LICENSE` as `LICENSE.txt`, together with generated `THIRD-PARTY-NOTICES.txt` and the ten original inert Aster Builder resources. The notice generator includes the complete COMZULCA `NOTICE` at the beginning of `THIRD-PARTY-NOTICES.txt`; it then retains the existing upstream notice collection and gap checks. This carries the project attribution without weakening or expanding the exact twelve-resource package contract.

Redistributors should preserve the original licence and copyright files for vendored material and distribute the generated notices with application binaries. The generator records package source identities and relevant source-access URLs. A package licence inventory is an engineering record; checking the actual distribution, modifications and applicable obligations remains the distributor's responsibility.

## Branches, history and contributions

The owner requested this transition on every active branch. Each branch receives a normal descendant commit or reviewed integration, with its own existing source and dependency graph preserved. Older proprietary wording in dated evidence is historical and is superseded only for material COMZULCA is entitled to license. Upstream licences and third-party copyright notices remain effective.

Contributions intentionally submitted for inclusion follow section 5 of Apache 2.0 unless explicitly stated otherwise or covered by a separate agreement. Contributors must have the necessary rights to their submissions.

References: [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0), [Apache licensing guidance](https://www.apache.org/licenses/), [Mozilla MPL 2.0 FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/).
