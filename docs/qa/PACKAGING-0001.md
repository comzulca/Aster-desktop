# PACKAGING-0001 — unsigned engineering installer gate

Current status, **2026-10-02: PASS for the delivered final protocol-2 unsigned engineering package**. Exact main source `1782d3375db47466f89be6bd3cd3c6affbfa95b7`, run `37040168700` attempt 1, passed all four jobs; artifact `11243338033` passed independent five-entry ZIP/manifest/NotSigned setup/bounded NSIS payload/resource/application/delivery-alias verification. The setup and byte-identical `Aster_0.1.0_x64-setup.exe` alias are 19,017,189 bytes, SHA-256 `0357a0798714922dd6bc6dc612eb6ed0a5c9c6224197328e1f186112c0256bcb`. [OWNER-ACCEPTANCE-0001](OWNER-ACCEPTANCE-0001.md) records full identities and six owner-reported category PASS results dated 2026-10-02 Edinburgh/BST, with non-elevated Aster in the existing administrator account. Unreported detail is explicitly transferred to [v0.2.0](V0.2.0-FOLLOW-UP-0001.md). PR artifacts remain non-installable candidates. Signing, legal clearance and production promotion are not claimed; no new installer is built by this documentation update.

## Historical 2026-10-01 — retained NSIS metadata failure and bounded remedy

