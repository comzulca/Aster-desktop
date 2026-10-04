# THREAT-MODEL-0004 — presentation ownership and transient feedback

Date: 3 October 2026. Extends THREAT-MODEL-0003; provider, credentials, storage, spending and tools remain unchanged.

TM-026: Ctrl-wheel/native zoom or feedback overlays bypass a guarded modal/busy presentation path, compound scale, hide native/caption controls or imply fresh disclosure authority. Mitigate with SEC-025: one bounded local zoom state; native zoom prevention even on blocked Ctrl-wheel; supported disabled native zoom hotkeys; closed modifiers/finite vertical delta; fixed unscaled pointer-transparent status, one timer/cleanup; existing keyboard/menu lock/modal guards. AC-064 / T-074 must exercise denied and malformed routes and verify no send/window/store/credential effects.

Assets: unsent draft/selection, native-review focus, readable caption/UI and true zoom value. Actors: owner input, malformed synthetic events, untrusted renderer/model text. Data flow is local gesture -> guarded renderer presentation state -> inert percent display. No new network/storage/host authority. Residual OS/WebView/GPU scheduling and device-specific input behaviour need owner-installed tests; renderer/host UI maintainers own this risk.

A dark startup surface is reliability mitigation, not antivirus/security certification. The owner-reported v0.2.1 white flash remains an observed failure until new installed evidence establishes correction. Do not mask readiness failures, weaken CSP or security controls, add broad window commands, alter draft/spend state, or claim zero flashes from source tests alone.

Verification checkpoint: R1 source/config, negative fixtures and delayed/failure headless paint passed; all four independent hosted checks passed for the same runtime in PR #29 / run 37150703411. LOCAL-V022-0001 and CI-V022-0001 bind the delivered local unsigned bytes and disclose the retained local Smart App Control block. These results do not close native white-flash, installed accessibility/DPI/input, credential/history/live acceptance or whole-installer antimalware gaps. Only the v0.2.2 PR and checks are authorised; no merge, security workaround or additional host authority was performed.

## Owner clarification — 4 October 2026

Installed v0.2.2 flickering is **FAIL (owner-reported)** with the previously delivered local setup. Source/configuration/headless PASS remains valid only for its stated scope; it did not cure the reported installed defect. See [OWNER-STARTUP-V022-0001](../qa/OWNER-STARTUP-V022-0001.md) for exact runtime/artifact attribution and missing execution details. Correction is planned and pending for v0.2.3, not implemented, tested or independently attributed to a precise cause. Preserve historical failures and unrelated NOT RUN. The current owner request permits normal gated PR #29 source resolution, superseding only the earlier PR/check-only merge restriction; no security/host authority or signing/release/production permission is added.
