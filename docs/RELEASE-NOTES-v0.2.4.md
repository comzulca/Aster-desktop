# Aster v0.2.4 — engineering notes

Current delivered software snapshot: main `57339a687125a0b1fd8d6cda787649b5a111fa65`, 5 November 2026. This identifies an unsigned engineering artifact, not a production release. [Exact receipt](qa/FINAL-ENGINEERING-V024-20261105.md) / [current status](ENGINEERING-STATUS.md).

## 5 November maintenance

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

Main run `42081672227`, attempt 1: all five jobs PASS; frontend 178 PASS; default Rust suite 412 PASS / two ignored; the two ignored disposable Credential Manager cases passed separately. The actual packaged executable passed fifteen codec and six resource probes, with independent causal CPU validation. The unsigned installer and twelve resources were independently checked without launching the application/setup. See the [receipt](qa/FINAL-ENGINEERING-V024-20261105.md) for exact identities and limits.

## Upgrade and retained limitations

Close Aster normally and install the engineering build. In Settings, configure/validate the current direct Meta key as needed, review a current spending configuration, then review Contributor terms/account eligibility. Only synthetic or eligible public content is admitted. [Operating guide](SETUP-AND-USE.md).

Automatic full-window compaction, the 4-GiB transcript migration and runtime GitHub mutations remain unavailable; arbitrary script/MCP execution and model file-writing tools are not enabled. Current text limits remain 20 continuation turns / 512 KiB, distinct from the model window and the media wire-body limit. No causal flicker cure, owner-installed/live acceptance, production signing or completion of the 42 compound protocols is inferred. GLib remains open under the existing upstream deferral.

## Documentation reconciliation

The latest documentation separates current behaviour/evidence from historical preparation. It corrects protocol, attachment, policy-date, test and installer statements without changing runtime code or rebuilding/retagging the delivered file. Earlier candidate notes are preserved byte-for-byte in [RELEASE-NOTES-v0.2.4-history-57339a6.md](RELEASE-NOTES-v0.2.4-history-57339a6.md).
