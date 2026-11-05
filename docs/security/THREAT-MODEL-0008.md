# THREAT-MODEL-0008 — spending activation and readiness

Before code, 5 November 2026. [SPEC-0008](../specs/SPEC-0008.md), [ADR-0023](../adr/ADR-0023.md).

Data flow: host clock/review policy → bounded status deadline → local form intent → host validation → exact native review → fresh host validation and existing durable Configure event → refreshed status → renderer readiness. No expiry, receipt or renderer control grants inference authority.

| Threat | Mapping | Control / residual |
| --- | --- | --- |
| TM-044 | SEC-039 / AC-092,093 / T-102,103 | A permissive date picker, stale clock/deadline or forged form bypasses expiry, or silent clamping approves a different proposal. Host-owned upper bound, visible local-time limit, fixed error categories before review and fresh commit validation; no ledger/window extension. Native review may cross a deadline and must still fail. Historical replay retains original rules and reservations. |
| TM-045 | UX-021 / SEC-039 / AC-094,095 / T-104,105 | Filled fields, a cancelled receipt or stale/incompatible status looks active; errors hide behind Settings; classification guidance becomes approval. Show only refreshed host activation, render notices inside the active dialog, version/strictly validate status, name the missing Send condition without selecting classification or sending automatically. Host admission remains authoritative. |

The owner separately authorises a bounded local synthetic live probe with the attached key. Only that synthetic prompt may enter Contributor. Keep the key in process memory, TLS verification and fixed Meta origins, disable redirects/retries, cap response/time and retain only non-secret outcomes. This is independent service evidence, not proof of the owner's installed Windows/native review path. Existing external-client spending and exchange/tax assumptions remain residuals; the local ceiling is not Meta's global invoice cap.
