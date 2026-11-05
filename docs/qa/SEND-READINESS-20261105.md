# Spending activation and Send readiness — 5 November 2026

Pre-publication checkpoint, baseline actual main `f3b3152a7627fb690e11af4e8d94da407fd7f94a`. [SPEC-0008](../specs/SPEC-0008.md) / [threats](../security/THREAT-MODEL-0008.md) / [ADR-0023](../adr/ADR-0023.md). The previous `57339a6` installer and [receipt](FINAL-ENGINEERING-V024-20261105.md) remain unchanged historical bytes; their PASS does not qualify this changed source.

## Trigger and correction

The owner's expiry of 6 December 2026 at 20:33 London time exceeds the existing 6 December 00:00 UTC policy deadline. Host rejection was correct. The form mentioned only 31 days, the generic error rendered behind Settings and the action discarded its declined receipt. Filled fields could look saved while the host still reported `needs_budget`. Independently, an empty or unclassified draft kept Send disabled without naming that prerequisite.

IPC protocol **5**, financial policy **4**: the host publishes the permitted expiry and independently validates the exact proposal before native review and again at commit. Settings shows the local limit, inline failures, declined review and host-confirmed active/inactive/expiry status. The composer explains its local prerequisite and links an unconfigured budget to Usage. Native disclosure, authentication, classification, original stores/reservations and the £89 ceiling remain.

## Owner operation

1. Install the separately verified corrected v0.2.4 engineering installer; no protected-store reset is needed.
2. In Settings → Usage, retain explicitly verified conversion/charge assumptions and select a valid expiry. For the reported London setup, **30 November 2026 at 23:59** fits the November window; **6 December at 20:33** does not. The host displays its local limit; no silent clamping occurs.
3. Select **Review financial configuration in Windows**, confirm the exact native proposal and check **Policy status: Active** and its confirmed expiry. Entered fields alone do not activate it.
4. Close Settings, write a wholly synthetic message and select the reasoning menu → **Classify content → Synthetic**. Send requires all existing host gates and opens a separate native disclosure review before dispatch.
5. Missing/unverified authentication needs the existing Providers validation. Expired policy, unresolved charges and host/store failures remain independently blocked.

## Evidence at this checkpoint

| Check | Observed result |
| --- | --- |
| New frontend journeys against old implementation | FAIL, all six; retained local before-change log at 21:51 UTC. |
| Corrected full frontend suite | PASS, 184 tests / 15 files, Linux/jsdom at 21:55 UTC, locked Vitest 5.0.5 CLI. |
| TypeScript / Vite production build | PASS, direct locked CLIs, Node 24.22.1; lock/dependency inputs unchanged. |
| Secret guard | PASS before publication. Exact supplied credential must also be absent from staged files; it is never uploaded to CI. |
| Local direct Meta probe | HTTP 200, final stop, DONE and usage using the unchanged Aster developer/streaming/medium/8192/n=1/store=false envelope. The optional exact-literal response assertion failed and remains FAIL; complete generated prose is not an exact-match success. Service acceptance and model wording are distinct. |
| Fresh five-job Windows/source/package CI | NOT RUN at this pre-publication checkpoint; successor PR/Actions evidence must bind exact source/run/attempt/artifact before delivery. |

Only synthetic content entered Meta. The non-secret local report excludes the key. A separate natural-language probe, if run, has its own outcome without promising deterministic prose. Headless/service checks do not prove the owner's installed GUI/native review path. All 42 compound obligations, production/signing and unresolved GLib remain separate. The agent does not launch app/setup, reset stores or change security controls.
