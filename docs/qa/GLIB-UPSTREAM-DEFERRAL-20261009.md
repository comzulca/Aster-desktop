# GLib 0.18.5 — unresolved, deferred to integrated upstream updates

Date: 9 October 2026. Status: **OPEN / UNRESOLVED / UPSTREAM-DEPENDENT**. This records the owner's explicit scope decision; it neither remediates a dependency nor dismisses a security alert.

## Owner decision and boundaries

The owner expressly declines a local GLib backport, vendored patch, override or dependency-remediation implementation at this stage. Preserve the existing registry-sourced `glib 0.18.5`, manifest/lock and vendor identities. Keep Dependabot alert `GHSA-wrw7-89jp-8q8g` / `RUSTSEC-2024-0429` OPEN, visible and documented; do not dismiss it, add an ignore, downgrade a gate or describe it as fixed. Deferral is not zero-risk acceptance, production security clearance or a closed issue.

The preference is an integral architecture review when upstream provides a **complete, compatible, stable and supported chain**, with GTK4/Tauri 3 as the owner's preferred future direction. This does not approve a migration now, require a particular major version as the only possible remedy, promise a schedule, or claim that GTK4 itself is not yet stable. A compatible stable upstream correction on another supported line can be reported for owner review without implementing it. Any future architecture/dependency migration requires a separately approved scope, before-code specification/threat/ADR amendments and fresh exact-source gates.

Only the separate [CPU containment correction](CPU-CONTAINMENT-20261009.md) is authorised for the present implementation PR. It must retain the 10-second user-CPU and 15-second wall budgets and all mandatory gate obligations. No GLib source, manifest, lock, vendor, security-setting or audit-ignore change is included. Strict upstream follow-up below is a review procedure for subsequent authorised dependency work; no scheduled automation, polling job, new account or monitoring service is created by this record.

## Exact inspected baseline and alert

| Field | Source-bound value |
| --- | --- |
| Actual main | `4e46b1bf126cd9937d68d1058ccfcc0f1c162339` |
| Tree / application | `ee96ce466deecabe9b330d64df5ba4ea67a9ea73` / Aster v0.2.3 |
| Cargo.lock Git blob / package records | `f8be7ed23c2f086bb82c0a813d853a6408221827` / 485 |
| Existing chain in broad lock | Tauri 2.12.1 / Wry 0.57.0 / GTK Rust crate 0.18.2 / WebKit2GTK Rust crate 2.0.2 / GLib Rust crate 0.18.5 |
| GLib registry archive checksum | `233daaf6e83ae6a12a52055f568f9d7cf4671dabb78ff9560ab6da230ce00ee5` |

The advisory describes unsound pointer mutability in GLib's Variant string iterator, not merely a version-number conflict. The affected range is `>=0.15.0, <0.20.0`; upstream identifies `>=0.20.0` as patched. Thus the current 0.18.5 source remains affected. A correct backport could address that particular code defect, but the owner rejects that maintenance route in favour of integrated upstream compatibility; no backport or technical remediation is performed. Adding a second direct GLib version would not by itself remove the old transitive copy. [RustSec advisory](https://rustsec.org/advisories/RUSTSEC-2024-0429.html), [official upstream correction](https://github.com/gtk-rs/gtk-rs-core/pull/1343).

The displayed GitHub alert and broad-lock RustSec warning remain separate from target-specific application exposure and from package qualification. Audit exit zero with allowed warnings, a disappearance caused by source/inventory coverage changes, or a green unrelated gate cannot establish a correction. Current-source RustSec also discloses `proc-macro-error 1.0.4` / `RUSTSEC-2024-0370`; this deferral does not resolve that warning.

## Platform evidence — narrow claim only

