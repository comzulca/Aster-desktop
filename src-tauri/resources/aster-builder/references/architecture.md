# Authority and deterministic state
## Authority matrix
| Input/action | UI | Model or retrieved content | Trusted host | Human |
| --- | --- | --- | --- | --- |
| Select provider/intent | Request | No authority | Validate against fixed registry | Choose |
| Store/load secret | Never access bytes | Never access bytes | OS-backed custody; provider-scoped lookup | Enter through isolated trusted path |
| Network request | Show status | Supply bounded content only | Fixed destination, serializer, parser and limits | Authorize provider disclosure and live use |
| Read project file | Request grant | Propose closed read call | Check grant, root identity and bounds | Scope grant |
| Change project file | Display diff | Propose change only | Stage, recheck identity/hash and recover | Confirm exact change separately |
| Open link/web/MCP | Display user action | Untrusted URL/data | Apply destination/rights/SSRF policy | Activate capability deliberately |

Do not use a renderer-only check as the host's security boundary. No global shell or wildcard filesystem permission. Prefer a single owner of each capability; narrow every crossing with a versioned DTO and reject unknown fields, forged callers, oversized content and malformed framing.

## State machine and repeatability
Use explicit states: UNCONFIGURED -> READY -> REQUEST_RESERVED -> STREAMING_OR_TOOLS -> TERMINAL; optional PROPOSAL -> AWAITING_HUMAN -> COMMITTING -> VERIFIED or ATTENTION_REQUIRED. Tie every event and cancellation to the request/conversation/provider pair; deliver exactly one terminal outcome. Make grant consumption, expiry, replay checks and resource ceilings host decisions. Test the same fixture twice, including duplicate and out-of-order messages, and compare decisions and persisted state (not model prose).

Start with no provider origin, tool, file root or web destination admitted. Version the policy; pin exact dependencies and fixtures. Record authorized deviations as ADRs. For real OS resources use identity-safe primitives appropriate to the chosen system; an app's language/framework is a parameter. Distinguish internal agent instructions, repository governance and in-product runtime prompts: none grants authority by its mere presence.
