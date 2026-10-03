# Meta hosted text transport — v0.2.0

Reviewed against primary documentation on **2 October 2026**. Active implementation contract: [SPEC-0002](../specs/SPEC-0002.md), [ADR-0010](../adr/ADR-0010.md) and [ADR-0011](../adr/ADR-0011.md). The only admitted pair is Meta Model API / `muse-spark-1.3-contributor`. This supersedes the active v0.1.0 transport; the retired text remains below as historical evidence.

**Source-pinned evidence, 3 October 2026:** tested local implementation `f8622a0bd7f0c1ca89f435a2f1d6a415842525ec`, tree `c4e4848201cdb3061be1d74a4a7f565e05cc0ad4`. **143 Rust PASS, two ignored credential fixtures and 41 frontend cases PASS**; format/check/Clippy/typecheck/build, source guards/audits and source-pinned unsigned local build/bounded payload inspection passed. The draft/native-denial/late-receipt and financial-headroom corrections are exercised PASS, not rerun pending. [LOCAL-V020-0001](../qa/LOCAL-V020-0001.md) records source/commands/UTC/hashes and retains earlier unpinned checkpoints/failures. Account entitlement, credential GUI/native Windows journeys, paid/live canary, main-CI package/manifest and installed acceptance remain **NOT RUN**. Local fixtures/payload do not establish live compatibility or promotion; no historical PASS transfers to this adapter.

## Closed destination and reviewed body

| Property | Active Aster contract |
| --- | --- |
| Destination | HTTPS POST `https://api.meta.ai/v1/chat/completions` only; no renderer/model origin, route or header override. |
| Authentication | Host-only Bearer direct Model API credential from fixed Windows target `AsterDesktop/MetaModelAPI/v1`; no key bytes through ordinary IPC, prompts, repository, logs or CI. |
| Request | JSON UTF-8; `Accept: text/event-stream`; only the model-specific closed fields documented in [the model contract](meta-muse-spark-1.3-contributor.md). |
| Transport | Standard TLS certificate/hostname verification, HTTPS-only, redirects disabled, unintended proxies disabled. |
| Dispatch | One immutable owner-window-reviewed byte allocation sent once; no rebuild, automatic retry or provider/model fallback. |
| Tools | Omitted and independently denied in responses. All four project capabilities are false; no files, attachments, proposals, shell, browser, web/MCP or generic tool channel. |

The provider's authentication and endpoint are documented in [Authentication](https://dev.meta.ai/docs/authentication) and [Chat Completions](https://dev.meta.ai/docs/protocols/chat-completions). The local implementation is stricter than a generic compatible SDK. It uses one fixed developer instruction, then alternating visible user/assistant text ending with a user turn. External Chat Completions does not preserve private reasoning; Aster never renders, logs, stores or replays `reasoning_content`. [Reasoning](https://dev.meta.ai/docs/reasoning)

Before every request, the host validates protocol/policy 3, sole-model identity, globally exclusive request/replay limits and full context eligibility. Native terms/eligibility acknowledgement is owner attestation, not verified account access. The host builds one immutable body and displays its exact bytes in a default-deny owner-window native review. After approval it durably reserves budget and persists the pending transcript, reads the native Meta credential, then durably marks possible dispatch before transmitting that allocation. A proven-unsent key/preflight failure can release a reservation; uncertain network failures cannot. A renderer Boolean, document, generated answer or restored record cannot manufacture approval.

On 3 October the owner shared a pay-as-you-go API-key-list screenshot and reported Contributor access. A key listing and owner report are not independent model-entitlement, API-authentication or paid-inference PASS. No real credential has been accessed or paid request executed; live financial/data gates are still unsatisfied. No credential values, key identifiers or secret-file contents are reproduced in this evidence.

