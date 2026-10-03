# THREAT-MODEL-0002 — Contributor and persistent history

Design date 2026-10-02. Supersedes the active profile of THREAT-MODEL-0001; historical threats/results remain records, not enabled capabilities. Assets: API key, admissible text, stored conversations, financial reservations, native approvals. Trust boundaries: untrusted WebView/model -> typed host; host -> native owner-window review; host -> fixed Meta HTTPS; host -> current-user protected private records. The model never owns any grant, money, secret or write operation.

| Threat | Control and trace | Residual risk / owner |
| --- | --- | --- |
| TM-016: prohibited data through prompt/continuation | SEC-016, full-body local admission and default-deny exact native review; AC-035/T-045 | Human/filter omissions, not universal DLP; security/host |
| TM-017: legacy pair or stolen-key destination | SEC-014/015, sole pair, distinct Meta key, no old IPC/redirect/proxy/override; AC-033/041/042 | Same-user or OS compromise; provider integration |
| TM-018: lost reservation or cancellation treated as free | SEC-018, flushed reserve/dispatch/settle journal, integer accounting, exclusive lease, unknown usage retention; AC-036/037/044 | Account-external spend, price changes, same-user rollback; financial owner |
| TM-019: restored protocol/grant/approval authority | SEC-019/022, protocol 3, strict schema, display-only restore, new native review and no old model import; AC-038/045/050 | Historical corrupted state requires attention; architecture |
| TM-020: false connection/privacy/tool promises | UX-003/004/005, truthful composer/settings, false project flags, no placeholder actions; AC-039/051 | User misunderstanding; UX |
| TM-021: unsupported account/territory/tier | PR-003, native terms/eligibility gate and independent live acceptance; AC-032/049 | Provider/account policy changes; owner |
| TM-022: persisted text disclosed or overwritten | SEC-021, separate DPAPI entropy/domain, owner protected DACL, identity-pinned NTFS paths, bounded immutable records, native delete; AC-050/052/T-060/062 | Same-user access and physical-loss durability not guaranteed; persistence owner |
| TM-023: clock/config/store manipulation frees spend | SEC-018/021/022, closed configuration, no reset API, monotonic checks/explicit period reconciliation and unknown-state denial; AC-036/044/052 | Compromised same-user rollback cannot be detected universally; financial owner |

No remote classifier, project forwarding, tool execution, automatic live test, key export or security-setting change mitigates these threats. File-creation journal schema, old credentials and old evidence remain untouched. New private stores cannot reuse/widen the creation journal. Negative fixtures must prove denial before network and zero regained authority on restore. Physical power-loss and interactive owner acceptance stay NOT RUN until separately observed.

TM-017 compatibility review, 2026-10-03: opaque `LLM_` acceptance must not become a generic Bearer-key or alternate-provider route. Keep a closed prefix/alphabet/length contract, deny header injection and unmodified malformed native inputs, retain both decoder bounds and secret scanning, and never parse the opaque string as an account identity. Existing native consent, separate key target, fixed Meta origin and inference/finance gates remain mandatory. A separate authenticated catalogue observation does not enable application dispatch or replace native/inference acceptance.
