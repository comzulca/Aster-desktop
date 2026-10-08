# LOCAL-0001 — development workspace checks

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
