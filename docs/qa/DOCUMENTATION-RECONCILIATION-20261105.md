# Documentation reconciliation — 5 November 2026

## Before-change scope and evidence boundary

The owner requests cross-reviewing repository documentation and updating it to reflect the latest changes. This documentation-only task implements SPEC-0006 NFR-012 / AC-085 / T-095. Baseline: main `57339a687125a0b1fd8d6cda787649b5a111fa65`, tree `f29dcbaca827ac02574cee674ce5ca297f012063`, after merged [PR #68](https://github.com/comzulca/Aster-desktop/pull/68).

Inventory/link review covers 136 first-party Markdown files and 2,043 relative references, excluding vendored third-party and immutable bundled instruction resources. Baseline relative file targets have zero missing paths. Manual technical review focuses on the active README, v0.2.4 index/notes, provider/attachment contracts, setup, inventory and verification navigation, checked against current Rust/TypeScript sources and independently observed main CI.

The main [run 42081672227](https://github.com/comzulca/Aster-desktop/actions/runs/42081672227), attempt 1, is independently observed successful for this exact baseline. It qualifies the delivered unsigned engineering artifact within its recorded scope. It does not qualify later documentation commits, live provider/account access, owner-installed behaviour, production or a causal flicker remedy.

## Planned corrections

| Stale or ambiguous entry | Source/evidence to use | Correction |
| --- | --- | --- |
| No v0.2.4 main installer; four-job/old local failure presented as current | Exact main run, installer manifest and native receipt | Add a current status page and final engineering receipt with exact source/run/artifact hashes. Preserve original failures as dated history. |
| Whole-batch replacement and cancellation discard | `attachments.rs`, `lib.rs`, `App.tsx`, SPEC-0007 and new passing regressions | Document additive snapshot-bound selection, atomic rejection/cancellation retention and unchanged aggregate/expiry limits. |
| 2 November review deadline and 2 October current terms | `contributor_policy.rs`, `contributor.rs`, `budget.rs`, dated primary-source review | Document 9 October terms, 12 October geography, 5 November review and 6 December 00:00 UTC local deadline; preserve old record/quote semantics. |
| Protocol 3, text-only/no attachments or a universal 512-KiB wire cap | `policy.rs`, `lib.rs`, `provider.rs` | Document protocol/policy 4, native PDF/normalised images, 512-KiB text limits and separate 24-MiB complete media-request cap. |
| Test-only foundations described as universally NOT RUN | Exact main Windows test log | Record executed pure tests separately from unavailable runtime mutations/full-window migration and unobserved owner/live acceptance. |
| Unqualified setup/history presented beside current operations | Active UI/host routes and dated setup decisions | Add a current operating guide; clearly scope historical registration/permission observations and retired providers. |

## Preservation and integration rules

Keep the full original README disclaimer/preview prefix. When replacing a current entry page, copy its complete baseline bytes to a clearly named archive in the same directory so its relative links remain meaningful; record each SHA-256 below after execution. Do not edit original dated QA receipts, licences, source, resources, dependency inputs or frozen JSON ledgers. Prepending current navigation to procedural/legacy pages must preserve their original bodies verbatim.

Validate local paths and anchors, archive identity, disclaimer/preview identity, source facts and the documentation-only diff. Existing five-job hosted gates and actual fifteen-codec/six-resource/CPU probes remain required before normal integration; do not weaken or skip them. No new installer delivery or production qualification is requested by this task. New documentation-source CI is NOT RUN at this before-change checkpoint and must be observed separately.

## Implemented documentation corrections

The current README, version index, engineering notes, provider and transport pages now describe software source `57339a6` and link to a single current status matrix, operating guide, build guide and exact engineering receipt. The README retains the supplied disclaimer/preview prefix unchanged. New attachment documentation covers additive selection, cancellation/rejection retention, aggregate bounds and temporary expiry; it separates parser, text-context and complete media wire limits.

Retired-provider pages, historical attachment/setup records and the inventory/verification entry points receive dated current-navigation prefaces without changing their original bodies. The dedicated GitHub App's reported October installation scope is not presented as a freshly verified external state. All original dated QA receipts, spec/threat/ADR bodies, frozen JSON ledgers and bundled resources remain untouched.

The current status also corrects the test-only foundation distinction: hosted logs for exact main show 34 GitHub mutation, 27 full-window capacity, 27 transcript-layout and eight native-operation lease cases passing, while their dependent runtime capabilities remain unavailable. The two credential cases ignored in the default 412-PASS suite passed separately on disposable targets; this is not owner/native GUI acceptance.

## Byte-identical archive identities

Each archive remains in the original document's directory to preserve relative-link meaning. The complete baseline bytes were copied before replacing current entry pages.

| Original document | Preserved archive | SHA-256 |
| --- | --- | --- |
| `README.md` | [README-ARCHIVE-57339a6.md](../../README-ARCHIVE-57339a6.md) | `a0d22f9f6f0a1088501815fc23e8d8679895cb5016fb967852778d5c4c5a83ec` |
| `docs/V0.2.4-DOCUMENTATION-INDEX.md` | [V0.2.4-DOCUMENTATION-INDEX-history-57339a6.md](../V0.2.4-DOCUMENTATION-INDEX-history-57339a6.md) | `95fc2ddaa5aa8bba8db839988e4aed77370fe785a62c2a784a7581b67b826a5f` |
| `docs/RELEASE-NOTES-v0.2.4.md` | [RELEASE-NOTES-v0.2.4-history-57339a6.md](../RELEASE-NOTES-v0.2.4-history-57339a6.md) | `4f292ff36bb66b0ec0602684d5809eacd0d00625c47d0a7760caefd122e0ac6f` |
| `docs/providers/transport.md` | [transport-history-57339a6.md](../providers/transport-history-57339a6.md) | `4650afe523dbae5fad71d28e742fe8a4fb4a019ca6bf126c9eca0881a0c0240d` |
| `docs/providers/meta-muse-spark-1.3-contributor.md` | [meta-muse-spark-1.3-contributor-history-57339a6.md](../providers/meta-muse-spark-1.3-contributor-history-57339a6.md) | `9dac652beafcfb31910e58fadb9d348d14f260c6ba363543fc9a81e16a322163` |

## Verification boundary for this documentation change

Local path/anchor, archive/preface identity, README disclaimer/preview identity, Markdown structure, current-source fact and documentation-only diff checks are recorded after execution below. Hosted CI for the documentation commit must be observed separately; the delivered software's run is retained as evidence only for source `57339a6`. This change does not deliver, retag, sign or requalify a different binary.

## Observed local checks before publication

PASS: 147 first-party Markdown files / 2,199 relative references checked, zero missing local paths or Markdown anchors; five archive byte/SHA-256 identities; six verbatim original bodies beneath navigation prefaces; exact README disclaimer/preview prefix and supplied image hash; current canonical code-fence structure; source-pinned policy/protocol/context/wire/date facts; installer manifest/source/run/hash binding; no personal download/session URL in current pages; Markdown-only tracked diff and `git diff --check`. No application test, live call or installer execution is claimed for this documentation-only local validation.

The runtime, manifests, dependencies/locks, workflows, root licences/NOTICE/disclaimer file, artwork, bundled inert resources and frozen JSON ledgers are unchanged. Hosted checks for the forthcoming documentation commit remain pending at this local checkpoint; their observed outcome belongs to that exact PR/run, not a retroactive edit of the software receipt.