PR [run 36905811883, job 110525864621](https://github.com/comzulca/Aster-desktop/actions/runs/36905811883/job/110525864621) (run number 89) is a retained **FAIL**, not an installer-build failure. The hosted job completed the NSIS build in 13 minutes 30 seconds at 18:57:42 UTC and produced a reported 3.61 MiB setup executable. At 18:57:43 UTC, the next fail-closed member-size preflight rejected the 7-Zip technical listing because one nonempty member had no numeric `Size`. Main run number 90 had all three prerequisite code jobs passing while its packaging job was still **IN PROGRESS** at the evidence checkpoint, but it contained the same pre-remediation packaging logic; no final-package PASS or artifact claim may be inferred from that partial state. No installer from either run was installed or executed.

The missing field is a format/handler limitation, not evidence that the member exceeded its bound. Tauri's supported [`bundle.windows.nsis.compression`](https://v2.tauri.app/reference/config/#nsisconfig) option defaults to `lzma`; its exact [CLI 2.12.0 NSIS template](https://raw.githubusercontent.com/tauri-apps/tauri/tauri-cli-v2.12.0/crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi) emits `SetCompressor /SOLID` for `lzma`, `zlib` and `bzip2`, but emits [`SetCompress off`](https://nsis.sourceforge.io/Reference/SetCompress) for `none`. The upstream 7-Zip [NSIS handler](https://raw.githubusercontent.com/ip7z/7zip/main/CPP/7zip/Archive/Nsis/NsisHandler.cpp) omits `kpidSize` when it has neither a defined item size nor a usable solid-stream estimate. Its [NSIS parser](https://raw.githubusercontent.com/ip7z/7zip/main/CPP/7zip/Archive/Nsis/NsisIn.cpp) defines the item size directly for a non-solid, uncompressed record.

A native NSIS 3.11 fixture run from 19:07:31.9640734 through 19:07:32.6251906 UTC (`9121a0`) reproduced that boundary. The solid fixture was 39,394 bytes, SHA-256 `2c26bd160c7a8290b4a0d3677c3ee77b5280bbfd9723d89368921add0bb9c0a6`, and the unchanged strict parser denied it because the final `uninstall.exe` size was blank. The `SetCompress off` fixture was 47,930 bytes, SHA-256 `38c18ae9cfde129aa569b510c0c33e517ab6d8061236c8300a5359ed6f203110`, and the same parser passed its two declared entries with 1,390 advertised bytes. This proves the proposed format/parser compatibility for the fixture only; it is not proof of an Aster artifact.

Advertised NSIS metadata is also not expanded-byte evidence. In the uncompressed fixture, 7-Zip advertised 784 bytes for `uninstall.exe`, while a bounded standard-output extraction measured 42,288 actual bytes (`db5e8a`, 19:09:14 UTC). A separate synthetic filesystem extraction/hash attempt ended with `AccessDenied` (`687c76`, 19:12:03 UTC); that **FAIL** is retained and no security-policy bypass was attempted. The fixture executable was inspected as archive data only and was not run.

The accepted narrow remediation for the next exact source revision is `bundle.windows.nsis.compression: "none"`. Keep the existing pre-extraction requirements unchanged: every advertised member must have a numeric size, each must be at most 256 MiB, the advertised aggregate must be at most 512 MiB, and all entry-count, path and link checks remain fail-closed. After extraction, independently require every actual regular file to be at most 256 MiB and the actual aggregate to be at most 512 MiB before hashes or package classification are accepted. Record advertised and actual-expanded totals as distinct evidence. This avoids a custom NSIS template and does not weaken the parser; its cost is a larger installer and new artifact hashes. A fresh exact-source main run and a successful independent verifier against its real five-entry ZIP remain **NOT RUN**.

The product identity remains `Aster Desktop`. Tauri's manifest artifact is therefore `Aster Desktop_0.1.0_x64-setup.exe`. The requested `Aster_0.1.0_x64-setup.exe` may be supplied only as a byte-for-byte delivery alias. Record both filenames, byte counts and SHA-256 values; they must be identical in content, while the manifest continues to identify the original Tauri filename.

Both the Windows-host and packaging jobs regenerate schema-2 notices; local development packaging must do so before Tauri bundling. Pass both complete `cargo metadata` and a second `--filter-platform x86_64-pc-windows-msvc` metadata file to `third_party_notices.py`, followed by that target triple. The inventory distinguishes all installed source packages from `requiredForPackagedApp`: the target Cargo normal-dependency closure plus production npm graph. It fails unless the required subset has zero missing texts and zero unknown license declarations. This is not an exact binary SBOM or a legal decision.

The installer must retain proprietary `LICENSE.txt` and generated `THIRD-PARTY-NOTICES.txt` at its application root. Exact-source schema 2 contains 575 total/291 required, 45 direct gaps, 10 peer-covered, four overlay-covered (one canonical), 31 non-required missing and zero required missing/unknown. `2bcacd7` generated 4,577,083 notice bytes, SHA-256 `45c54b9a9c2ff83296ec2357be44dbb973c8109fa60963cc87754fe842a803ca`. The final main job must reproduce its own inputs, extract the actual NSIS with 7-Zip, require exactly one notice/license resource and bind those hashes plus supplemental evidence. Structural closure is not exact binary SBOM or legal approval.

The uploaded packaging ZIP must contain exactly five non-link entries and nothing else:

1. `manifest.json`
2. `Aster Desktop_0.1.0_x64-setup.exe`
3. `THIRD-PARTY-NOTICES.txt`
4. `third-party-notice-inventory.json`
5. `installer-payload-list.txt`

The manifest binds source/run/attempt, executable bytes/hash/`NotSigned`, SHA-256 for all three supplemental evidence files and exact hashes for the two installed-root resources. On the owner laptop, first make `7z.exe` available without changing security policy, then run `scripts/qa/Test-EngineeringInstallerArtifact.ps1` with the expected GitHub ZIP digest/source/run/attempt and a new absolute destination. It hashes through one read handle, rejects duplicate/unexpected/link/oversized entries, validates schema-2 counts/provenance and zero unresolved required packages, verifies supplemental hashes, materializes without execution, rechecks Authenticode, independently extracts the NSIS to `verified-installer-payload` and compares both bundle-resource hashes. Its success classification is `VERIFIED_NOT_INSTALLED`; installation remains separate.

At that historical checkpoint, fresh exact-main CI and five-entry/extracted-payload verification were still required. They subsequently passed for the final `1782d337` build above. The owner's 2026-10-02 category-level acceptance and explicit v0.2.0 transfer supersede the then-pending manual scope only; the two disclosed off-Windows warnings, unavailable seams, granular missing records and historical FAILs remain visible. No legal, signing or production decision is inferred.

## Historical verified artifacts

The latest historical baseline is source `b0610c534fad0348486fb7cdd87774a9a2e81978`, [workflow run 36873066753](https://github.com/comzulca/Aster-desktop/actions/runs/36873066753), run number 85. Its four jobs passed, but it predates protocol 2 and is not a current delivery:

| Item | Historical identity/result |
| --- | --- |
| Installer artifact ID | `11169203231` |
| Downloaded ZIP | 3,590,406 bytes; SHA-256 `9049f2e611d076ab71f3e64de253851978391bbdad633d4c43187d567afedcec` |
| Executable | `Aster Desktop_0.1.0_x64-setup.exe`, 3,608,012 bytes |
| Executable SHA-256 | `f305e691ea498a1e1321f3710619b80ebee0c6a5323dc3c5d0538ff45e01e162` |
| Manifest SHA-256 | `924b2bc1d5fee7ef70f8b0df4764a49de95cdabfb73f68806bec9bbaab5b85bb` |
| Authenticode | `NotSigned` |
| Artifact retention | Recorded expiry 2026-10-15; availability must be rechecked |

The older 2026-09-30 run-58 record below remains historical as well.

### Historical run 58 — 2026-09-30 UTC

Packaging and artifact integrity **PASS** for source `31a82bf50fc46ea4fbb91c50c59cc02a65da4c71` in [workflow run 36190804857](https://github.com/comzulca/Aster-desktop/actions/runs/36190804857), run number 58. Windows 11 install/uninstall, native UI and live-provider acceptance were **NOT RUN**. On this date the owner chose an unsigned engineering build.

After the `source-policy`, `frontend`, and `windows-host` jobs all pass on the same `main` commit, the `engineering-installer` job on hosted Windows Server 2025 may build one NSIS setup executable using Node 24 and the pinned Rust 1.97.0 inputs. `package.json` requires Node `>=24.15.0 <25` and pnpm 11.19.0; this run observed Node 24.21.0. It runs the Tauri CLI with `--ci --no-sign`, rejects changed lockfiles and an unexpected executable count, verifies that Authenticode reports `NotSigned`, then uploads the executable and a JSON manifest with exact source commit, workflow run/attempt, filename, byte count and SHA-256. Its private GitHub Actions artifact has 14-day retention; check current availability before use. The ZIP-level artifact digest and executable SHA-256 refer to different bytes and must never be interchanged.

## Historical run-58 artifact

| Item | Recorded identity/result |
| --- | --- |
| Source and hosted verification | `31a82bf50fc46ea4fbb91c50c59cc02a65da4c71`; all four jobs **PASS**, 9/9 frontend and 28/28 Rust tests |
| Installer artifact ID | `10888958678` in run `36190804857` |
| Downloaded installer ZIP SHA-256 | `c6e7d67a50d438f5b2979c6a33e5fbf69ae3a20ae01e968a79baec3893880d9f` |
| Executable | `Aster Desktop_0.1.0_x64-setup.exe`, 3,527,015 bytes |
| Executable SHA-256 | `8175407e55838319b3553c9d6e577b5858a67bc0413903aa7178d2522f5df19e` |
| Authenticode/classification | `NotSigned` / `unsigned-engineering-candidate` |
| Scope | Packaging/integrity of this binary, not Windows 11 acceptance, public trust or production promotion |

The ZIP digest and executable hash were independently checked against the artifact/manifest on 2026-09-30. The prior `e35dcf7` artifact and its hashes remain historical evidence in [CI-0005.md](CI-0005.md); they cannot identify this newer binary. A later source revision, including capability changes for issue #12, requires its own successful run and newly verified artifact. Do not attach this older binary's PASS to changed runtime code.

Before using an artifact on a Windows 11 laptop, verify the GitHub workflow and manifest refer to the desired source revision; hash the downloaded executable locally with `Get-FileHash -Algorithm SHA256` and compare it with `sha256` in the manifest. Check the original JPG hash from `README.md` in source. Use [WINDOWS11-ACCEPTANCE-0001.md](WINDOWS11-ACCEPTANCE-0001.md) to record exact commands, UTC timestamps and outcomes. This installer is explicitly unsigned; no claim is made about SmartScreen/ESET acceptance, installed behavior, Windows Credential Manager on the user's account, native file mutation, either NVIDIA live model or bit-for-bit reproducible bundling. Keep Windows and security software protections active. Use the owner's existing administrator account, launch the app normally without elevation, and allow installer UAC only if required. Do not create a guest/new user, install a certificate, grant trust, or change security settings for these tests. A signed production installer, if separately requested later, needs signing custody, final binary identity, Windows 11 install/uninstall and accessibility tests, and a promotion decision recorded against that exact binary.

## Historical design state

At design time packaging was **NOT RUN** and Node 22 was the configured input. This was superseded by the dated CI records and Node 24 dependency/toolchain update above. Historical failures and passes remain in `CI-0001.md` through `CI-0006.md`; no old result is rewritten as a result for the current source.
