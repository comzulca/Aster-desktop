# NVIDIA hosted chat transport contract (draft v0.1.0)

**Reviewed:** 2026-09-24 UTC. **Scope:** Aster-desktop's host-side adapter for exactly two allowed NVIDIA-hosted model IDs. This is an engineering contract, not evidence that the network path or Windows packaging has passed a live test.

## Fixed authority and data flow

| Property | Contract |
| --- | --- |
| HTTPS origin | `https://integrate.api.nvidia.com` only |
| Request route | `POST /v1/chat/completions`; no arbitrary URL, alternate host, or path accepted from UI/model/content |
| Authentication | `Authorization: Bearer <NVIDIA credential>` added only inside the trusted Windows host after reading Windows Credential Manager; secret bytes never cross into WebView, IPC results, repository, logs, fixture files, exports, or telemetry |
| Request content | `Content-Type: application/json`, `Accept: text/event-stream`; UTF-8 serialized JSON with a model-specific closed request shape |
| Model catalog | `nvidia/nemotron-3.5-lightning-30b-a3b` (default), `meta/muse-glimmer-30b` (selectable); unknown model/provider/destination rejected before network |
| Transport | Standard certificate and hostname verification; redirects and unintended proxies disabled; no fallback endpoint/model and no automatic retry |
| Tools | Provider-side tool calls disabled/omitted for this MVP; generated text is untrusted proposal data. File operations require a separately defined typed host proposal, explicit per-operation user approval, revalidation, and distinct host commit path. No provider response can grant itself approval. |

Only send user-visible text and specifically approved context to NVIDIA. Request creation and credential access are host operations; IPC accepts typed user intent, model ID, conversation ID, and bounded content, never a URL, key, arbitrary request headers, transport options, or an unbounded provider JSON body. Do not dump raw requests or responses in diagnostics. A configured-key flag may be returned to UI; no key bytes may be returned. Neither this adapter nor its streaming parser writes project files.

Treat prompt, generated response, optional `reasoning_content`, usage and errors as potentially sensitive. Separate reasoning from assistant `content`; do not turn reasoning text into a UI answer, an instruction, a log field or an edit operation. Rendering or retention of reasoning requires a separate product decision. Keep per-conversation continuation isolated, respect each model's role order, and never mix response chunks from concurrent request IDs or models. Sampling `seed`, if later enabled, only promises NVIDIA's best-effort repeatability and does not prove deterministic prose.

## Streaming state machine and proposed bounds

The user requested `stream: true` for both models. NVIDIA documents data-only server-sent events (SSE) with `data: <JSON>` frames and terminal `data: [DONE]`. Host-only state progression is `ADMITTED -> CONNECTING -> STREAMING -> {COMPLETED, CANCELLED, FAILED}`. Enforce exactly one terminal event for a request; cancellation drops subsequent chunks and closes the stream; a missing `[DONE]` or transport EOF is `FAILED`, never silent success. Decode UTF-8 across byte boundaries and SSE frame boundaries. Accumulate answer `delta.content` incrementally, collect any `delta.reasoning_content` on a separate channel, skip empty `choices` when valid, enforce `choices[].index` consistency, and distinguish `finish_reason=length` from complete answer. No project edit may be committed on truncation, parsing failure, cancellation or partial output.

Initial host safety ceilings proposed for the versioned registry: connect 10 seconds, first response headers 30 seconds, idle between frames 60 seconds, total request 900 seconds, one SSE event 256 KiB, total response 8 MiB, and 262,144 frames. Enforce them in the host and prove them with a fake server before live enablement; adjust only by explicit policy revision and tests. Bound upstream messages/request body and per-conversation memory separately in the product spec. Application memory is bounded by streaming, not by retaining every raw chunk. These values are Aster policy choices, **not** NVIDIA-advertised limits.

For `stream=true`, require a successful HTTP status and `text/event-stream` framing; parse each `data:` payload as JSON or the exact `[DONE]` marker. Reject invalid UTF-8, malformed JSON, oversized frames/body, contradictory IDs/model/indexes, unexpected tool calls, duplicate terminal outcomes and impossible state transitions. Unknown harmless JSON metadata may be ignored after bounds checks; unknown actionable message types never expand authority. If a provider returns `202` pending rather than an SSE stream, mark the feature unsupported with a clear error until its polling contract is separately specified. Map authentication, validation, rate limit/quota and server failures to user-safe error classes without copying error bodies or keys to logs. A partial answer displayed before a failure must be marked incomplete. If usage is omitted or arrives only in a final chunk, record it as unavailable until an authoritative count is seen; never invent cost or remaining quota.

## Fixture matrix and promotion rule

Use a loopback fake server with a fake key for each model. Serialize and snapshot the *exact* JSON request, model ID, headers (redact the fake token in snapshots), selected params, and `stream=true`; assert zero network calls for disallowed IDs or missing credential. Drive normal chunks with answer and separate reasoning, empty choices, split UTF-8/frame delimiters, fragmented `[DONE]`, missing usage, `finish_reason=length`, large and malformed frames, missing terminal marker, wrong model/index/ID, unexpected `tool_calls`, cancellation during connection/stream, idle timeout, HTTP 202/401/403/422/429/500, redirect response, and proxy configuration. Assert secret absence from IPC, logs, errors and test artifacts. Do not convert fixture PASS into live-provider compatibility.

Live canaries for **each** model are `NOT RUN`: no actual NVIDIA key or authority for provider cost/data disclosure is present in this environment. A canary needs an explicit user-approved request with non-sensitive prompt after native Windows credential setup; capture timestamp, model ID, request shape excluding token, response result, and cost/usage if reported. Recheck model availability, terms and pricing at live-enablement time. Windows build, credential read/write, and live HTTPS remain separately gated.

## Source evidence

- NVIDIA [Nemotron 3.5 Lightning endpoint schema](https://docs.api.nvidia.com/nim/reference/nvidia-nemotron-3-5-lightning-30b-a3b-infer), reviewed 2026-09-24.
- NVIDIA [Muse Glimmer endpoint schema](https://docs.api.nvidia.com/nim/reference/meta-muse-glimmer-30b-infer), reviewed 2026-09-24.
- NVIDIA [Nemotron hosted code example](https://build.nvidia.com/nvidia/nemotron-3.5-lightning-30b-a3b), reviewed 2026-09-24.
- User-provided endpoint examples: `01-nemotron-3.5-lightning-30b-a3b.txt` (SHA-256 `826261f2c1ae3ebba7f365b7ad8a865ad46f2e0b3ecb347a35c1e22482db63dc`) and `02-muse-glimmer-30b.txt` (SHA-256 `9a44533e93b5fdd645e174e6a6f4f397216e1090d0ff4c1c1d8220fbb0562407`); both contained only a literal `$NVIDIA_API_KEY` placeholder.
