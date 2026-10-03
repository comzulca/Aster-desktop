# Contract: Meta Muse Glimmer 30B via NVIDIA hosted API

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
