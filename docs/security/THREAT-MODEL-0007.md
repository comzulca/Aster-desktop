# THREAT-MODEL-0007 — renewed policy and accumulated temporary originals

Before-code register, 5 November 2026. [SPEC-0007](../specs/SPEC-0007.md), [ADR-0022](../adr/ADR-0022.md).

Data flow: explicit owner picker → bounded native reads/isolated processing → transactional process-local registry → metadata-only renderer receipt → separate classification/native disclosure → existing immutable request/financial pipeline. Terms and financial replay remain in independent existing protected domains. No data or credential is transmitted by the tests.

| Threat | Mapping | Control / residual |
| --- | --- | --- |
| TM-040 | SEC-037 / AC-087,088 / T-097,098 | Blind expiry extension, display/enforcement drift or old acceptance grants new policy. One dated host policy, recognised historical windows, fresh current native acceptance and explicit expiry error. Provider changes before the local deadline remain a residual; no automatic legal or account clearance is claimed. |
| TM-041 | SEC-038 / AC-089 / T-099 | Renewal loses reservations or reinterprets old charges/configurations. Historical policy-window replay, unchanged rates and schema, old expiry preserved, new owner configuration, pending-spend carry and original quote settlement. Same-user rollback and external-client charges remain existing residuals. |
| TM-042 | SEC-038 / AC-090,091 / T-100,101 | Repeated additions bypass aggregate limits, leak across chats, resurrect expired grants or reset five-minute custody. Snapshot-bound atomic append, whole-union validation, earliest immutable deadline, same-conversation and duplicate-ID checks. Expiry can legitimately remove the batch while a picker remains open. |
| TM-043 | UX-020 / AC-090,091 / T-100,101 | Cancellation erases user work, or a malformed/uncertain response displays grants that the host no longer owns. Cancellation returns full current metadata; known atomic rejections preserve prior selection. Unknown/malformed receipts clear displayed IDs and best-effort revoke known IDs; no auto-send or renderer-supplied paths. |

All pre-existing native worker, privacy, processed-history, tool, budget and platform residuals remain. No live key/provider test or owner app launch follows from source qualification.
