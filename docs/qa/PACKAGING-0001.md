# PACKAGING-0001 — unsigned engineering installer gate

Current status: **NOT RUN for packaging of the protocol-2 candidate**. Runtime source `c718e2c75a6c94a77020eb7e047c07503945b388`, tree `24e48836a421e099c47d94593b49409749096088`, is pinned and its local post-freeze suites pass; fresh hosted run 36894176872 and the manifest-bound unsigned artifact were still incomplete at this checkpoint. No earlier executable is the deliverable for this behavior. The owner chose an **unsigned engineering build**, not a self-signed or signed production release; signing is outside this early-stage classification, not an automatically passed production gate.

The product identity remains `Aster Desktop`. Tauri's manifest artifact is therefore `Aster Desktop_0.1.0_x64-setup.exe`. The requested `Aster_0.1.0_x64-setup.exe` may be supplied only as a byte-for-byte delivery alias. Record both filenames, byte counts and SHA-256 values; they must be identical in content, while the manifest continues to identify the original Tauri filename.

Both the Windows-host and packaging jobs regenerate schema-2 notices; local development packaging must do so before Tauri bundling. Pass both complete `cargo metadata` and a second `--filter-platform x86_64-pc-windows-msvc` metadata file to `third_party_notices.py`, followed by that target triple. The inventory distinguishes all installed source packages from `requiredForPackagedApp`: the target Cargo normal-dependency closure plus production npm graph. It fails unless the required subset has zero missing texts and zero unknown license declarations. This is not an exact binary SBOM or a legal decision.

The installer must retain proprietary `LICENSE.txt` and generated `THIRD-PARTY-NOTICES.txt` at its application root. CI independently extracts the NSIS with 7-Zip, requires exactly one of each resource, compares their hashes with the source inputs and records classification `verified-extracted-nsis-payload`. The manifest binds `bundle_resource_sha256` for both resources as well as the three supplemental archive-evidence hashes. Frozen dual-metadata inputs describe 575 installed-source packages, 291 target-required, 45 lacking package-owned text, 10 covered by exact repository/license peers, four covered by reviewed overlays, and 31 still missing but non-required (30 other-target/build Cargo packages and npm dev dependency `saxes@6.0.0`); required missing and required unknown are zero. Every peer/overlay retains source/hash/provenance. Canonical pre-freeze output `6f7658` against those later-frozen inputs is 4,577,083 bytes with SHA-256 `b181b1e7d4aa59018bd23a828112a72adbc9a2b8d76764132159ba924bcb1c92`; the exact-source package job must independently reproduce it and compare it with both the archive supplement and extracted installer resource. The 31 non-required gaps remain production-distribution review, not proof that affected packages ship. Bind notices, extracted payload evidence, SBOM, audit warnings, ZIP, executable and delivery-alias hashes to this immutable source revision.

The uploaded packaging ZIP must contain exactly five non-link entries and nothing else:

1. `manifest.json`
2. `Aster Desktop_0.1.0_x64-setup.exe`
3. `THIRD-PARTY-NOTICES.txt`
4. `third-party-notice-inventory.json`
5. `installer-payload-list.txt`

The manifest binds source/run/attempt, executable bytes/hash/`NotSigned`, SHA-256 for all three supplemental evidence files and exact hashes for the two installed-root resources. On the owner laptop, first make `7z.exe` available without changing security policy, then run `scripts/qa/Test-EngineeringInstallerArtifact.ps1` with the expected GitHub ZIP digest/source/run/attempt and a new absolute destination. It hashes through one read handle, rejects duplicate/unexpected/link/oversized entries, validates schema-2 counts/provenance and zero unresolved required packages, verifies supplemental hashes, materializes without execution, rechecks Authenticode, independently extracts the NSIS to `verified-installer-payload` and compares both bundle-resource hashes. Its success classification is `VERIFIED_NOT_INSTALLED`; installation remains separate.

Once the immutable-source CI, required-notice/bounded engineering-warning disposition and five-entry plus independently extracted-payload verifier gates pass, the unsigned setup is eligible for internal testing. For this classification, `proc-macro-error` and `glib` are outside the target-required graph while `yoke-derive` is a tested build-time procedural macro; this does not assert zero warnings, approve external distribution, establish legal suitability or record owner production-risk acceptance. Before final delivery, test those exact bytes on this current primary Windows 11 laptop under the owner's existing principal account, with no guest/new account: install/launch, every feasible credential/native review, accessibility, restart/no-history, T-041 observation, uninstall and the two authorized live canaries. Preserve unavailable shipping-seam rows as NOT RUN and review every FAIL/blocker before delivery.

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