Contributor permits training and excludes personal, confidential and sensitive data; `store:false` is not a privacy exemption. Only host-admitted synthetic or eligible public text may be sent, and public availability alone is insufficient. No project content is automatically appended. [Terms, 2 October 2026](https://dev.meta.ai/legal/terms-of-service), [ADR-0012](../adr/ADR-0012.md)

## Streaming closure and outcomes

Meta documents data-only SSE and an exact `[DONE]` marker. Usage, when requested, arrives in a final empty-choices chunk and may be absent on interruption. [Chat Completions schemas](https://dev.meta.ai/docs/api-reference/chat-completions/schemas), [Streaming responses](https://dev.meta.ai/docs/cookbook/streaming-responses)

The following stricter rules are **Aster policy and fixture expectations**, not claims that Meta advertises every local bound:

- Decode complete LF/CRLF SSE frames only, preserving UTF-8 fragmented across reads. Bound raw bytes, frame size/count and accumulated visible text.
- Require fixed non-empty chunk `id`, exact `model`, constant `created`, `object=chat.completion.chunk` and single choice index 0. Reject unknown fields outside the closed supported shape and duplicate JSON members, including escaped-name duplicates.
- Admit null/absent usage on ordinary chunks. An empty `choices` usage chunk is admitted only after a valid finish, at most once. Invalid or duplicate usage cannot become settlement data.
- `stop` needs non-empty visible text, valid terminal marker and clean HTTP body closure. `length` is truncated; refusal/filter output is refused, including content-free filter/truncation outcomes. Returned tool/function calls deny.
- After `[DONE]`, continue bounded reading to HTTP EOF to reject trailing/duplicate terminal events. EOF before valid terminal framing fails; keeping partial text does not prove completion.
- Cancellation, timeouts, HTTP errors, malformed input and premature EOF retain available partial visible text with an explicit non-completed outcome. No automatic continuation uses failed, cancelled, truncated, refused or interrupted output.
- Safe local error classes replace provider error bodies; raw responses, keys, private reasoning and user text are not diagnostic log material.

The host receives a structured result: visible answer, `Completed | Truncated | Refused | Cancelled | Failed`, optional safe failure, independently validated optional token usage and a marker that dispatch may have occurred. Cancellation cannot silently become completed continuation. A possible-dispatch marker is not proof of billing.

## Limits and financial settlement

| Resource | Aster ceiling |
| --- | --- |
| User input / reviewed body and context | 32 KiB UTF-8 / 512 KiB |
| Completed continuation / process identifiers | 20 turns / 32 conversation IDs and 64 request IDs |
| Concurrent request | One globally active request |
| Completion | 8,192 tokens including reasoning |
| SSE frame / raw wire / answer | 256 KiB / 8 MiB / 512 KiB |
| SSE events | 262,144 |
| Connect / headers / frame inactivity / total | 10s / 30s / 60s / 900s |

Reject excess; do not silently trim/summarise. These local values are independently versioned policy, not Meta service quotas.

Usage validation requires non-negative integer counts, `prompt+completion=total`, prompt/total at most 1,048,576, completion at most 8,192, cached input no greater than prompt and reasoning no greater than completion. Optional absent details imply no claimed cache discount or extra reasoning charge. Reasoning is already part of completion; never double-count it. Invalid/duplicate/contradictory usage clears settlement eligibility. Missing usage can coexist with a completed visible answer but leaves spending unresolved.

The independent financial journal uses a **£89.00 monthly admission ceiling and £80.10 warning**, not an account balance. Native-confirmed dated owner FX/margin/tax/payment assumptions are required; no fabricated conversion is allowed. The current dated price/terms policy expires at **2026-11-02T00:00:00Z** and configuration cannot exceed it or 31 days. Conservatively reserve USD 0.106496 before conversion, assuming maximum input and completion with no cache.

Settlement uses consistent request/model-bound usage independently of terminal classification: a stopped/failed request may still be charged, and validated usage already received must not be lost solely because cancellation follows. Unknown usage retains its reservation across restarts and month changes, and any unresolved reservation blocks fresh dispatch. One exact reservation may be reconciled only against an independently native-confirmed owner statement/dashboard charge for its original account and quote; this is labelled owner-confirmed, not API-measured. No retry, history deletion, credential rotation or restart frees ambiguous spend. [ADR-0014](../adr/ADR-0014.md)

## Restart and fixture boundaries

The owner's persistent-history decision overrides the plan's ephemeral-only assumption. Display history uses a separately protected host store; restored complete text is merely candidate context and must pass fresh admission/full-body review. Incomplete restored requests become interrupted. No active request, grant, approval, credential or money authority is deserialised. Removal is a native-confirmed tombstone, not secure erasure; financial and historical recovery records remain independent. [ADR-0015](../adr/ADR-0015.md)

Offline fixtures cover exact reviewed bytes, closed model/effort/field admission, zero dispatch on pre-cancel/invalid body, split UTF-8/CRLF, wrong identity/object/model/index, unknown or duplicate JSON, usage-only termination, missing/impossible/duplicate usage, refusal/truncation/tools, duplicate/trailing terminal, cancellation before headers and during an open stream, premature EOF, bounds and local HTTP error classification with no retry. Private loopback seams and known synthetic keys are test-only; they are not configurable production destinations. The integrated suite must establish its own source-bound outcome before promotion.

## Retired v0.1.0 transport evidence

The following original record is **RETIRED**. Its NVIDIA origin, protocol 2, defaults, capabilities, authorisations and PASS reports belong only to the named v0.1.0 source/artifact. They do not register a provider, transfer authority, authorise a new request or demonstrate v0.2.0 acceptance. Legacy provider-specific pages remain historical with retirement banners; old credentials are untouched.

<details>
<summary>Original v0.1.0 contract and release evidence (retired)</summary>

# NVIDIA hosted chat transport contract (draft v0.1.0)

**Reviewed:** 2026-10-01 UTC against the official model/endpoint pages; acceptance status reconciled 2026-10-02, Edinburgh (`Europe/London`, BST, UTC+01:00). **Scope:** Aster Desktop's host-side adapter for exactly two allowed NVIDIA-hosted model IDs. Final source/Windows package binding is PASS at source `1782d3375db47466f89be6bd3cd3c6affbfa95b7`, main CI run `37040168700` attempt 1, artifact `11243338033`. The owner reports **providers/history PASS** on the matching delivered/installed build; see [OWNER-ACCEPTANCE-0001.md](../qa/OWNER-ACCEPTANCE-0001.md). This category-level owner report is distinct from independently recorded per-model canary/transport details, which are assigned to [v0.2.0 review](../qa/V0.2.0-FOLLOW-UP-0001.md) where not supplied. It is not production certification.

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

Only send user-visible text and specifically approved context to NVIDIA. Request creation and credential access are host operations; IPC accepts typed user intent, model ID, conversation ID and bounded content, never a URL, key, arbitrary request headers, transport options or an unbounded provider JSON body. Do not dump raw requests or responses in diagnostics. A configured-key flag may be returned to UI; no key bytes may be returned. For a selected project file, the renderer receives metadata only. The host composes at most 32 KiB of project-bearing prompt, serializes the exact body, displays those exact bytes in an owner-HWND native review, consumes a one-use approval and transmits that byte sequence once without reserialization. The project prompt and derived answer are excluded from continuation. Neither this adapter nor its streaming parser directly writes project files.

Treat prompt, generated response, optional `reasoning_content`, usage and errors as potentially sensitive. Separate reasoning from assistant `content`; do not turn reasoning text into a UI answer, an instruction, a log field or an edit operation. Rendering or retention of reasoning requires a separate product decision. Keep per-conversation continuation isolated, respect each model's role order, and never mix response chunks from concurrent request IDs or models. Sampling `seed`, if later enabled, only promises NVIDIA's best-effort repeatability and does not prove deterministic prose.

## Streaming state machine and bounds

The user requested `stream: true` for both models. NVIDIA documents data-only server-sent events (SSE) with `data: <JSON>` frames and terminal `data: [DONE]`. Host-only state progression is `ADMITTED -> CONNECTING -> STREAMING -> {COMPLETED, CANCELLED, FAILED}`. Enforce exactly one terminal event for a request; cancellation drops subsequent chunks and closes the stream; a missing `[DONE]` or transport EOF is `FAILED`, never silent success. Decode UTF-8 across byte boundaries and SSE frame boundaries. Accumulate answer `delta.content` incrementally, collect any `delta.reasoning_content` on a separate channel, skip empty `choices` when valid, enforce `choices[].index` consistency, and distinguish `finish_reason=length` from complete answer. No project edit may be committed on truncation, parsing failure, cancellation or partial output.

Current versioned host ceilings are: connect 10 seconds, first response headers 30 seconds, idle between frames 60 seconds, total request 900 seconds, one SSE event 256 KiB, total response 8 MiB, accumulated answer 512 KiB and 262,144 frames. The provider request body is capped at 512 KiB; ordinary input/context and project-composition limits are specified separately in SPEC-0001. Protocol-2 fake-server/exact-body fixtures pass locally at immutable runtime source `2bcacd7c6f2496eda233d4a069ec7b65a93d47b3`; completed main-branch CI/package binding is now PASS as identified above. The owner's provider/history PASS is recorded separately from detailed final-artifact per-model canary evidence. Adjust only by explicit policy/spec/test revision. These values are Aster policy choices, **not** NVIDIA-advertised limits.

For `stream=true`, require a successful HTTP status and `text/event-stream` framing; parse each `data:` payload as JSON or the exact `[DONE]` marker. Reject invalid UTF-8, malformed JSON, oversized frames/body, contradictory IDs/model/indexes, unexpected tool calls, duplicate terminal outcomes and impossible state transitions. Unknown harmless JSON metadata may be ignored after bounds checks; unknown actionable message types never expand authority. If a provider returns `202` pending rather than an SSE stream, mark the feature unsupported with a clear error until its polling contract is separately specified. Map authentication, validation, rate limit/quota and server failures to user-safe error classes without copying error bodies or keys to logs. A partial answer displayed before a failure must be marked incomplete. If usage is omitted or arrives only in a final chunk, record it as unavailable until an authoritative count is seen; never invent cost or remaining quota.

## Fixture matrix and promotion rule

Use a loopback fake server with a fake key for each model. Serialize and snapshot the *exact* JSON request, model ID, headers (redact the fake token in snapshots), selected params, and `stream=true`; assert zero network calls for disallowed IDs or missing credential. For project disclosure, hash/compare the exact owner-reviewed body with the loopback-received bytes and assert one dispatch with no rebuild/retry. Drive normal chunks with answer and separate reasoning, empty choices, split UTF-8/frame delimiters, fragmented `[DONE]`, missing usage, `finish_reason=length`, large and malformed frames, missing terminal marker, wrong model/index/ID, unexpected `tool_calls`, cancellation during connection/stream, idle timeout, HTTP 202/401/403/422/429/500, redirect response, and proxy configuration. Assert secret/project-text absence from renderer IPC, logs, errors and test artifacts. Do not convert fixture PASS into live-provider compatibility.

The historical pre-installation checkpoint recorded live canaries for each model as `NOT RUN`; retain that dated result. On 2026-10-02 the owner reports providers/history PASS for the delivered engineering build, without per-model request counts, timestamps, streaming/terminal traces or usage fields. Record this as owner-reported category acceptance, not a fabricated one-request-per-model canary result. Detailed model-specific runtime verification not covered by that report is explicitly deferred to v0.2.0 review. The earlier authorization allowed exactly two minimal plain-text requests, one per exact model, using the existing Windows Credential Manager key; it is not evidence of execution, evidence that the budget remains unused, or authorization for agent retries, additional requests, project upload, live cancellation/error cases or fallback. Any future execution needs its own applicable authority and availability/terms/pricing review. Source fixtures, installed owner acceptance and production/live contract certification remain distinct.

## Source evidence

- NVIDIA [Nemotron 3.5 Lightning endpoint schema](https://docs.api.nvidia.com/nim/reference/nvidia-nemotron-3-5-lightning-30b-a3b-infer), reviewed 2026-09-24.
- NVIDIA [Muse Glimmer endpoint schema](https://docs.api.nvidia.com/nim/reference/meta-muse-glimmer-30b-infer), reviewed 2026-09-24.
- NVIDIA [Nemotron hosted code example](https://build.nvidia.com/nvidia/nemotron-3.5-lightning-30b-a3b), reviewed 2026-09-24.
- User-provided endpoint examples: `01-nemotron-3.5-lightning-30b-a3b.txt` (SHA-256 `826261f2c1ae3ebba7f365b7ad8a865ad46f2e0b3ecb347a35c1e22482db63dc`) and `02-muse-glimmer-30b.txt` (SHA-256 `9a44533e93b5fdd645e174e6a6f4f397216e1090d0ff4c1c1d8220fbb0562407`); both contained only a literal `$NVIDIA_API_KEY` placeholder.

</details>
