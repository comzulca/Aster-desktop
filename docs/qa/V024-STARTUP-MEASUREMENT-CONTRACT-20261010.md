# Startup readiness measurement and staged reduction — 10 October 2026

Before-code NFR-010 / UX-017 / TM-032 / AC-076 / T-086 refinement. Root reviewed the existing source and two independent read-only reviews before approving this capture-only slice. The owner prioritises absence of recurrent flicker over speed and authorises measuring readiness, then progressively evaluating reductions with repeated evidence. They personally install and run the Windows builds. No agent app/setup launch, live call, credential/history change, security/GPU change, merge, release or signing is authorised.

## First slice: unchanged presentation

Keep the 6,000-ms minimum from the existing gate clock and 10,000-ms independent fail-visible deadline unchanged. Preserve hidden initial creation, opaque dark native/WebView/early CSS backgrounds, the one-shot UI-thread show, owned activation veto and cleanup, late/duplicate readiness handling, deadline priority and the single deadline attempt after early failure. Images settling, two animation callbacks, controller visibility and native visibility are not proofs of presented pixels. Hydration remains independent. There is no configurable delay, environment override or second reveal.

Share an owned clone of the existing `RenderTrace` with `StartupGate`. All exported milestones are timestamped by that trace's existing monotonic clock, never by transplanting the separate policy elapsed times. Record these fixed **native** categories only:

| Category | Observation boundary |
| --- | --- |
| `startup_gate_created` | Immediately after the existing gate clock/state is created; an observation, not an exact timestamp of process launch. |
| `startup_shell_accepted` | First accepted `!ready && Waiting` state transition only. |
| `startup_reveal_ready`, `startup_reveal_deadline` | The existing admitted reveal decision and its reason. |
| `startup_show_started` | Entry into the already-dispatched owning UI-thread closure. |
| `startup_show_visible`, `startup_show_denied` | Return of the unchanged guarded show, without cleanup failure. |
| `startup_show_visible_cleanup_failed`, `startup_show_denied_cleanup_failed` | The corresponding returned visibility with failed hook cleanup. |
| `startup_dispatch_denied` | UI dispatch rejection; no fabricated UI-start or native-show result. |

Retain the existing first `parent_visible` event/origin. Shell admission and decision markers are emitted while the existing policy transition is serialised; the only nested lock direction is **Policy → Trace**. Never hold the policy mutex across OS calls, callbacks or awaits. Trace failure must not affect presentation. A closure that never runs, destruction or panic leaves incomplete evidence; no invented result or extra retry.

Before publication, preserve the original receiver-before-argument evaluation order: the existing gate elapsed time is sampled **after acquiring Policy**, not before waiting for its mutex. A private immediate clock supplier may preserve this ordering in the wrappers, with synthetic closures only in pure fixtures. It is not a public timing override or a new clock origin. Add a no-sleep fixture proving the supplier runs while the policy lock is held. An earlier read-only review/format checkpoint precedes this narrow source correction and cannot qualify it.

The schema-1 fields, exact authenticated IPC objects and six renderer categories stay unchanged. Synchronise the Rust native enum, frontend closed union/validator and fixtures. Older validators correctly reject unfamiliar categories; source/artifact identity is therefore required. No content, path, URL, identity, raw native error, key state, extra file logger or automatic export is added.

## Fixed budget and saturation

Supersede only the earlier one-slot admission budget, preserving its original dated text. Within the same **96 events / 60,000 ms**, reserve up to nine slots: gate 1, accepted-shell 1, decisions 2, UI-starts 2, mutually exclusive outcomes 2 (dispatch denial shares an outcome slot), first parent visibility 1. Private per-kind counters bound consumption; surplus critical calls are dropped. Ordinary observations can initially occupy 87 entries. Used reservations reduce the remaining reservation; unused slots remain reserved, even after visibility, rather than evicting retained observations or releasing capacity speculatively. Keep monotonic retained order, immutable first native visibility and saturating dropped accounting. Renderer chatter cannot consume critical reservations. Beyond 60,000 ms no category or origin is recorded. Dropped ordinary observations still mean incomplete evidence; reservations do not establish complete capture or painting.

Required additive fixtures: native vocabulary/renderer forgery, first admission versus replay/late/forged callers, unchanged minimum/deadline/one-shot race, early failure and deadline retry, decision/UI-entry/outcome ordering, dispatch rejection without UI-entry, all critical slots retained after ordinary/renderer saturation, duplicate-kind bounds, expiry and unchanged origin. Preserve every existing presentation assertion and the privileged-route/security boundaries. Pure fixtures are not native acceptance.

## Installed experiment and promotion rule

First qualify and deliver a distinctly labelled **measurement baseline**, not the final installer alias. Independently observe its exact head/base/tested merge, all five jobs and the completed actual 15-codec/6-resource/causal CPU receipt with `qualificationComplete=false`; passively verify its bytes/resources without launching it. Retain previous installer bytes and recording unchanged. PR publication/check authority does not allow merge.

For each candidate, require the owner to identify exact setup SHA/source, Windows/WebView2 versions and each launch, capture the whole window from before reveal through at least 15 seconds after visibility, and provide the complete local JSON. Require **at least ten idle launches: five ordinary restarts and five first launches after separate owner-operated Windows restarts**. On 10 October the owner expressly agrees to perform all ten. Record cache/restart conditions honestly; ordinary app restarts are not guaranteed warm and Windows restart is not guaranteed cold. Do not clear caches/history or alter GPU/security settings. Also verify controlled resize/zoom/close and failed-readiness/deadline safeguards separately. One clean recording is encouraging but not repeated acceptance.

Evaluate the baseline distribution of gate observation → shell acceptance, gate observation → reveal decision, decision → UI-entry and UI-entry → outcome; record missing/overflow/failure samples separately. Millisecond quantisation and scheduler effects apply. Gate observation is immediately after actual gate creation; it is not the exact gate start. Queue and guarded-show intervals are not compositor latency. Compare pixel changes at +2/+5 seconds after appearance, excluding pointer/recorder overlays and recording compression. A 30-fps video cannot exclude sub-frame flashes or certify physical scan-out.

Only after complete repeated baseline evidence and a reviewed readiness margin, propose one fixed candidate at a time, tentatively 6,000 → 4,000 → 2,000 ms; these numbers are **not implemented, selected or qualified**. Never infer that 399-ms image readiness makes 400 ms safe. Each change needs a new before-code decision, exact-source five/15+6 gates and the same repeated installed protocol. Stop at any flash, unavailable readiness, cleanup/presentation regression or unresolved evidence; retain the preceding qualified candidate and preserve failures. A reduced value is adopted only after repeated comparison, not averaged away failures. Further reductions require another reviewed step; no unconditional zero-delay setting.

Current result: capture design approved; measurement implementation and changed-source execution initially NOT RUN. Repeated new-milestone owner baseline and every progressive reduction are NOT RUN. The older single owner recording is a bounded clean observation, not a causal cure or proof of a safe shorter wait.
