# Meta Model API / Muse Spark 1.3 Contributor

Current implemented pair: `meta_model_api` / **`muse-spark-1.3-contributor`**, reviewed against PR #70 head `c6974cd`, 5 November 2026. IPC protocol 5 / financial policy 4. [Maintenance and final delivery receipt](https://github.com/comzulca/Aster-desktop/pull/70). This is the default and only inference model; Standard tier, other model versions, retired providers, arbitrary endpoints and fallback remain unavailable.

## Source-pinned access contract

The [transport contract](transport.md) defines fixed Chat Completions inference, host-only key custody, strict request/response handling, native exact review, current-key model retrieval and independent spending admission. [provider.rs](../../src-tauri/src/provider.rs), [credential.rs](../../src-tauri/src/credential.rs), [credential_validation.rs](../../src-tauri/src/credential_validation.rs) and [host](../../src-tauri/src/lib.rs) are authoritative for the implementation.

A direct Model API key is entered only through Windows. Authentication/model visibility does not prove billing, quota, territorial access or every modality. A coding-harness subscription credential restricted to that harness is not a direct-API entitlement for Aster. The [5 November primary-source review](META-POLICY-REVIEW-20261105.md) records terms revision 2026-10-09, geographic revision 2026-10-12 and the unchanged exact model/rates. Account-specific eligibility remains the owner's independent declaration.

Contributor permits training use of submitted content. Personal, sensitive and confidential material, including code that must remain confidential, is prohibited. Local encryption, public/private repository permissions or an acceptance checkbox cannot change data eligibility. Both new content and selected prior visible answers are re-admitted before separate native disclosure. Personalization remains local-only.

## Implemented modalities and continuation

The current host is not text-only: [bounded attachments](attachments.md) support complete eligible native PDFs, normalised images and processed text. Original filenames/paths are not renderer authority; the fixed serializer chooses the outbound representation. Originals remain temporary; saved derivatives are local history, not restored read/disclosure grants.

Current runtime continuation remains 20 completed turns / 512 KiB text, with a separate 24-MiB media wire limit, 32-KiB user/composed message admission and 8,192 completion tokens. The model's documented larger context does not implement the unfinished full-window counter/compaction/history migration. Reasoning uses the five fixed efforts and defaults to `medium`; private reasoning is not saved or exposed as ordinary continuation.

## Dated prices and local finance

The recorded 5 November Contributor rate basis remains USD **0.10 input / 0.002 cached input / 0.20 output per million tokens**. Provider account/team quota and availability are separate and can change; the application does not infer them from key validation. See the dated review and its primary links rather than treating a historical quota figure as current account capacity.

Aster conservatively reserves USD 0.106496 per admitted maximum request before the owner's explicitly reviewed integer GBP conversion/margin/tax/payment assumptions. Its local £89 London-calendar-month ceiling and £80.10 warning are not account-global invoice guarantees. Unknown spend blocks dispatch, and historical quotes retain their original basis.

The current local policy expires at **2026-12-06T00:00:00Z**. Budget configuration has at most 31-day validity and cannot outlive that boundary. Legacy 2 November expiry remains only for historical events; expired configuration and old terms acceptance are not silently renewed. [Policy](../../src-tauri/src/contributor_policy.rs), [terms](../../src-tauri/src/contributor.rs), [budget](../../src-tauri/src/budget.rs), [ADR-0022](../adr/ADR-0022.md).

## Verification and historical records

Initial complete maintenance run `42109052799` passed all five jobs and actual package probes. A separately authorised local synthetic text stream returned HTTP 200, final stop, DONE and consistent usage with the fixed Aster-compatible envelope. This is narrow real-key service evidence, not installed Aster/native review, legal/account clearance, live multimodal entitlement or production. [Successor verification](../qa/SEND-READINESS-20261105.md#observed-successor-verification) and the [PR #70 delivery receipt](https://github.com/comzulca/Aster-desktop/pull/70) bind candidate and final main evidence independently. The earlier [57339a6 main receipt](../qa/FINAL-ENGINEERING-V024-20261105.md), all 42 compound obligations and unresolved GLib retain their own scope.

The complete previous text-only and intermediate development contracts, D5 evidence, original model catalogue diagnostic and earlier prices/quotas/statuses are preserved in [meta-muse-spark-1.3-contributor-history-57339a6.md](meta-muse-spark-1.3-contributor-history-57339a6.md). They retain their original dates and source scope; their old NOT RUN/PASS statements do not replace current evidence.
