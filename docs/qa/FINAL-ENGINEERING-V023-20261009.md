# Final v0.2.3 engineering build and documentation reconciliation

Observed 9 October 2026, Edinburgh / Europe/London (BST, UTC+01:00). Hosted package completed at 19:18:24 UTC (20:18:24 BST); passive local verification completed at 19:38:51.7115406 UTC and alias equality at 19:39:30.1549408 UTC. Status: **verified unsigned engineering installer, not installed by the agent and not a production release**.

## Exact source and independent evidence

[PR #30](https://github.com/comzulca/Aster-desktop/pull/30) and the subsequent CPU correction [PR #39](https://github.com/comzulca/Aster-desktop/pull/39) are merged. The delivered installer is exclusively bound to actual main `54c64e404b3147170bd5d1a320e11536e9e5406e`, tree `bb6fddb393d2bd9c10c6428d0f93252eb2c00d54`, [push run 37974140580](https://github.com/comzulca/Aster-desktop/actions/runs/37974140580), attempt **1**. It is not a PR merge-ref package or a rebuilt documentation-follow-up binary. Root and an independent read-only reviewer checked GitHub's final job records/logs; root additionally verified the downloaded bytes locally.

| Mandatory job | Exact job ID | Completed UTC | Result / scope |
| --- | --- | --- | --- |
| source-policy | 113967994495 | 18:33:55 | PASS: 227 UTF-8 files/zero findings; 73 structural fixtures; aligned Tauri 2.12.1; 184 negative CPU-receipt controls. |
| frontend | 113968068899 | 18:34:33 | PASS: typecheck/build, 100 tests/eight files; moderate advisory gate successful. |
| codec-source-fixtures | 113968069005 | 18:34:34 | PASS: 14 exact-production-module and ten vendor-private decoder tests. |
| windows-host | 113968317136 | 19:03:45 | PASS: formatting, locked all-target check/Clippy, complete Rust suite 235 PASS/zero FAIL/two ignored, zero-test binary/doc targets; the two separately authorised exact synthetic native credential cases each then PASS on the disposable runner. |
| engineering-installer | 113979766891 | 19:18:24 | PASS: actual release-executable 15 codec/six resource cases, independently validated causal CPU receipt, locked/clean source, notices and bounded payload/hash/NotSigned gates. |

Hosted context is `runnervmfi6oq\runneradmin`, Windows NT 10.0.26100.0, Rust 1.97.0 (2d8144b78, 2026-07-07). This is not the owner's interactive Windows 11 session. RustSec completed successfully with two retained warnings: `proc-macro-error 1.0.4 / RUSTSEC-2024-0370` and `glib 0.18.5 / RUSTSEC-2024-0429`; no zero-warning or remediation claim. SBOM lists 484 components; the audit scans 485 locked dependencies. The notice inventory covers 583 broad packages / 301 packaged-app packages / zero required notice gaps, a structural scope result, not a legal sufficiency opinion.

Commands are the fixed steps in the [source-bound CI workflow](https://github.com/comzulca/Aster-desktop/blob/54c64e404b3147170bd5d1a320e11536e9e5406e/.github/workflows/ci.yml), including:

```text
python3 scripts/ci/scan_secrets.py --self-test
python3 scripts/ci/scan_secrets.py
python3 -I scripts/ci/verify_cpu_receipt.py --self-test
corepack pnpm typecheck
corepack pnpm test
corepack pnpm build
corepack pnpm audit --audit-level moderate
cargo +1.97.0 test --manifest-path scripts/qa/codec-fixtures/Cargo.toml --locked
cargo +1.97.0 test --manifest-path scripts/qa/zune-unit-fixtures/Cargo.toml --locked --lib --no-default-features --features std
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo check --manifest-path src-tauri/Cargo.toml --locked --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --locked
corepack pnpm exec tauri build --ci --no-sign
.\src-tauri\target\release\aster-desktop.exe --aster-attachment-fixture
python -I scripts/ci/verify_cpu_receipt.py --receipt reports/attachment-worker-fixture.json
```

The credential commands use the workflow's fixed existing exact case names with `--locked --lib <fixed-case> -- --ignored --exact --test-threads=1`. Synthetic temporary targets are cleaned on the disposable runner; the owner's laptop/production credential/Meta key are not touched.

## Real CPU containment, with earlier failures retained

Actual fixture receipt at 19:18:02.1736772 UTC: schema 1, `allPassed=true`, exactly 15 codec and six resource PASS; **`qualificationComplete=false`**. CPU mechanism `parent-user-cpu-watchdog`: trigger user CPU **100000000 × 100 ns**, trigger wall **10046 ms**, physical elapsed **10048 ms**, exact exit **2776498177**. Successful sampling, live-child/priority/exact-Job dispatch, physical expected exit, cleanup, terminal denial, marker and statistics are proved; natural exit, Job closure and fallback are false. Native termination count remains zero, not an invented native-limit result. The fixed 10-second user-CPU/15-second wall budgets remain; sampling/dispatch are not hard real time. See the preserved [CPU contract](CPU-CONTAINMENT-20261009.md).

Predecessor actual main `4e46b1bf126cd9937d68d1058ccfcc0f1c162339`, [run 37950896361](https://github.com/comzulca/Aster-desktop/actions/runs/37950896361), stays **FAIL**:

| Attempt / package job | Failure time UTC | Retained CPU outcome |
| --- | --- | --- |
| 1 / 113901326653 | 16:05:58.0083992 | FAIL: wall15029 ms, user147656250 ticks, no native CPU termination. |
| 2 / 113909777038 | 16:26:39.8807291 | FAIL: wall15025 ms, user148437500 ticks, no native CPU termination. |

Earlier P1–P11, Code Integrity/local and owner flicker records remain source-bound history; no historical failure is relabelled PASS.

## Installer and passive local verification

GitHub artifact **11641015085**, `UNSIGNED-engineering-aster-desktop-54c64e404b3147170bd5d1a320e11536e9e5406e-37974140580-1`, expires 23 October 2026 at 19:18:18 UTC. Download the exact archive from the linked run while retained; do not substitute a later artifact.

| Identity | Actual value |
| --- | --- |
| ZIP bytes / SHA-256 | 8671037 / `b0237a6b14a2d07cacf93f44a79e0029f8b342761412f5e31bcfb2441458d687` |
| Canonical setup | `Aster Desktop_0.2.3_x64-setup.exe` |
| Requested delivery alias | `Aster_0.2.3_x64-setup.exe` — independently byte-identical by size/SHA-256 |
| EXE bytes / SHA-256 | **23085677 / `16b3f1bc5eec35d1df7392ca1bf44c0b149f22cd767c01ee4e9b04a7fccf6ac4`** |
| Authenticode | `NotSigned`, independently checked locally |
| Manifest build time | 2026-10-09T19:18:12.7005555Z |
| Passive verifier | `VERIFIED_NOT_INSTALLED`; five exact archive members; ten bounded payload entries / 22990191 extracted bytes |
| Notices | `STRUCTURALLY_VERIFIED_NO_REQUIRED_GAPS` |

The archive digest is **not** the executable digest. Local output is a new `engineering-builds/Aster-0.2.3-54c64e4-run37974140580-1` directory; no old installer/source was overwritten. Root used the unchanged source-bound [artifact verifier](../../scripts/qa/Test-EngineeringInstallerArtifact.ps1) with the downloaded archive, the exact expected ZIP SHA/source/run/attempt above and a new named absolute destination. The parameter template below omits machine-specific absolute paths; the saved local delivery receipt records the actual output location:

```text
.\scripts\qa\Test-EngineeringInstallerArtifact.ps1 -ZipPath <downloaded-archive> -ExpectedZipSha256 b0237a6b14a2d07cacf93f44a79e0029f8b342761412f5e31bcfb2441458d687 -ExpectedSourceRevision 54c64e404b3147170bd5d1a320e11536e9e5406e -ExpectedRunId 37974140580 -ExpectedRunAttempt 1 -Destination <new-named-absolute-directory>
```

No setup/app/extracted executable was launched. 7-Zip listed/extracted bounded payloads only. The new alias was copied without overwrite, then both size/hash and unsigned status were checked. No Meta calls, credentials, history, financial settings, security changes or exclusions were used.

Supplemental SHA-256 identities remain bound to the manifest:

- `THIRD-PARTY-NOTICES.txt`: `be749305ac42c72e3a7893698718995816f40b9eae618dbb09eb6dd31f3ec097`.
- `LICENSE.txt` payload: `58786a5cf900bbbc778a26cc43220592ce0f36d9da06152c54c1c5a63bce3856`.
- `third-party-notice-inventory.json`: `a063d05021c30b5cb8049d49a94e6950ac0d986cdbd944d8848b7dd7e4baafaf`.
- `installer-payload-list.txt`: `cc19068581c008db5a8a952966594df4bed7d5048bc246f2136e5197b3d67137`.

## Cross-documentation findings and resolution

The read-only cross-check covered specifications, ADRs, provider/storage/UI contracts, release notes, README, QA and the applicability index. Its pre-update checkpoint checked **92 non-vendor Markdown files / 1615 relative links / zero missing targets**; not remote URLs or anchors. Requirements remain consistent:

- British-English UI, existing branding geometry/sky-blue Send/Stop, title-level Union Jack, homogeneous centre and adjacent branding/menus.
- Ctrl-wheel and Ctrl-plus/minus bounded zoom with a 100% default and one-second percentage notice.
- Fixed hidden-startup reveal after at least six seconds plus authenticated readiness, independently fail-visible at ten seconds: implemented policy, **not proof that the owner's flash is cured**.
- Composer `+` without a black frame; independent classification submenu below Extra high; Medium default and classification initially unset.
- Twelve direct formats: PDF, TXT, MD, JSON, XML, YAML, CSV, TSV, PNG, JPG, JPEG, static GIF. DOCX/XLSX/PPTX require safe external owner conversion to PDF; no internal Office converter.
- Native PDF from an immutable temporary copy; encrypted history keeps admitted processed pages/text and responses, never the original PDF/grants. Another complete native read requires reattachment. Local page inspection does not disclose again.
- Persistent current-user encrypted history, Meta-only Contributor policy, prohibited-data restrictions, exact-body native approval and independent durable financial reservations.

New final applicability notices point here before retained snapshots. The CPU contract has a dated actual-main outcome above its unchanged before-code record. README/index/CI/inventory distinguish present evidence from preserved draft/P1–P11 statements. `docs/aster-builder/contract.json` stays a **frozen historical P7/P8 ledger**, not the current execution register; its hashes/NOT RUN/FAIL fields are not rewritten. This documentation-only follow-up changes no runtime, dependencies, CI gates, policy, assets or verified installer bytes. Any subsequent PR has its own source/check identity and does not inherit this run.

## Remaining acceptance, explicitly not hidden

Engineering source/package gates for the exact source above are complete. They do **not** establish:

- All 42 compound cases, which remain NOT RUN. Owner-installed Windows 11 startup/flicker, DPI, accessibility, focus, menus/zoom, performance and attachments need their own acceptance.
- A new owner v0.2.3 visual PASS; historical v0.2.2 flashes stay FAIL.
- Live Meta/multimodal interpretation, real-key/account/data/financial acceptance or usage/settlement; these remain separate. Contributor confidentiality restrictions apply.
- GLib remediation: 0.18.5 stays OPEN/UNRESOLVED under the [upstream deferral](GLIB-UPSTREAM-DEFERRAL-20261009.md). Both RustSec warnings are retained; no dismissal/ignore/dependency fix is performed.
- Signing, production qualification or redistribution. Old ESET clearance is not a certificate for these new bytes.

v0.2.1 remains locally tested/discarded for bugs; no v0.2.1 branch/release/installer/discarded commit series is published. The owner authorised a documentation-only PR and conditional integration after **all its own checks** pass on 9 October. Publication/integration results are observed separately in that PR, not invented retrospectively in this tested-source receipt. No production release is created.
