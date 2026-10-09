<!-- OCT9 CURRENT APPLICABILITY BEGIN -->
> Current-source applicability — 9 October 2026, 13:02:58 UTC: the [dated reconciliation](../qa/OCT9-RECONCILIATION-20261009.md) owns the newer exact-source outcomes and their limits. Earlier “current”, “pending” and NOT RUN statements below remain source-bound history where superseded there; original bodies and before-code approvals are preserved. P10 hosted success does not qualify a changed cumulative source. Final-source CI/package gates require re-execution; owner flicker FAIL, all 42 compound cases NOT RUN and independent manual/live/signing/production obligations remain.
<!-- OCT9 CURRENT APPLICABILITY END -->

# Contract: Meta Muse Glimmer 30B via NVIDIA hosted API

<!-- P10 CURRENT APPLICABILITY BEGIN -->
> Current-source applicability — 8 October 2026, 21:51:41 UTC: the [dated reconciliation](../qa/OCT8-RECONCILIATION-20261008.md) owns this checkpoint. PR #38 and #31–#35 are normally merged after their four exact-source gates; #36/#37 remain pending. P9's complete Windows suite is FAIL; its isolated synthetic diagnostic PASS does not override it. The [P10 scoped-factory contract](../qa/PDF-SCOPED-FACTORIES-20261008.md) received before-code design approval at 21:47:59 UTC; source/documentation preparation is in progress, not a published or native/package PASS. Earlier “current” headings and statuses below are retained dated history, not the latest source result. The dependency auxiliary case is a private child lease-denial test, not Credential Manager proof. Owner v0.2.2 flicker/Code Integrity FAIL and all 42 compound cases NOT RUN remain; manual/live/signing/production acceptance is not inferred.
<!-- P10 CURRENT APPLICABILITY END -->

> Current-source applicability — 8 October 2026, P9 preparation: the [dated reconciliation](../qa/OCT8-RECONCILIATION-20261008.md) supersedes current-status claims below. PR #38 and #31–#34 are normally merged after four source-bound gates; PR #35–#37 and PR #30 are not complete. P8's native Windows access violation remains FAIL, its installer SKIPPED. The [P9 contract](../qa/PDF-CRASH-DIAGNOSTIC-20261008.md) admits only test-only diagnosis after the unchanged complete suite fails; 112 counted pure checks and static review are not native qualification or an installer PASS. The dependency auxiliary test is an isolation lease test, not native credential execution. Original dated results, owner v0.2.2 flicker/Code Integrity FAIL, credential custody and manual/live/production obligations remain unchanged.

**Retained dated v0.2.3 applicability — 4 October 2026, Edinburgh / Europe/London:** dated evidence below retains its original source, version and result; it does not qualify changed v0.2.3 bytes. Unchanged constraints remain subject to the [current documentation index](../V0.2.3-DOCUMENTATION-INDEX.md) and [SPEC-0005](../specs/SPEC-0005.md). The owner authorises a v0.2.3 PR and GitHub checks only, not merge, release or signing. Final-source, package, native and live qualification require independent evidence in [LOCAL-V023-0001](../qa/LOCAL-V023-0001.md). No retired provider, historical grant or old PASS is reactivated.

**Current-status navigation — 8 October 2026:** see [the reconciliation record](../qa/OCT8-RECONCILIATION-20261008.md) for actual PR/dependency/source/gate outcomes and the later owner-authorised conditional integration. This dated paragraph and the original evidence below are preserved history, not current authority, a new execution receipt or a reactivation of retired providers. Manual/native/live/production results do not transfer between sources.


**v0.2.2 applicability, 3 October 2026:** retained requirements/contracts apply only where unchanged; retired providers, disabled capabilities and older dated results are not current authority or new-source PASS. [SPEC-0004](../specs/SPEC-0004.md), [ADR-0017](../adr/ADR-0017.md) and [LOCAL-V022-0001](../qa/LOCAL-V022-0001.md) govern active zoom/startup changes and fresh evidence. Earlier evidence remains source-bound history; the later owner-reported v0.2.1 startup flash is recorded separately. The entire original v0.2.1 checkout/installer is unchanged.

**Historical applicability, reconciled 3 October 2026:** the dated record/procedures below retain their original version, identifiers, authority limits and PASS/FAIL observations. They do not qualify or authorise changed v0.2.1 source, current key activation, retired providers or disabled project capabilities. Active scope is [SPEC-0003](../specs/SPEC-0003.md) and fresh verification is [LOCAL-V021-0001](../qa/LOCAL-V021-0001.md); no old PASS transfers.

