# v0.2.4 owner-reference UI conformance — 10 October 2026

Status: **main-agent reviewed and approved before code at 2026-10-10 13:30:10 UTC**. Refines SPEC-0006 UX-016 / AC-074 / T-084, new UX-019 with FR-031/032 / AC-078,079 / T-088,089, and retains UX-018 / AC-083 / T-093 for separate Personalization. It adds no provider, persistence, IPC or tool authority. No frontend behavioural/style change is made by this document. Baseline HEAD is `d8b9422eab83526a67bd16a23fbb0d029870d709`; the candidate is an uncommitted shared working tree, not a qualified build.

## Visual authority and observed gap

The owner explicitly requires their new interface throughout v0.2.4. `Aster_UI_0.2.4.png` governs the chat workspace; `Aster_Plugins.png` and `Aster_Skills.png` govern the Customize workspace structure; the three Personalization references govern that separate Settings page. Old screenshots and the Aster Builder default JPG do not override these references. Screenshot text about unrelated providers/services or sending personal data does not authorise those capabilities.

The read-only review found Customize embedded in Settings with horizontal category tabs and a single-column catalogue, rather than the supplied full-page category subrail/two-column composition. Current main CSS also uses 320 px rail, 965 px composer and 134 px orb maxima, while the 1914-pixel chat reference visibly measures approximately 400 / 1206 / 166 image pixels. This is a comparison of static source and references, **not** an observed native rendering failure or a new pixel-match PASS.

Reference provenance, read locally without changing the files:

| Owner file | SHA-256 |
| --- | --- |
| `Aster_UI_0.2.4.png` | `295e46c8025045db8763f7717c00e1d98521533bfc57c8f1afd3af31e6b7c747` |
| `Aster_Plugins.png` | `4a7a761d4e14b421d3cea5a709de61ad2a744328aaa9be4df0ece7c8a5d41cf1` |
| `Aster_Skills.png` | `63070e93a5f38414abe0ca11c534b36533d5e08898d86cfa052b93dcea012056` |
| `Personalization1.png` | `7245cdc39aa9416435c568215efbe1b44908ab945217232f20182e90e13d51d1` |
| `Personalization2.png` | `b8ea71f4fe0af4591263144e52dcc45e71356ee3b15bf08dd076b5fe24e3c058` |
| `Personalization3.png` | `46b2bea46eaecf9d6f0b1a271ce5d1b3fe2c00213119aed452c3bee3dcb3a9c4` |

These files are owner-supplied visual references under `C:/Main/Aster-desktop-Aster_builder_skill_on/`; this amendment does not add them to installer resources or assert unrelated third-party redistribution rights.

## Coordinate-aware chat geometry

Record the viewport in CSS pixels, device-pixel ratio/Windows scale and local Aster interface zoom for every rendered comparison. Image pixels cannot be silently treated as CSS pixels. The three ratios are close to 1.25, suggesting a possible 125% device scale; the screenshot's original scale has **not** been measured. Do not change Windows settings or set a global 125% interface zoom to imitate the reference.

At the explicit **1914-CSS-pixel wide, 100% local-zoom fixture**, target approximately 400 px conversation rail, 1206 px centred composer and 166 px welcome orb. Use adaptive reference-normalised proportions (approximately 20.9%, 63.0% and 8.7% of viewport width) with safe minima/maxima and the existing compact-navigation breakpoints. Around 1531 CSS pixels these proportions give approximately 320 / 965 / 133 px; at a separately recorded 125% device scale their physical appearance approaches the reference. This avoids double enlargement from an unverified scale assumption.

Preserve the uniform dark canvas, actual existing Aster artwork/crop, one top menu/caption row, Union Jack beside native caption controls, white glowing transparent '+' glyph, compact adjacent model label, centred empty state, blue send/stop treatment and existing message/classification/privacy/footer notices. The constrained viewport may reduce geometry or collapse navigation rather than clip the composer, conceal the key warning or introduce horizontal scrolling. No viewport or zoom change may authorise a send or import.

## Full-page Customize workspace

Move Customize out of the Settings content column into a full-page local workspace beneath the same caption/menu row. Retain an explicit route from Settings and an accessible return-to-chat control. Workspace navigation must preserve the selected main conversation, unsent per-chat draft, classification and temporary attachment identifiers; it must not create/load/adopt a conversation, send a message, consume originals or connect an account merely to change presentation.

Follow the supplied composition: narrow local navigation/return affordance, a left **Customize** category subrail with Plugins and Skills, and a spacious right content panel with the selected category title, description, search and explicit Add/import/generate actions. At wide widths use two equal catalogue-card columns; fall back to one column when two readable cards no longer fit, and adapt the subrail on compact windows. Category navigation retains accessible selected state, named controls, arrow-key behaviour and visible focus.

Show only real built-in or imported catalogue entries. Aster Builder remains a read-only instruction entry with a separate enable preference; GitHub remains the sole reviewed executable connector, with its actual enable slider and account-management popup. Unrelated services shown in the references (Gmail, cloud environments, image tools, marketplaces, browser extensions and so on) must not appear as falsely installed/installable features. Public/personal/system marketplace filters must not suggest unavailable catalogues. Imported packages and enabled instruction preferences remain inert; labels describe their actual availability.

