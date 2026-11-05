# Current host transport — v0.2.4

Current implementation reviewed against PR #70 head `c6974cd`, 5 November 2026. Financial policy: **4**; IPC protocol: **5**. [Maintenance and final delivery receipt](https://github.com/comzulca/Aster-desktop/pull/70). The previous `57339a6` receipt describes its original bytes only. [Current status](../ENGINEERING-STATUS.md), [engineering receipt](../qa/FINAL-ENGINEERING-V024-20261105.md), [provider pair](meta-muse-spark-1.3-contributor.md), [attachments](attachments.md).

## Fixed routes and authority

| Purpose | Fixed host route | Meaning |
| --- | --- | --- |
| Inference | POST `https://api.meta.ai/v1/chat/completions` | Only `muse-spark-1.3-contributor`; native exact-body disclosure and durable spending admission before dispatch. |
| Explicit key validation | GET `https://api.meta.ai/v1/models/muse-spark-1.3-contributor` | Bounded model receipt for current-key authentication/model visibility; not inference, billing, quota or universal entitlement. |

Bearer credentials stay in the native Windows path, not renderer IPC, prompts, logs, source or CI. The HTTP client has fixed destinations, disabled redirects and bounded operation/body/stream handling. There is no generic URL override, automatic retry, provider/model fallback or model-executable tool. GitHub's separate closed broker is not part of Meta inference and does not authorise private repository disclosure.

## Request construction and review

[provider.rs](../../src-tauri/src/provider.rs) owns serialisation and strict admission. Its fixed developer instruction precedes eligible alternating visible user/assistant continuation. Only completed eligible turns are candidates; the host re-admits all selected prior content. Personalization, private GitHub content, packages, keys, grants and provider private reasoning are not implicitly added.

The closed body retains `model`, `messages`, `stream`, `stream_options`, `reasoning_effort`, `max_completion_tokens`, `n` and `store`: streaming is true, usage is requested, completion is bounded to 8,192 tokens, `n=1` and `store=false`. Reasoning accepts only `minimal`, `low`, `medium`, `high`, `xhigh`, with `medium` the explicit default. `store=false` does not exempt Contributor disclosure from its data-use rules.

Text-only and mixed-media messages use the same fixed endpoint. Text attachments become inert processed text; images use normalised complete image content and PDFs use the immutable complete temporary original with a host-chosen safe attachment filename. No file-upload endpoint, provider-held conversation ID or renderer-supplied original path is introduced. Complete immutable outbound bytes and native visual previews are bound to review; sending does not rebuild an equivalent body after approval.

## Separate text and wire limits

| Boundary | Limit |
| --- | --- |
| User/composed text message | 32 KiB UTF-8 admission |
| Runtime text context and text-only serialised request | 512 KiB; at most 20 completed continuation turns |
| Complete media serialised request | 24 MiB, in addition to text/source/media limits |
| Inline sources | Four files, 8 MiB each / 16 MiB total |
| Completion / visible answer | 8,192 tokens including reasoning / 512 KiB visible text |
| Stream frame / complete response / event count | 256 KiB / 8 MiB / 262,144 |
| Key-validation receipt / operation | 32 KiB strict JSON / 15 seconds |

Base64 and JSON overhead count towards the media wire bound. The model's documented 1,048,576-token context is not an implemented full-window admission counter or migration. Exceeding bounds denies the operation instead of silently truncating, summarising or changing protocol.

## Responses, cancellation and finance

Strict JSON/SSE handling rejects duplicate/malformed or incompatible fields, unsupported tool operations, oversized data and incomplete streams. Completed, refused, truncated, cancelled, failed and interrupted outcomes remain distinct; partial output is not automatic continuation. The exact field/finish/usage fixtures in [provider.rs](../../src-tauri/src/provider.rs) are the executable contract.

Token counts must be non-negative integers with consistent prompt/completion/total and bounded cached/reasoning detail. Reasoning is included in completion, not added again. Visible completion and validated financial usage are independent: a completed answer can still leave usage unresolved. Cancellation or a network/parse failure after possible dispatch does not establish zero charge.

The protected journal reserves conservatively before networking and marks possible dispatch durably. Only a proven-unsent failure can automatically release its reservation; missing/invalid/duplicate usage keeps uncertainty and blocks new dispatch. Owner statement reconciliation is separate exact native review against the original quote/account/period. See [budget.rs](../../src-tauri/src/budget.rs), [setup](../SETUP-AND-USE.md) and [ADR-0014](../adr/ADR-0014.md).

## Current policy and evidence

[contributor_policy.rs](../../src-tauri/src/contributor_policy.rs) centralises terms 2026-10-09, geography 2026-10-12, review 2026-11-05 and local deadline 2026-12-06T00:00:00Z. Legacy acceptance/financial events retain their original window; current acceptance/configuration needs fresh review. This is Aster's review boundary, not Meta key/service/subscription expiry.

Initial complete maintenance run `42109052799`, attempt 1, tested merge `8e92ef7`: all five jobs, 184 frontend/414 Rust cases and the actual fifteen codec/six resource probes PASS. A separately authorised local synthetic stream using the unchanged request envelope returned HTTP 200, final stop, DONE and consistent usage. It did not run Aster's installed native review or qualify account/territorial rights, other modalities, invoice limits, all 42 compound obligations or production. [Observed successor evidence](../qa/SEND-READINESS-20261105.md#observed-successor-verification) and the [PR delivery receipt](https://github.com/comzulca/Aster-desktop/pull/70) distinguish candidate and final main bytes. The previous [57339a6 receipt](../qa/FINAL-ENGINEERING-V024-20261105.md) remains source-bound history.

## Retired v0.1.0 transport evidence

NVIDIA/Nemotron/Glimmer and original v0.1.0 details are historical only. The complete former transport document is preserved byte-for-byte in [transport-history-57339a6.md](transport-history-57339a6.md), including its [retired transport appendix](transport-history-57339a6.md#retired-v010-transport-evidence). Historical PASS, grants, keys and routes do not reactivate a provider or qualify changed source.
