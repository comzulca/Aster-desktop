# FRONTEND-V021-0001 — bounded renderer implementation evidence

**v0.2.3 applicability — 4 October 2026, Edinburgh / Europe/London:** dated evidence below retains its original source, version and result; it does not qualify changed v0.2.3 bytes. Unchanged constraints remain subject to the [current documentation index](../V0.2.3-DOCUMENTATION-INDEX.md) and [SPEC-0005](../specs/SPEC-0005.md). The owner authorises a v0.2.3 PR and GitHub checks only, not merge, release or signing. Final-source, package, native and live qualification require independent evidence in [LOCAL-V023-0001](../qa/LOCAL-V023-0001.md). No retired provider, historical grant or old PASS is reactivated.

**v0.2.2 applicability, 3 October 2026:** retained requirements/contracts apply only where unchanged; retired providers, disabled capabilities and older dated results are not current authority or new-source PASS. [SPEC-0004](../specs/SPEC-0004.md), [ADR-0017](../adr/ADR-0017.md) and [LOCAL-V022-0001](../qa/LOCAL-V022-0001.md) govern active zoom/startup changes and fresh evidence. Earlier evidence remains source-bound history; the later owner-reported v0.2.1 startup flash is recorded separately. The entire original v0.2.1 checkout/installer is unchanged.

## Later visual-review corrections — 2026-10-03

The 56-case snapshot below remains historical. Headless browser review subsequently exposed mouse-opened menus ignoring Escape at the focused trigger and inverse percentage compensation breaking zoom layout. Both defects received failing regression fixtures before correction. Escape now closes the menu without losing trigger focus. Zoom uses the native CSS zoom layout with 100% dimensions, a zero grid minimum and bounded scrollable menu height; keyboard focus scrolls menu items into view. Ready artwork adapts only when the content region is constrained. No provider/credential/financial interface changed.

The Escape red fixture failed at `17:45:09Z`; the zoom red fixture failed at `17:49:43Z`. The follow-up complete suite started `17:52:33Z`: **58 tests / five files PASS**, typecheck/build/scoped whitespace check exit zero, Vite 28 modules/146 ms. The earlier direct `pnpm exec vitest` command failed before executing tests; the bundled Vitest entry point and normal complete scripts subsequently ran. Runner remains `QUARK2\CodexSandboxOffline`. Final follow-up physical hashes: App `d265eb3f3748c4c93b3ad728ee8b4600452a2d813de85caab1c0ef4d6634371f`; CSS `513d7afc59f4390e475d1735f140ed64dd1e1292110195cc9cbc7659b5e59b8b`; menu tests `77403fb456d9c7bb6069ebb6481af31f145f4612edf932723b44d7c2c148ef39`. Final source commit/full qualification belongs to LOCAL-V021-0001; headless/native results are not interchangeable.

Review checkpoint: `2026-10-03T17:36:51Z` / Edinburgh `18:36:51+01:00`. Source: isolated v0.2.1 worktree based on main `46fd0729a2ad69b7af4929da832e44649b439f38`, with the uncommitted renderer files identified below; this is not a final source-commit or installer record. Runner: `QUARK2\CodexSandboxOffline`, Windows shell, Node `24.19.0`, pnpm `11.19.0`, Vitest `5.0.2`, Vite `8.3.1`. Windows `GMT Standard Time` observes BST at this checkpoint.

## Red-first and regression sequence

Added `local-ui.test.ts` and `menus.test.tsx` before behavioural implementation. The initial new-test run failed: the new helper module did not exist and all four initial menu/quit/edit/navigation fixtures failed against the v0.2.0 implementation. Its start timestamp was not captured; the tool output retains the failed result. The first combined implemented run, Vitest local start `18:21:02` / `17:21:02Z`, reported 45 PASS / 3 FAIL out of 48. Two historical assertions still selected removed v0.2.0 menu labels and one new assertion incorrectly referenced an unavailable matcher. The simultaneous typecheck exposed two unsupported `exact` query options and that matcher typo. Corrected the tests to exercise the deliberately replaced Select All/Delete, bundled Documentation and native-confirmed Settings removal journeys; privacy, financial, error-mapping and terminal-race assertions were retained.

An expanded run at `17:26:00Z` retained one failed Copy-selection fixture (53 PASS / 1 FAIL of 54). Strengthened selection setup and delayed-caret restoration so a later explicit selection/edit is not overridden by an earlier animation-frame completion. Subsequent 54-, 55- and 56-case suites passed. Independent review also identified real shifted-bracket keyboard values (`{`/`}`) and Unicode lowercase expansion shifting Find highlights; both were corrected with dedicated fixtures, not waived.

## Final scoped checks

Task PATH prefixed the bundled Node directory; commands ran in the v0.2.1 worktree using the existing offline-prepared lock/dependencies:

