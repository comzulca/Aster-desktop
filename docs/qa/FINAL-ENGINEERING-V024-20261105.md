# Delivered v0.2.4 engineering artifact — 5 November 2026

This source-bound receipt consolidates independently observed GitHub evidence and passive staging for the delivered unsigned engineering build. It does not qualify later documentation commits or claim an installed/live/production release.

## Source and CI identity

| Item | Observed identity |
| --- | --- |
| Repository / version | `comzulca/Aster-desktop` / `0.2.4` |
| Main software commit | `57339a687125a0b1fd8d6cda787649b5a111fa65` |
| Source tree | `f29dcbaca827ac02574cee674ce5ca297f012063` |
| Integration | [PR #68](https://github.com/comzulca/Aster-desktop/pull/68), squash merge; parent `0448445813211783086cb7f4a44a4ec6a564da5f` |
| Main workflow | [42081672227](https://github.com/comzulca/Aster-desktop/actions/runs/42081672227), attempt 1, `push`, `refs/heads/main`, SUCCESS |
| Main Windows host job | `127436504423`, SUCCESS |
| Main engineering-installer job | `127444907510`, SUCCESS |
| Runner / toolchain | Windows hosted image; `Microsoft Windows NT 10.0.26100.0`; `rustc 1.97.0 (2d8144b78 2026-07-07)` |
| Package diagnostic principal | `runnervmhao91\runneradmin`, hosted disposable runner, not owner session |
| Manifest build timestamp | `2026-11-05T18:35:01.3478596Z` |

The PR head was `929eea9d6dc1bdb5139efa1c07382b44648780fc`; its tested merge was `9576918b62727c5d5d40d8e1796bcf358f09f923`, with the same tree. Its separate run `42076984584`, attempt 1, passed all five jobs and actual probes before integration. The delivered artifact comes from the subsequent actual-main run, not the PR candidate.

## Automated outcomes

| Gate or scope | Result |
| --- | --- |
| `source-policy`, `frontend`, `codec-source-fixtures`, `windows-host`, `engineering-installer` | All five PASS on exact main source |
| Frontend | 178 PASS across 14 files; TypeScript/Vite build PASS |
| Windows default Rust suite | 412 PASS, zero FAIL, two explicitly ignored native credential cases; zero-test targets PASS |
| Disposable Credential Manager follow-up | Both exact ignored cases executed separately and PASS on the ephemeral host; real owner target/app bootstrap/live Meta were not entered |
| Formatting / all-target check / strict Clippy | PASS |
| Advisory, licence, notices and SBOM gates | PASS with existing two allowed RustSec warnings; not a warning-free or GLib remediation claim |
| Actual release executable | Fifteen codec cases PASS; six resource cases PASS |
| Causal CPU receipt | Independent `verify_cpu_receipt.py` PASS; parent-user-CPU-watchdog, no fallback qualification |
| Qualification flags | `allPassed=true`, `qualificationComplete=false` |

The two credential follow-up cases are `credential::native::tests::synthetic_credential_round_trip_uses_a_disposable_target` and `credential::native::tests::malformed_stored_blobs_are_not_configured_on_a_disposable_target`. They are not the owner's credential-entry GUI or actual API key.

The fifteen codec cases include complete text/PNG/JPEG/static GIF/two-page PDF plus malformed, animated, renamed, page/dimension/pixel/source/text/UTF-8 denials. The six resource cases are CPU kill, memory denial, wall kill, active-process denial, kill-on-close and cancellation kill. No native probe was substituted with a source-only fixture.

## Installer and artifact binding

| Item | Value |
| --- | --- |
| Classification | `unsigned-engineering-build` |
| Original setup name | `Aster Desktop_0.2.4_x64-setup.exe` |
| Delivery alias | `Aster_0.2.4_x64_Engineering_57339a6-setup.exe`, independently byte-identical |
| Installer size | 24,204,364 bytes |
| Installer SHA-256 | `70e566cc9a279fded4730ec05abe6496105daf81fccc701c87642a0737d39366` |
| Windows Authenticode result | `NotSigned`, observed in hosted packaging |
| Main installer artifact ID | `13441129460` |
| Artifact ZIP SHA-256 | `f6e2f7c091de52d7869638d02af68424ea2b971bd2f34db5c1d714450cec6495` |
| Artifact retention observed | Expires `2026-11-19T18:35:03Z`; availability is time-limited |
| Main native-probe artifact ID | `13441824140` |
| Native-probe ZIP SHA-256 | `a06c6514b124bef7fd7768cd0b6fac64baea489bd13ac55d28f9384a6471602b` |

Artifacts are listed on the linked main workflow. This receipt contains no personal download/session URL or credential. There is no signed production release associated with these bytes.

## Independent passive staging

The main artifact ZIP digest was checked against GitHub's observed digest before inspection. Its exact five members, lengths, manifest source/run/attempt/ref/event, installer bytes/hash and supplemental evidence hashes were validated. The x64 application, complete extracted payload and all twelve resource sizes/hashes were checked against the reviewed source and notice evidence, without launching setup or the application.

The twelve resources include the ten original inert Aster Builder files, root LICENSE and generated notices. LICENSE matches the exact Windows CRLF checkout derived from the tracked source. The initial strict Linux-LF byte comparison failed at this one text representation; the recorded correction verifies the exact derived CRLF size/hash, not an arbitrary manifest-defined replacement. The ten inert resource identities remain unchanged. Twenty payload entries and 24,109,098 extracted file bytes match the CI evidence.

The local check was static Linux inspection, including absence of an embedded PE certificate table; it did not execute Windows `Get-AuthenticodeSignature`. The native `NotSigned` result is independently attributed to the hosted Windows packaging step. No claim of bit-for-bit reproducible builds or owner installation follows from alias equality.

## Remaining qualification and history

Owner-installed Windows 11/native GUI/privacy/disclosure and actual account/key/inference/invoice acceptance, repeated startup/flicker evaluation, all 42 compound obligations, signing and production remain separate. Runtime GitHub mutations, full-window compaction and the 4-GiB migration remain unavailable. [GLib](GLIB-UPSTREAM-DEFERRAL-20261009.md) remains OPEN. Original failures, source receipts and frozen ledgers retain their exact historical scope.

See [current status](../ENGINEERING-STATUS.md), [maintenance specification](../specs/SPEC-0007.md), [implementation checkpoint](V024-MAINTENANCE-20261105.md) and [documentation reconciliation](DOCUMENTATION-RECONCILIATION-20261105.md).
