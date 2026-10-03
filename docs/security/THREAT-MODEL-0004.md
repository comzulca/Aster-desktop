# THREAT-MODEL-0004 — presentation ownership and transient feedback

Date: 3 October 2026. Extends THREAT-MODEL-0003; provider, credentials, storage, spending and tools remain unchanged.

TM-026: Ctrl-wheel/native zoom or feedback overlays bypass a guarded modal/busy presentation path, compound scale, hide native/caption controls or imply fresh disclosure authority. Mitigate with SEC-025: one bounded local zoom state; native zoom prevention even on blocked Ctrl-wheel; supported disabled native zoom hotkeys; closed modifiers/finite vertical delta; fixed unscaled pointer-transparent status, one timer/cleanup; existing keyboard/menu lock/modal guards. AC-064 / T-074 must exercise denied and malformed routes and verify no send/window/store/credential effects.

Assets: unsent draft/selection, native-review focus, readable caption/UI and true zoom value. Actors: owner input, malformed synthetic events, untrusted renderer/model text. Data flow is local gesture -> guarded renderer presentation state -> inert percent display. No new network/storage/host authority. Residual OS/WebView/GPU scheduling and device-specific input behaviour need owner-installed tests; renderer/host UI maintainers own this risk.

A dark startup surface is reliability mitigation, not antivirus/security certification. The owner-reported v0.2.1 white flash remains an observed failure until new installed evidence establishes correction. Do not mask readiness failures, weaken CSP or security controls, add broad window commands, alter draft/spend state, or claim zero flashes from source tests alone.
