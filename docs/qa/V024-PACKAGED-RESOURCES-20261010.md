# v0.2.4 inert packaged-resource verification contract

Before-code contract recorded on 10 October 2026, 13:04:50 UTC, on developing source based on actual main `d8b9422eab83526a67bd16a23fbb0d029870d709`. This extends the [controlled integration contract](V024-INTEGRATION-CONTRACT-20261010.md), [SPEC-0006](../specs/SPEC-0006.md), [TM-0006](../security/THREAT-MODEL-0006.md) and [ADR-0021](../adr/ADR-0021.md), under the owner's narrow inert-resource rights/transfer decision. No new native or artifact PASS is asserted.

## Exact before-code resource inventory

The two existing mappings remain `../reports/THIRD-PARTY-NOTICES.txt` to `THIRD-PARTY-NOTICES.txt` and `../LICENSE` to `LICENSE.txt`. Their hashes/lengths are derived from the exact built checkout and independently verified inside the artifact. The following ten mappings are closed additions; source paths are relative to `src-tauri` and destination paths to the installer payload root. Their immutable byte identities come from the [passively inspected original ZIP-member receipt](ASTER-BUILDER-REFERENCE-20261009.md#10-october-owner-rights-and-bounded-inert-transfer).

| Source | Destination | Bytes | SHA-256 |
| --- | --- | --- | --- |
| `resources/aster-builder/SKILL.md` | `aster-builder/SKILL.md` | 7329 | `4a2eb40201f49ac4dee89cfd08852b4f764c44e770d90bee2d4dd18847773c71` |
| `resources/aster-builder/references/architecture.md` | `aster-builder/references/architecture.md` | 2240 | `f1835a20299004b83318d9ebb068a909deee02a8518af4dcecaad77b22dc7f10` |
| `resources/aster-builder/references/spec-threats.md` | `aster-builder/references/spec-threats.md` | 1677 | `0a5c611e7508d9d13251f97a682afa4fc8a382c8966b511f5b0d537ef3ab3cfd` |
| `resources/aster-builder/references/provider-contracts.md` | `aster-builder/references/provider-contracts.md` | 1655 | `675b3443e9005a559d4d5d719121aac3cbd826929c1508646db8f98577b6f294` |
| `resources/aster-builder/references/storage-tools.md` | `aster-builder/references/storage-tools.md` | 1481 | `46b01b2fb5dd3a8adb00cafad6d71e92e188dfd30c6e40bcf6fdb8ea24db9ac9` |
| `resources/aster-builder/references/ui.md` | `aster-builder/references/ui.md` | 1613 | `c22cfa4a62ddf334f3b3b9a24f46b5d755c4cddcb4204d758c0e548e9548720a` |
| `resources/aster-builder/references/verification.md` | `aster-builder/references/verification.md` | 2256 | `ef0e3c086f350c2b9ef629f087c4a80f66de308f7efd97fb5ca61ec1f4a534bf` |
| `resources/aster-builder/references/aster-case-study.md` | `aster-builder/references/aster-case-study.md` | 1160 | `31e868a022e71c84fa2bd0418a2ff621c597fc576bdd056fc90be303afadaf3a` |
| `resources/aster-builder/references/ledger-schema.md` | `aster-builder/references/ledger-schema.md` | 4172 | `d322712e3d4614827248aceed46ff82ac126501e60f805ba94a68a86191dd591` |
| `resources/aster-builder/assets/icon.svg` | `aster-builder/assets/icon.svg` | 1738 | `8936b24f674a1250d999e2c2b5901374fb4240426f32bd1ae340c352a90d617d` |

## Required manifest and passive-validation behaviour

Before changing `.github/workflows/ci.yml` or the active `scripts/qa/Test-EngineeringInstallerArtifact.ps1`, require the exact twelve config mappings above, with no glob, executable, alternative prefix or implicit recursive acceptance. The existing root-relative notice/LICENSE extraction in the qualified v0.2.3 listing is historical path evidence, not proof that a new artifact contains the new resources.

CI must compare the ten source byte identities to this fixed inventory, then verify exact root-relative member names and sizes before extraction and exact lengths/hashes after bounded extraction. The entire `aster-builder` file namespace is closed: missing, duplicate, case-aliased, nested-prefix, unknown or executable members fail closed before extraction. Preserve the existing global 8,192-member, 256-MiB/member and 512-MiB/aggregate preflight/extraction limits; no resource expansion raises them. Existing notices/LICENSE remain exactly named root-relative resources and retain independent generated/source hash comparisons.

The package manifest records exactly twelve `bundle_resource_sha256` properties and twelve corresponding `bundle_resource_byte_count` properties. `installer-payload-list.txt` records `required_resource_count=12`, `aster_builder_resource_count=10` and one exact hash/length line per required resource, before the raw listing. The independent active passive verifier carries the fixed ten-path byte inventory itself: an untrusted manifest/header cannot authorise a different skill, path or hash. It independently rechecks the actual installer member inventory and extracted bytes; no source-tree dependency or installer/application launch is required for that verification.

The artifact ZIP still contains exactly five entries. Preserve all five CI jobs, exactly fifteen codec and six native resource probe cases, their original budgets, causal CPU proof, `qualificationComplete=false`, actual-main provenance, unsigned status and frozen historical evidence. The ten inert files are packaging checks, not ten new native probes and not an inferred native six-case PASS. Synthetic in-memory/static fixtures may exercise the closed inventory/manifest validators without 7z, a native executable, network, credentials or owner history. A passing fixture does not qualify the changed installer; actual CI/installer/native/owner observations remain NOT RUN until separately recorded for their exact source.

The resource checks grant no runtime resource-read/export IPC, tool, shell, MCP or GitHub authority. The scripts, installer, Codex agent configuration and old JPG excluded in the provenance receipt remain excluded. No app release, signing, integration or general redistribution authority is added.

## Execution receipt

Implementation, static fixtures and new-artifact verification: **NOT RUN** at this before-code checkpoint. Preserve this statement as dated history when adding a later source-bound receipt.

### Later narrow source and static receipt — 13:12 UTC

The CI installer-packaging block and active passive verifier now implement the twelve-name hash/length maps, exact config-source/destination validation, ten fixed original Builder byte identities, pre-extraction closed member inventory and post-extraction lengths/hashes. The existing 16-KiB passive manifest cap is also enforced by CI after serialization. ZIP shape/provenance and native probe counts/budgets are unchanged. No generic glob, resource-root prefix, script entry or capability grant was introduced.

At `2026-10-10T13:12:22.5632014Z`, from the uncommitted working source based on main above, the exact command `& './scripts/qa/Test-AsterBuilderResourceContract.ps1'` exited 0 on PowerShell **7.6.5**. Its **32 static/synthetic cases PASS** exercise the three reviewed pure verifier definitions only, including JSON roundtrip, all ten missing-member cases, duplicates/case aliases, unlisted executable/nested members, original hash/size substitutions, non-exact manifest properties, invalid length types and duplicate/forged header lines. The fixture independently matched all ten source files (25,321 original bytes in total), twelve exact config mappings, both fixed inventory copies and identical CI/passive member-validator definitions. It does not execute the verifier's archive/materialisation/native top-level code. PowerShell 5.1 execution has not been observed merely because the scripts declare that minimum.

| Exact checked source | SHA-256 |
| --- | --- |
| `.github/workflows/ci.yml` | `c3c54aac1ec06c082157fb33fbda3b39979db055c132802bd81b9702798a80cd` |
| `scripts/qa/Test-EngineeringInstallerArtifact.ps1` | `f385fa6db10e9f64c23f9fd0296d46454bc473e9125f38a18b21658461e3d2cc` |
| `scripts/qa/Test-AsterBuilderResourceContract.ps1` | `c47a341e16e316930400b7bcef1806356b02c96dbda1277de2fc323c00643005` |

At `2026-10-10T13:12:23.2902600Z`, an independent read-only baseline comparison found every workflow line before the installer packaging block and the entire artifact-upload/failure-preservation tail identical to main `d8b9422eab83526a67bd16a23fbb0d029870d709`. Thus all five jobs, the existing fifteen-codec/six-native-resource receipt checks, causal CPU proof and budgets are preserved exactly outside this narrow package block. Both edited PowerShell bodies and the new fixture parsed with zero syntax errors. This is static source correspondence, not execution of those jobs/probes.

Changed-installer/7z/extracted-payload verification, all native resource probes, final CI and owner-installed/live/manual acceptance remain **NOT RUN** here. No app, installer, supplied script, native executable under verification, network, credential or owner-history operation was launched. `qualificationComplete=false`; all 42 compound obligations and unresolved GLib remain separate. A later source edit invalidates these hashes and requires a fresh fixture/artifact receipt.

### Successor alias-denial receipt — 13:13:57 UTC

The earlier 32-case result remains dated source-bound history. The same before-code alias-denial intent now explicitly detects trailing-dot/space Builder-prefix aliases and a trailing-dot existing-license alias before extraction, rather than waiting for post-extraction identity checks. At `2026-10-10T13:13:57.4012246Z`, the same pure fixture command on PowerShell 7.6.5 exited 0 with **35 cases PASS**, including those three new negative cases. No native/artifact result or capability status changed.

The successor exact source hashes are workflow `ee79bd0b0d355f5f6de60d71beae44c993122b3a49d6eca411b737fe1187f5cc`, active passive verifier `7f1fef1167ef7d80764174abe84309ee78d8536814237de5cfa690ee8a8601e7` and pure fixture `8b946e637070667233d507215e277b21c99a3919cf9596e055d372fd10040451`. Its ten original file identities/twelve mappings and equal CI/passive validator definitions remain checked. The full native jobs/probes, independent actual-artifact verification, owner acceptance and all prior residuals remain separate and NOT RUN by this receipt.

### Before-code checkout byte-preservation refinement

A later passive review observes `core.autocrlf=true` from the local Git system configuration and the original `SKILL.md` has 52 LF line endings, zero CRLF sequences and the exact 7,329-byte original SHA above. Without explicit attributes, a Windows checkout could rewrite original inert-resource bytes and correctly fail the fixed-identity gate. Before changing `.gitattributes`, require **ten exact `-text` entries**, one for each `src-tauri/` source path in the table above, with no glob, global line-ending change, rewritten member, relaxed SHA comparison or change to existing notice/vendor rules. This metadata preserves original bytes; it is not a provenance or licence substitute. At this refinement checkpoint the root agent is notified; the attributes have not yet been changed or qualified by this worker.

### Before-code notice-inventory scope clarification

The documentation cross-review finds a stale generator metadata string in `scripts/ci/third_party_notices.py`: `payloadEvidenceScope` names "the two declared" resources. Before code, permit changing only that explanatory scalar to say the source notice inventory itself proves no installer payload and requires separate exact manifest-bound CI/passive verification of the declared resources. Do not change schema 2, package selection, hashes, overlays, licence conclusions, gate counts or any native check. Preserve earlier generated JSON snapshots; regenerate a distinctly named successor from the same exact metadata. The twelve-resource manifest/byte checks above remain the actual new-artifact contract, not a PASS inferred from notice generation.