The inspected stable Wry 0.57.0 manifest places GTK/WebKit2GTK in the Linux/DragonFly/FreeBSD/OpenBSD/NetBSD target branch and uses WebView2 on Windows. The actual-main Windows check/build logs contain no direct compilation/check record for the GLib 0.18.5 crate. These observations support a target-specific non-observation for the inspected Windows build; they do **not** prove universal Windows immunity, absence from every build-time/feature/dependency route, or that the advisory can affect only Linux. [Pinned Wry platform dependencies](https://raw.githubusercontent.com/tauri-apps/wry/wry-v0.57.0/Cargo.toml).

Cargo.lock is a broad source inventory, not an exact runtime binary inventory. Future exposure review must distinguish host from target, normal/build/proc-macro/dev edges, activated features, actual compilation and packaged payload. GTK can be used on Windows in other architectures; do not generalise Aster's inspected WebView2 route to all GLib consumers. [Official GTK4 documentation, including Windows support](https://docs.gtk.org/gtk4/).

The Linux job currently verifies fourteen exact-production-module synthetic codec fixtures and ten vendor-private fixtures. It is not an execution of the GLib iterator regression or qualification of the complete Tauri/GTK Linux application. Linux/BSD application acceptance, all 42 compound protocols, owner Windows 11 visual/startup/private-history tests, real-credential/live-provider acceptance, signing and production promotion remain independently NOT RUN.

## Dated upstream status — not an implemented solution

Primary-source review on 9 October 2026 observes GTK4's published stable library documentation. Tauri's published major-3 package is `3.0.0-alpha.4`, not a stable major-3 release. Stable Tauri 2.12.2 still declares GTK 0.18 in its Linux/BSD target dependencies. These are dated upstream observations, not an authorisation to replace the present Tauri 2.12.1 graph. [GTK4 documentation](https://docs.gtk.org/gtk4/), [Tauri 3 alpha release](https://github.com/tauri-apps/tauri/releases/tag/tauri-v3.0.0-alpha.4), [stable Tauri 2.12.2 manifest](https://raw.githubusercontent.com/tauri-apps/tauri/tauri-v2.12.2/crates/tauri/Cargo.toml).

Upstream GTK 0.19 coordination also exists independently of a GTK4/Tauri 3 migration: [Wry PR #1843](https://github.com/tauri-apps/wry/pull/1843) and [Tauri PR #16170](https://github.com/tauri-apps/tauri/pull/16170) are draft/unmerged at the inspected cutoff. A draft branch, one released leaf, a closed upstream issue or an alpha package is not a complete stable compatible chain. No sole-remedy conclusion or release-date prediction is made.

## Strict checklist for subsequent authorised dependency reviews

Maintainers conducting the next authorised repository dependency review must record a UTC cutoff and exact inspected source, and:

1. Check the advisory's affected/patched ranges and GitHub alert state; retain every still-affected package identity and existing failure history.
2. Inspect official released Tauri/runtime/Wry/Tao/GTK/GLib/WebKit2GTK/menu/tray/plugin manifests, migration notes, supported platforms and Rust/native toolchain minima. Separate release tags from drafts, prereleases and unmerged proposals; assess the complete selected chain, not a single version label.
3. Compare the exact locked graph, sources/checksums, all duplicate GLib versions and relevant activated target/host edges. Corroborate metadata with actual compiler/build receipts and payload evidence; lack of one record is not proof of universal absence. [Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html).
4. Report availability, incompatibilities, residual risks and architecture options to the owner. Preserve this no-backport/no-remediation boundary until a separately authorised migration is agreed; merely checking availability cannot expand runtime or implementation authority.
5. For an authorised migration, amend governing specifications, threats, ADRs, traceability, platform support and acceptance first. Preserve Contributor-only capabilities, credential custody, processed-only history, financial safeguards, CPU/wall/memory/process limits and all independent gate obligations.
6. Re-run exact-source locked installation, source/frontend/full Windows tests, target-appropriate Linux/GLib optimised regressions, audits, provenance, licences/notices/SBOM, actual packaged executable 15-codec/6-resource probes and package guards. Technical gate success is not manual acceptance, legal redistribution clearance or production promotion.
7. Close the technical deferral only with evidence that no affected selected source remains in the intended relevant chain, compatibility/runtime tests pass and the normal alert rescan is independently observed. A changed audit origin, missing inventory, manual dismissal or status-only edit cannot count as remediation. Record any remaining alert separately.

## Independent PR and package evidence

PR #30 merged normally at `2026-10-09T15:19:39Z` after all five final PR-source jobs and all fifteen codec/six resource probes passed on its exact tested source. Its distinct actual-main push [37950896361](https://github.com/comzulca/Aster-desktop/actions/runs/37950896361) subsequently failed the CPU resource probe in both the original package job and one bounded retry. Four source jobs passed; each package execution reported fifteen codec PASS, five resource PASS and CPU FAIL. There is no qualified actual-main installer from that run. The [PR #30 closing receipt](https://github.com/comzulca/Aster-desktop/pull/30#issuecomment-6082465272) records that source-bound outcome.

The CPU correction candidate begins with fresh changed-source gates NOT RUN and requires its own observed results; historical PR success cannot substitute. GLib remains independently OPEN/UNRESOLVED regardless of CPU CI outcomes. Neither this documentation decision nor a future unsigned engineering artifact establishes installed startup/flicker correction, full compound containment, live-provider approval, signing or production readiness.
