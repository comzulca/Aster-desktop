# Aster v0.2.4 — engineering notes

Current maintenance: [PR #70](https://github.com/comzulca/Aster-desktop/pull/70), implemented head `c6974cd`, initial complete candidate run `42109052799`, attempt 1. The PR delivery receipt separately records final main source and verified installer identity. The previous main `57339a6` [receipt](qa/FINAL-ENGINEERING-V024-20261105.md) remains unchanged historical evidence. Unsigned engineering is distinct from production. [Current status](ENGINEERING-STATUS.md).

## Spending activation and Send correction

The host publishes its allowed expiry and validates the exact proposal before native review and again at commit. Settings shows the policy deadline in local Windows time, inline errors, declined review and confirmed active/inactive/expiry state. Filling fields alone does not activate a policy. The composer explains empty/invalid/unclassified drafts and links an unconfigured budget to Usage. Classification and separate native disclosure remain mandatory.

IPC protocol changes to 5; financial policy 4, application version 0.2.4, stores, quote replay, reservations, £89 ceiling, dependencies and provider envelope remain unchanged. [SPEC-0008](specs/SPEC-0008.md), [ADR-0023](adr/ADR-0023.md), [threats](security/THREAT-MODEL-0008.md). For the reported London November setup, 30 November at 23:59 fits the host window; 6 December at 20:33 exceeds its 00:00 UTC policy deadline.

## Earlier 5 November maintenance

### Renewed policy

The host now centralises the terms/geography/review dates and local policy expiry. Terms revision is 2026-10-09, geography revision 2026-10-12, review date 2026-11-05 and local deadline 2026-12-06T00:00:00Z. Expiry remains enforced with an actionable message. Recognised legacy terms records replay without a reset; current acceptance requires fresh native review. Historical financial quotes, charges and pending reservations remain bound to their original conditions, and expired budget configuration is not automatically extended.

### Additive document selection

`+` retains existing rows while the Windows selector opens, appends new documents through an atomic host-validated union and retains the previous selection/draft on cancellation or known pre-commit rejection. Count/byte/text/page/RGBA limits, conversation/snapshot binding, one-use consumption and the earliest five-minute deadline remain enforced. Unknown/malformed receipts fail closed. No originals are newly persisted to disk or restored across chats/restarts.

Traceability: [PR #68](https://github.com/comzulca/Aster-desktop/pull/68), [SPEC-0007](specs/SPEC-0007.md), [ADR-0022](adr/ADR-0022.md), [threats](security/THREAT-MODEL-0007.md).

## Other integrated maintenance

- The 1 November dependency resolution retains the reviewed compatible WinRT cohort and adds an early manifest/lock guard. The incompatible standalone update was not used. [Resolution contract](qa/PR-AND-BRANCH-RESOLUTION-20261101.md).
- The 2 November transition licenses COMZULCA-owned material under Apache 2.0, preserving upstream licences, attribution and exact resource checks. [Licensing](LICENSING.md).
- The earlier v0.2.4 source implements local-only Personalization, bounded inert catalogue/editor/import, isolated Muse-assisted draft generation, key authentication proof, built-in GitHub authentication/read routes and capture-only startup diagnostics. Their live/owner acceptance remains separate.

## Verified engineering evidence

Initial complete maintenance run `42109052799`, attempt 1: all five jobs PASS; frontend 184 PASS; default Rust suite 414 PASS / two ignored, followed by two separate disposable Credential Manager PASS cases. The actual candidate executable passed fifteen codec/six resource probes and independent causal CPU validation; passive inspection checked twelve resources without launching app/setup. A separate natural-language synthetic Meta stream returned HTTP 200 with complete output and valid usage. The earlier exact-literal assertion failure is preserved as a distinct wording result. See [successor verification](qa/SEND-READINESS-20261105.md#observed-successor-verification) and the [PR delivery receipt](https://github.com/comzulca/Aster-desktop/pull/70) for candidate/final-main boundaries.

## Upgrade and retained limitations

Close Aster normally and install the engineering build. In Settings, configure/validate the current direct Meta key as needed, review a current spending configuration, then review Contributor terms/account eligibility. Only synthetic or eligible public content is admitted. [Operating guide](SETUP-AND-USE.md).

Automatic full-window compaction, the 4-GiB transcript migration and runtime GitHub mutations remain unavailable; arbitrary script/MCP execution and model file-writing tools are not enabled. Current text limits remain 20 continuation turns / 512 KiB, distinct from the model window and the media wire-body limit. No causal flicker cure, owner-installed/live acceptance, production signing or completion of the 42 compound protocols is inferred. GLib remains open under the existing upstream deferral.

## Documentation reconciliation

The [current documentation reconciliation](qa/DOCUMENTATION-SEND-RECONCILIATION-20261105.md) cross-checks status, setup, protocol, expiry, test counts and service/package evidence against the spending/Send implementation. Its follow-up changes Markdown only, requires fresh source checks and does not retag an earlier installer. Earlier candidate notes are preserved byte-for-byte in [RELEASE-NOTES-v0.2.4-history-57339a6.md](RELEASE-NOTES-v0.2.4-history-57339a6.md).
