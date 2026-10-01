# Contract: NVIDIA Nemotron 3.5 Lightning 30B A3B

**Registry key:** `nvidia/nemotron-3.5-lightning-30b-a3b`

**Status:** implemented fixture contract; protocol-2 automatic loopback/exact-body fixtures pass locally at immutable source `c718e2c75a6c94a77020eb7e047c07503945b388`, while completed hosted CI/package binding and the one authorized final-artifact live canary remain `NOT RUN` as of this evidence checkpoint on 2026-10-01 UTC.

**Default selection:** yes.

**Trusted transport:** see [transport.md](transport.md).

## Exact request

The host constructs this UTF-8 JSON structure, replacing the sample message with admitted conversation messages; the renderer cannot alter the model, host, credentials, or fixed sampling policy. The Python `extra_body` in the user-supplied sample becomes top-level JSON fields on the wire.

```json
{
  "model": "nvidia/nemotron-3.5-lightning-30b-a3b",
  "messages": [{"role": "user", "content": "Example user text"}],
  "temperature": 1,
  "top_p": 0.95,
  "max_tokens": 16384,
  "chat_template_kwargs": {"enable_thinking": true},
  "reasoning_budget": 16384,
  "stream": true
}
```

`POST https://integrate.api.nvidia.com/v1/chat/completions`, `Content-Type: application/json`, `Accept: text/event-stream`, host-only bearer credential. Allowed message flow is optional leading system message followed by alternating user/assistant messages, ending in user. Continuation is conversation-local; never replay a previous model's hidden reasoning as assistant content. `max_tokens` includes the generated answer; the sample's `reasoning_budget=16384` may consume the full budget, so measure this in the live canary and revise this versioned policy only if evidence justifies it. The sample is a starting parameter set, not a guarantee of useful answer length.

## Response normalization and gates

NVIDIA's hosted example accesses streamed `choices[0].delta.reasoning_content` separately from `choices[0].delta.content`; both may be absent or null in individual chunks. Aster must keep these channels separate and expose `content` as the answer. `finish_reason=length` is incomplete even if a terminal `[DONE]` follows. Empty `choices` may be skipped without treating a partial stream as complete. Only `[DONE]` after a valid terminal response completes the request. Do not treat a `seed` value as proof of identical model text. `tools`, `tool_choice`, and arbitrary endpoint overrides are outside this model-specific documented schema; do not synthesize provider tool calls or offer the model a commit channel in this release.

Fixture checks specific to this pair: request byte-level JSON structure, thinking and reasoning budget, independent reasoning/content channels, empty choices, output exhaustion, `[DONE]` without final content, delayed and truncated events, and host rejection of model ID mismatches. Shared transport and hostile-response checks are in [transport.md](transport.md). A live canary with real NVIDIA service has **not** run; neither acceptance of `reasoning_budget` nor event behavior is proven at runtime by documentation alone.

## Provenance

- User-provided `01-nemotron-3.5-lightning-30b-a3b.txt`, SHA-256 `826261f2c1ae3ebba7f365b7ad8a865ad46f2e0b3ecb347a35c1e22482db63dc`.
- NVIDIA [hosted model example](https://build.nvidia.com/nvidia/nemotron-3.5-lightning-30b-a3b), [endpoint-specific schema](https://docs.api.nvidia.com/nim/reference/nvidia-nemotron-3-5-lightning-30b-a3b-infer), rechecked 2026-10-01 UTC. These show `stream=true`, the exact ID, 16,384-token sample budget, and SSE completion with `data: [DONE]`.