> **RETIRED — v0.1.0 historical evidence, 2 October 2026.** This pair is absent from the active v0.2.0 registry and cannot be selected, used as fallback or restored as conversation context. The original body below is retained for provenance: its defaults, transport links, fixture PASS, owner acceptance and deferred checks apply only to the named historical source/artifact. They are not new requirements or v0.2.0/live-provider proof. No old credential is read, reused, renamed or deleted. Use [the current Meta-only contract](meta-muse-spark-1.3-contributor.md) and [SPEC-0002](../specs/SPEC-0002.md) for the active profile.

**Registry key:** `meta/muse-glimmer-30b`

**Status, reconciled 2026-10-02 (Edinburgh, `Europe/London`, BST, UTC+01:00):** implemented fixture contract; protocol-2 automatic loopback/exact-body fixtures pass locally at immutable runtime source `2bcacd7c6f2496eda233d4a069ec7b65a93d47b3`. Final source/package binding is PASS at source `1782d3375db47466f89be6bd3cd3c6affbfa95b7`, main CI run `37040168700` attempt 1, artifact `11243338033`. The owner reports providers/history PASS on the matching installed build in [OWNER-ACCEPTANCE-0001.md](../qa/OWNER-ACCEPTANCE-0001.md). The historical 2026-10-01 canary NOT RUN checkpoint is retained; no separate Muse request count, timestamp or trace was supplied, so detailed per-model verification is explicitly assigned to [v0.2.0 review](../qa/V0.2.0-FOLLOW-UP-0001.md), not inferred from the category report.

**Default selection:** no; user must explicitly select it.

**Trusted transport:** see [transport.md](transport.md).

## Exact request and recorded deviations from the example

The user-provided Python sample uses `temperature=1`, `top_p=0.95`, `max_tokens=8192`, `stream=False`. NVIDIA's model-specific schema recommends `temperature=0.95`, `top_p=1` and explicitly supports SSE when `stream=true`. Aster uses the documented recommended sampling pair and the user-requested streaming; the 8,192-token ceiling stays as supplied. This is an explicit policy choice for v0.1.0, not a silent interpretation of the attachment. Verify it against the hosted API before live promotion.

```json
{
  "model": "meta/muse-glimmer-30b",
  "messages": [{"role": "user", "content": "Example user text"}],
  "temperature": 0.95,
  "top_p": 1,
  "max_tokens": 8192,
  "stream": true,
  "tool_choice": "none"
}
```

`POST https://integrate.api.nvidia.com/v1/chat/completions`, `Content-Type: application/json`, `Accept: text/event-stream`, host-only bearer credential. Omit `tools`. Setting documented `tool_choice=none` is an extra fail-closed measure: this model's schema otherwise defaults to `auto`. The native host independently rejects any tool call returned in spite of this request. Do not treat model-proposed patches as tool invocations; host file edits require typed proposals, explicit human approval for each operation, resource identity checks, and host revalidation in a separate application contract. The user-supplied sample does not demonstrate tool calling.

The schema allows an optional first `system` message followed by alternating user/assistant roles, with a user message last. The selected conversation provides its own history, and raw thinking or assistant text from Nemotron is never inserted automatically into Muse history. Muse's docs mention optional `reasoning_content`; if a chunk contains it, keep it separate from answer `delta.content` and outside logs, edits and ordinary persistence. Its tokens share the `max_tokens` budget. Neither a documented `seed` nor fixed sampling makes model prose deterministic.

## Response normalization and gates

For `stream=true`, NVIDIA documents partial JSON deltas as data-only SSE `data: ...` events and terminal `data: [DONE]`. Completion, truncation, cancellation, unknown action output, malformed bytes, HTTP failures and missing usage use [transport.md](transport.md)'s host policy. Treat a standalone JSON response as a mismatched live contract when streaming was requested; do not silently fall back to nonstreaming. Fixture checks specific to Muse: exact request parameters including `tool_choice=none`, valid streamed answer with/without reasoning, returned tool call despite `none` (rejected), and absence of answer because reasoning consumes the 8,192-token budget. A fake-server PASS does not demonstrate that the remote endpoint accepts this exact shape. The owner-reported providers/history PASS is recorded separately; its absent detailed model-specific trace is explicit v0.2.0 follow-up, not a fabricated canary outcome or authority to make another request.

## Provenance

- User-provided `02-muse-glimmer-30b.txt`, SHA-256 `9a44533e93b5fdd645e174e6a6f4f397216e1090d0ff4c1c1d8220fbb0562407` (nonstreaming example only).
- NVIDIA [Muse Glimmer endpoint-specific schema](https://docs.api.nvidia.com/nim/reference/meta-muse-glimmer-30b-infer), rechecked 2026-10-01 UTC: exact model ID, recommended `0.95/1` sampling, `tool_choice=none`, streaming SSE, `reasoning_content` description and `data: [DONE]`.
- NVIDIA [Muse Glimmer model page](https://build.nvidia.com/meta/muse-glimmer-30b) for endpoint availability and hosting terms; recheck availability and terms when enabling live access.
