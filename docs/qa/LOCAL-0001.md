<!-- OCT9 CPU AND GLIB OWNER DECISION BEGIN -->
> Owner decision and current applicability — 2026-10-09 17:14:45 UTC: PR #30 is normally merged; its distinct actual-main package run remains FAIL after two CPU-probe failures, not a qualified main installer. The [separate CPU repair contract](CPU-CONTAINMENT-20261009.md) preserves the 10-second user-CPU and 15-second wall budgets and all mandatory gates; changed-source execution remains NOT RUN until fresh evidence. [GLib 0.18.5 remains OPEN and UNRESOLVED](GLIB-UPSTREAM-DEFERRAL-20261009.md), deferred to a complete compatible stable upstream chain; no backport, dependency remediation, dismissal or ignore is authorised. GTK4/Tauri 3 is a future owner-preferred architecture review, not an approved migration or promised fix. Earlier bodies below are retained dated source-bound history; no historical PASS transfers to changed source or owner/manual/live/signing/production acceptance.
<!-- OCT9 CPU AND GLIB OWNER DECISION END -->

<!-- OCT9 FINAL FOLLOW-UP BEGIN -->
> Operational follow-up — 2026-10-09 14:18:44 UTC: the [completed-source reconciliation](OCT9-RECONCILIATION-20261009.md#completed-p11-and-final-documentation-follow-up) records the newer tested-source facts and owner decision. For the observed status of later documentation-only CI, normal integration, actual main-push and Gmail handling, consult the [PR #30 closing receipt](https://github.com/comzulca/Aster-desktop/pull/30#issuecomment-6082465272), [live PR #30](https://github.com/comzulca/Aster-desktop/pull/30) and [Actions](https://github.com/comzulca/Aster-desktop/actions). Earlier cutoffs below remain dated history; green results never transfer to a changed source. The glib alert remains OPEN for separate work; manual/live/signing/production acceptance is not inferred.
<!-- OCT9 FINAL FOLLOW-UP END -->

<!-- OCT9 CURRENT APPLICABILITY BEGIN -->
> Current-source applicability — 9 October 2026, 13:02:58 UTC: the [dated reconciliation](OCT9-RECONCILIATION-20261009.md) owns the newer exact-source outcomes and their limits. Earlier “current”, “pending” and NOT RUN statements below remain source-bound history where superseded there; original bodies and before-code approvals are preserved. P10 hosted success does not qualify a changed cumulative source. Final-source CI/package gates require re-execution; owner flicker FAIL, all 42 compound cases NOT RUN and independent manual/live/signing/production obligations remain.
<!-- OCT9 CURRENT APPLICABILITY END -->

# LOCAL-0001 — development workspace checks

<!-- P10 CURRENT APPLICABILITY BEGIN -->
> Current-source applicability — 8 October 2026, 21:51:41 UTC: the [dated reconciliation](OCT8-RECONCILIATION-20261008.md) owns this checkpoint. PR #38 and #31–#35 are normally merged after their four exact-source gates; #36/#37 remain pending. P9's complete Windows suite is FAIL; its isolated synthetic diagnostic PASS does not override it. The [P10 scoped-factory contract](PDF-SCOPED-FACTORIES-20261008.md) received before-code design approval at 21:47:59 UTC; source/documentation preparation is in progress, not a published or native/package PASS. Earlier “current” headings and statuses below are retained dated history, not the latest source result. The dependency auxiliary case is a private child lease-denial test, not Credential Manager proof. Owner v0.2.2 flicker/Code Integrity FAIL and all 42 compound cases NOT RUN remain; manual/live/signing/production acceptance is not inferred.
<!-- P10 CURRENT APPLICABILITY END -->

> Current-source applicability — 8 October 2026, P9 preparation: the [dated reconciliation](OCT8-RECONCILIATION-20261008.md) supersedes current-status claims below. PR #38 and #31–#34 are normally merged after four source-bound gates; PR #35–#37 and PR #30 are not complete. P8's native Windows access violation remains FAIL, its installer SKIPPED. The [P9 contract](PDF-CRASH-DIAGNOSTIC-20261008.md) admits only test-only diagnosis after the unchanged complete suite fails; 112 counted pure checks and static review are not native qualification or an installer PASS. The dependency auxiliary test is an isolation lease test, not native credential execution. Original dated results, owner v0.2.2 flicker/Code Integrity FAIL, credential custody and manual/live/production obligations remain unchanged.

**Retained dated v0.2.3 applicability — 4 October 2026, Edinburgh / Europe/London:** dated evidence below retains its original source, version and result; it does not qualify changed v0.2.3 bytes. Unchanged constraints remain subject to the [current documentation index](../V0.2.3-DOCUMENTATION-INDEX.md) and [SPEC-0005](../specs/SPEC-0005.md). The owner authorises a v0.2.3 PR and GitHub checks only, not merge, release or signing. Final-source, package, native and live qualification require independent evidence in [LOCAL-V023-0001](../qa/LOCAL-V023-0001.md). No retired provider, historical grant or old PASS is reactivated.

**Current-status navigation — 8 October 2026:** see [the reconciliation record](../qa/OCT8-RECONCILIATION-20261008.md) for actual PR/dependency/source/gate outcomes and the later owner-authorised conditional integration. This dated paragraph and the original evidence below are preserved history, not current authority, a new execution receipt or a reactivation of retired providers. Manual/native/live/production results do not transfer between sources.


**v0.2.2 applicability, 3 October 2026:** retained requirements/contracts apply only where unchanged; retired providers, disabled capabilities and older dated results are not current authority or new-source PASS. [SPEC-0004](../specs/SPEC-0004.md), [ADR-0017](../adr/ADR-0017.md) and [LOCAL-V022-0001](../qa/LOCAL-V022-0001.md) govern active zoom/startup changes and fresh evidence. Earlier evidence remains source-bound history; the later owner-reported v0.2.1 startup flash is recorded separately. The entire original v0.2.1 checkout/installer is unchanged.

Date: 2026-09-24 UTC. Environment: isolated Linux development workspace; Node 24.19.0, pnpm 11.19.0, Python 3.12.14. Status: **engineering checks only**, source revision not yet published when recorded. These checks are not Windows or exact GitHub commit acceptance evidence. Rerun on the committed SHA for release gate decisions.

| Exact command | Observed result | Scope |
| --- | --- | --- |
| `pnpm install --frozen-lockfile --offline` | PASS, lockfile unchanged | Renderer dependencies from cached verified packages |
| `pnpm typecheck` | PASS | TypeScript renderer |
| `pnpm test` | PASS, 3 files / 9 tests | Renderer bridge, state, inert stream rendering |
| `pnpm build` | PASS, Vite 7.3.6 produced `dist/` | Renderer build; not a Tauri executable |
| `pnpm audit --audit-level moderate` | PASS, no known advisories reported on this date | Registry advisory snapshot; does not cover Rust or future advisories |
| `pnpm licenses list --json` and `pnpm sbom --sbom-format cyclonedx` | PASS, parseable JSON; SBOM lists 167 components | Dependency inventory only, legal review pending |
| `python3 scripts/ci/scan_secrets.py --self-test && python3 scripts/ci/scan_secrets.py` | PASS, heuristic check over Git-visible text | Not an exhaustive secret or binary scan |
| SHA-256 comparison of `public/assets/Aster_UI2.jpg` to uploaded original | PASS, both `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887` | Original reference bytes only |
| Python YAML/JSON/TOML parse for workflows, Tauri config and toolchain | PASS | Syntax only, no GitHub Actions or Rust build |

The `aster-builder` structural checker through phase 2 reports three outstanding gates (0, 1, 2) as **NOT RUN**; it reports no other structural issue. Cargo, Rustfmt, Windows host tests, native Credential Manager operations, NTFS grants, live NVIDIA model canaries, installer build/signature and actual laptop checkout remain **NOT RUN**. The dedicated hosted workflows and exact source revision must close those gates before any release claim.