```powershell
pnpm.cmd typecheck
pnpm.cmd test
pnpm.cmd build
& 'C:\Program Files\Git\cmd\git.exe' -c 'safe.directory=C:/Users/Rene Nehru Alvarez/Documents/aster-desktop-premium/Aster-desktop-v0.2.1' diff --check -- src
```

The final execution's Vitest start was `2026-10-03T17:36:32Z` (displayed Windows BST `18:36:32`), duration 9.35 seconds. All four commands returned zero. Typecheck **PASS**; **56 tests in five files PASS**; client build **PASS**, 28 modules, Vite duration 158 ms; scoped whitespace check **PASS**. No key, account operation, native installer/app execution, paid request, network dependency installation or security setting change was performed by this renderer task. The client build tool's phrase “production” describes optimisation, not the project's engineering release class.

## Implemented portions and boundaries

T-064/066/067/068/069/070 renderer/source portions: closed optional window DTO/close-event bridge; caption-level Union Jack and named controls; grouped faithful scope-filtered menu mapping with real shortcuts; mouse/keyboard modal and busy guards; adjacent saved-chat navigation; bounded local title/content search; 80–150% zoom/reset and sidebar restoration; bundled offline help/status/about; deliberate clipboard operations with failure and stale-response denial; default-cancel quit review that preserves drafts on close failure. Copy of explicitly selected rendered text cannot accidentally target an old composer range. Menu Delete edits the selection, never a saved conversation or credential.

Drafts are process-memory only: at most 129 keys (128 conversations plus the initial unsaved slot), each cached draft at most 32,768 UTF-8 bytes, each undo/redo stack at most 16 snapshots and 65,536 UTF-8 bytes. An over-limit current draft is not silently truncated: navigation is blocked until reduced. Initial unsaved text is associated with a newly created local conversation before a fresh chat/navigation, keeping it reachable. Classification is cleared when changing conversations; display drafts never restore native admission/approval/financial authority. Find counts literal case-insensitive matches using original string indices and retains at most 512 highlight spans per message; this presentation bound is documented in offline help. No background clipboard reader, native clipboard plugin, model tool or provider probe exists.

Original renderer artwork remains unchanged. The central orb is composed by CSS ellipse masking/positioning with a separate inert HTML caption; no bitmap editing/redrawing. Brand rendered width/height/aspect remain unchanged; only its vertical gap is reduced. Send/Stop retain their 44 px / responsive 38 px circular sizes and original SVG geometry; the common sky-blue background is `#64c3fd`, a representative of the two slightly different supplied blue references. Disabled/focus state remains visible. The original native-caption appearance is reproduced by a bounded custom renderer row required for the requested flag placement; the native host independently owns its authority.

These source/synthetic PASS portions are **not** complete T-063..070 installed acceptance. Actual Windows caption/clipboard, DPI/Narrator, render/performance, owner acceptance, protected-history restart, native credential/terms/budget/disclosure and live usage/settlement remain **NOT RUN**. Legal, dependency inventories/audits, Rust and installer qualification are separately owned checks; no renderer fixture grants those PASS.

## Frozen renderer file SHA-256

| File | SHA-256 |
| --- | --- |
| `src/App.tsx` | `ba95e9174ddcba97b666adef1679ec73dd9390fde94f5e6eb265a4ccbbbc7827` |
| `src/styles.css` | `fb12f2dd6e1490301e15e49e79dfc649f99ee51a844a378400a6622127414614` |
| `src/local-ui.ts` | `7100e96fc420ccd1e0e55a60e18ec2e747ff0059e7d3e50de39d4105f39209ff` |
| `src/local-ui.test.ts` | `0d51b50ba2e652441b6aeb4a8bebce11ea70ed982127e8619f90e8df44d42881` |
| `src/menus.test.tsx` | `3f899d18ac343b15a4d1d3161c8ff268bc3fae2f366902d49bb3032de5ada8c3` |
| `src/App.test.tsx` | `f2105bf25bbf5321000409dbfedce5c2dc8e84494d61f343fc2c6cd0206675c6` |
| `src/types.ts` | `46acc9d29610917b282ed2d1d1a637b845a137866db3a6abd0e59c0be15216ea` |
| `src/bridge.ts` | `071d189fe576e7fe8a9ebb49057e41210f1619ae7bcdb520477c5464c749128e` |
| `src/bridge.test.ts` | `40ce91071fc3c086d36e5127fba941266b7cb4178871575f5b91cf9af8452d26` |
| `src/test-fixtures.ts` | `8d33140601baa6bcbe871f1214da53bc872a09fe8941661c73e5ee07c1713b8a` |

Later changes invalidate the dependent snapshot/checks; the root release qualification must bind the final full source and package separately.