Reuse the current bounded editor/import/generation contracts and existing strict DTO validators. A completed validated Muse result opens an **unsaved** editable draft; Save and enable stay explicit and independent. No package installation, script, hook, MCP, automatic account connection, mutation or retry is introduced by a card/action. Writes and compaction remain visibly unavailable until separately implemented and qualified; no flicker cure is claimed.

Busy, modal and unsaved-draft guards apply across the workspace transition. Close/unmount of generation or GitHub popup keeps the existing owned-request cancellation and epoch invalidation; stale responses cannot update another workspace or create/save a draft. Background chat shortcuts cannot submit hidden content. Returning to chat is a local presentation action, not a restore of approval or tool authority. Preserve the sole authenticated native window-action route and bounded 80–150% interface zoom; no new window, clipboard, browser or generic IPC route is added.

Component-boundary refinement approved by the main agent before its code change: Personalization may report only an optional **busy-or-unsaved boolean** to App for the Settings → Customize guard. Profile contents remain entirely inside its existing component/store domain; no DTO, persistence, egress or authority change follows. A dirty or busy profile denies that transition until existing explicit Save/Clear succeeds or the owner explicitly closes Settings with the existing disclosed unsaved-discard behaviour. Do not infer dirtiness from DOM fields or pass profile values to navigation, Customize or generation. Focused source fixtures must retain unsaved values and deny abandoning an in-flight local change through the new transition.

Main-agent read-only review subsequently found that inherited shell focus/select/change capture could copy Personalization field values into App's edit target, despite the intended boolean-only boundary. No outgoing leak was identified; this remains a real component-isolation gap, not a qualified privacy PASS. Before its narrow correction, exclude every `.personalization-page` field from global edit/selection capture **before reading its value or selected text**, clear stale global edit-target/selection references, and preserve ordinary native local field editing. No profile value may enter App draft/edit-history/clipboard buffers through those hooks, including forged local edit-key attributes. Add focused authored-source and UI canary fixtures. Preserve the earlier 160-test receipt as source-bound history and rerun changed-source TypeScript, complete frontend fixtures and web build; no host, DTO, persistence or egress policy expansion follows.

Main-agent approved copy-only refinement before corrective source changes: unavailable catalogue methods or unknown/unverified account receipts cannot establish an empty catalogue, disabled existing preferences or an absent active account. State only that no change was performed by this view until a validated native receipt supports a narrower statement. Distinguish disabled main-chat/model/local-project/general-web tools from the separate explicit native-reviewed GitHub account/repository reader; that route grants neither mutations nor Meta disclosure. Current-version News must describe the approved owner-reference layout, local-only profile/inert catalogue, current-key admission, eligible-description-only generation with ordinary local prompt/response history, and separate GitHub enable/connect with deliberate reconnect after expiry/restart. Writes, automatic compaction and flicker cure remain unavailable/unestablished; observations are not a remedy. Display current credential-validation metadata separately from still-unverified inference, billing and capacity, without starting a probe. This amendment changes only reported UI text and focused negative/copy fixtures, not host, DTOs, state transitions, gates, profile contents or generation behaviour. Preserve the 154/160/162 receipts and run fresh TypeScript, complete serial Vitest and web build on the changed frontend snapshot.

## Settings Personalization remains separate

Retain the modern two-column Settings layout, Personalization sidebar entry, four optional fields, counters, Save/Clear and dark input surfaces from the three dedicated references. Do not move the personal profile into Customize or into any generation input. The owner's later **strict local-only** decision overrides screenshot text about transmitting preferences. Existing field/aggregate bounds, encrypted current-user storage, non-forensic Clear/retention copy and exclusions from every prompt, summary, package, tool, export and diagnostic remain unchanged.

## Verification intent and unchanged gates

Before implementation the main agent must read this complete draft, resolve any coordinate/layout ambiguity and explicitly approve the narrow frontend change. Root owns the accompanying SPEC/TM/documentation reconciliation; this draft is not implementation approval by itself.

After approval, add headless T-084/T-093 fixtures for explicit full-page entry/return, category/search/card structure, truthful installed/unavailable states, preservation of main conversation/draft/attachments, hidden-chat action denial, keyboard focus, and cancellation/stale callback behaviour. Preserve negative key/profile/generation fixtures and original failed reports. Run fresh TypeScript, complete serial Vitest and web build on changed frontend source, capturing the report/source identities; the prior 154/154 result remains source-bound historical evidence.

Rendered comparisons require separately authorised browser/headless capture or the owner's installed observation at reference-normalised wide and 1366 x 768 compact dimensions, with scale recorded. This draft does not authorise a native app/setup launch, claim raster fidelity from DOM assertions, or substitute a visual/source result for Windows, live or flicker acceptance.

Five mandatory CI jobs, fifteen codec/six native resource probes and their budgets, all 42 compound obligations, unresolved GLib, exact-artifact provenance, unsigned engineering classification and independent owner-installed/manual/live/signing/production gates remain unchanged. PR publication/check authority does not include integration or release. No requested final `Aster_0.2.4_x64-setup.exe` is qualified by this refinement.
