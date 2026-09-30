# PACKAGING-0001 — unsigned engineering installer gate

Current evidence snapshot: **2026-09-30 UTC**. Packaging and artifact integrity **PASS** for source `31a82bf50fc46ea4fbb91c50c59cc02a65da4c71` in [workflow run 36190804857](https://github.com/comzulca/Aster-desktop/actions/runs/36190804857), run number 58. Windows 11 install/uninstall, native UI and live-provider acceptance remain **NOT RUN**. On this date the owner chose an **unsigned engineering build**, not a self-signed or signed production release. Signing is outside this early-stage engineering decision, not an automatically passed production gate.

After the `source-policy`, `frontend`, and `windows-host` jobs all pass on the same `main` commit, the `engineering-installer` job on hosted Windows Server 2025 may build one NSIS setup executable using Node 24 and the pinned Rust 1.97.0 inputs. `package.json` requires Node `>=24.15.0 <25` and pnpm 11.19.0; this run observed Node 24.21.0. It runs the Tauri CLI with `--ci --no-sign`, rejects changed lockfiles and an unexpected executable count, verifies that Authenticode reports `NotSigned`, then uploads the executable and a JSON manifest with exact source commit, workflow run/attempt, filename, byte count and SHA-256. Its private GitHub Actions artifact has 14-day retention; check current availability before use. The ZIP-level artifact digest and executable SHA-256 refer to different bytes and must never be interchanged.

## Verified engineering artifact

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

At design time packaging was **NOT RUN** and Node 22 was the configured input. This was superseded by the dated CI records and Node 24 dependency/toolchain update above. Historical failures and passes remain in `CI-0001.md` through `CI-0005.md`; no old result is rewritten as a result for the current source.
