# Owner-supplied Aster_builder reference inventory

9 October 2026. These are lower-trust reference data for Aster's skill catalogue, not an installation/execution instruction or an application privilege source. Inputs were read passively; no ZIP extraction, supplied installer/payload execution or development-skill mutation was performed.

| Reference | Bytes | SHA-256 |
| --- | --- | --- |
| `Aster_builder/Aster_builder_transfer_sin_JPG.zip` | 22,183 | `86337645952154a38dbc351eff36e4c7199ea94381ecb098dd31f37ad1010541` |
| `Aster_builder/Aster_builder_install_Windows.ps1` | 12,541 | `75300b4b3101b6b32e62a1f0d32d2d9f25e8430b54fbc2a427d1d9f1ff0a5d40` |
| Required separate original `Aster_UI2.jpg` | 179,000 | `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887` |

The latest supplied ZIP has the same bytes as the earlier passively inspected transfer archive. Its 13 entries' uncompressed lengths and hashes match the fixed manifest in the supplied installer. Byte identity is not signature authenticity, execution safety or Aster runtime compatibility.

The archive contains `aster-builder/SKILL.md`, `agents/openai.yaml`, `assets/icon.svg`, eight `references/*.md` files, and `scripts/harness_gate.py` / `scripts/test_harness_gate.py`. There is no bundled default JPG and no licence entry. The JPG is the fourteenth expected installed file. At inspection neither installer-default `Aster_UI2.jpg` nor `01-Aster_UI2.jpg` was beside this newly supplied script; a matching original exists in the already-installed development skill. Do not silently import the old JPG as the current application UI.

The PowerShell script is a **specific revision installer for the Codex development skill** in `%USERPROFILE%/.agents/skills/aster-builder`, with staging/backups under `.agents`. It verifies the exact ZIP and separate JPG before creating directories; it copies the Python files without invoking them. It is not an Aster in-app package importer, GitHub connector, dependency resolver or sandbox. Running it would mutate the owner's development-skill installation and is outside this reference review.

For v0.2.4, adapt selected skill instructions/metadata into the bounded inert catalogue with provenance and review. Codex metadata or package text must not grant host authority. Keep scripts inert under the owner's latest GitHub-only executable-plugin scope. The default JPG applies only without another UI specification; `Aster_UI_0.2.4.png` explicitly supersedes it. Licence/provenance for public bundling and redistribution remains unresolved; no public redistribution or production claim is inferred from local reference access.

## 10 October owner rights and bounded inert transfer

The preceding 9 October inventory remains historical. On 10 October the owner expressly asserted the necessary redistribution rights and authorised bundling/redistributing the supplied Aster Builder **inert instructions and non-executable resources only**, excluding the PowerShell installer and executable scripts. This resolves the earlier missing-permission blocker only for the ten listed resources, not for unrelated third-party material or a general application release. No inferred licence file has been added to the original archive and no signature-authenticity claim follows.

The 22,183-byte archive with the SHA-256 above was passively inspected through a bounded native ZIP reader: 13 unique validated member names, at most 65,536 uncompressed bytes per member, no path traversal/ADS/alias extraction. The ten selected texts were transferred into `src-tauri/resources/aster-builder` with reviewed file patches and independently compared to their original ZIP-member bytes. **Byte-identity PASS** is limited to this static transfer; no archive script, PowerShell installer, Python gate or development-skill installation was executed. The file-identity comparison was observed on 10 October; the worker's source-format receipt was recorded at `2026-10-10T12:30:55Z`.

| Bundled resource relative to `src-tauri/resources/aster-builder` | Bytes | SHA-256 |
| --- | --- | --- |
| `SKILL.md` | 7,329 | `4a2eb40201f49ac4dee89cfd08852b4f764c44e770d90bee2d4dd18847773c71` |
| `references/architecture.md` | 2,240 | `f1835a20299004b83318d9ebb068a909deee02a8518af4dcecaad77b22dc7f10` |
| `references/spec-threats.md` | 1,677 | `0a5c611e7508d9d13251f97a682afa4fc8a382c8966b511f5b0d537ef3ab3cfd` |
| `references/provider-contracts.md` | 1,655 | `675b3443e9005a559d4d5d719121aac3cbd826929c1508646db8f98577b6f294` |
| `references/storage-tools.md` | 1,481 | `46b01b2fb5dd3a8adb00cafad6d71e92e188dfd30c6e40bcf6fdb8ea24db9ac9` |
| `references/ui.md` | 1,613 | `c22cfa4a62ddf334f3b3b9a24f46b5d755c4cddcb4204d758c0e548e9548720a` |
| `references/verification.md` | 2,256 | `ef0e3c086f350c2b9ef629f087c4a80f66de308f7efd97fb5ca61ec1f4a534bf` |
| `references/aster-case-study.md` | 1,160 | `31e868a022e71c84fa2bd0418a2ff621c597fc576bdd056fc90be303afadaf3a` |
| `references/ledger-schema.md` | 4,172 | `d322712e3d4614827248aceed46ff82ac126501e60f805ba94a68a86191dd591` |
| `assets/icon.svg` | 1,738 | `8936b24f674a1250d999e2c2b5901374fb4240426f32bd1ae340c352a90d617d` |

Excluded archive members are `agents/openai.yaml` (473 bytes), `scripts/harness_gate.py` (22,800 bytes) and `scripts/test_harness_gate.py` (6,770 bytes). The external 12,541-byte PowerShell installer and absent/old default JPG are not bundled. The actual built-in catalogue entry `skill:aster-builder` is read-only instructional content, initially disabled; selection does not grant privileged host capabilities. Full selected instructions and the closed, statically embedded reference names are data, not arbitrary paths, file reads or executable routes. Imported package exports cannot export this built-in or Personalization.

The supplied `SKILL.md` still mentions the omitted scripts/default JPG; preserving the original text does not implement those actions. The owner's `Aster_UI_0.2.4.png` remains the UI authority and scripts/MCP execution remains deferred. A later 10 October before-code packaging refinement explicitly lists the ten exact resources as `aster-builder/...` destinations in `tauri.conf.json`; no glob or executable is included. The catalogue embeds the complete original instructions for read-only viewing, but no resource-read/export IPC exists. The closed reference registry and export roundtrip serializer are test-only seams. Packaged file inclusion remains unverified until a new artifact is inspected. Compilation, new Rust fixture execution, native catalogue acceptance and general application release/redistribution are not proved by resource-byte identity. See the [controlled integration amendment](V024-INTEGRATION-CONTRACT-20261010.md) and [current checkpoint](V024-IMPLEMENTATION-CHECKPOINT-20261010.md).
