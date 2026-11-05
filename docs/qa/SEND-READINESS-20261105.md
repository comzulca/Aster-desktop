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

## Observed successor verification

The earlier table is the retained pre-publication cutoff. On 5 November the completed first candidate [run 42109052799](https://github.com/comzulca/Aster-desktop/actions/runs/42109052799), attempt 1, passed all five jobs against tested merge `8e92ef76a4e7910ae2193e87e7085c4910b58d1e`, PR head `c6974cdc61d4dd56864fa9076848f3c867e0fff9`. Observed results: 184 frontend PASS; 414 Rust PASS / two ignored; two separate disposable credential PASS cases; actual fifteen codec/six resource probes and independent causal CPU PASS.

Passive PR-candidate verification: archive SHA-256 `2a5ee82041094a75d1774e144870b6303e8d713cabb62525d0a31d4913971861`; setup 24,213,580 bytes / SHA-256 `d9445acf267804a33bb9547eab00bd5c90f8ee7f1e76044e3c6d5c12b45783e2`; twelve exact resources, notices, x64 executable and candidate/run/attempt identity verified. Windows CI recorded NotSigned; static Linux verification separately found no embedded certificate. Neither app nor setup was launched. This is a PR candidate, not the final main installer or delivery alias.

An independent natural-language synthetic service probe at 21:04 UTC returned HTTP 200, `stop`, DONE, non-empty acknowledgement and consistent usage: 78 prompt / 1021 completion / 1099 total tokens. It used the unchanged developer/streaming/medium/8192/n=1/store=false envelope and did not invoke the installed Windows app or owner stores. Its PASS criteria are response completion/usage, not an exact prose promise. The earlier literal-marker FAIL remains intact. An optional browser visual check was NOT RUN because the isolated browser archive could not be installed; headless DOM and hosted tests do not replace owner GUI acceptance.

The [documentation follow-up](DOCUMENTATION-SEND-RECONCILIATION-20261105.md) cross-checks current entry pages without changing runtime source or old artifacts. Its changed source requires fresh CI. The [PR #70 delivery receipt](https://github.com/comzulca/Aster-desktop/pull/70) records the final independently verified main source/run/attempt/installer/hash separately, after those gates.
