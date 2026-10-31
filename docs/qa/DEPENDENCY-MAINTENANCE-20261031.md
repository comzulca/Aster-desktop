# Dependency maintenance — 31 October 2026

## Scope and baseline (before dependency changes)

Remote baseline: `comzulca/Aster-desktop` main `d8b9422eab83526a67bd16a23fbb0d029870d709`, tree `a0a7329bf88259b37c876d41cb14a912e3e03cd8`. The owner requests reviewing Gmail's GitHub/Dependabot notifications, correcting necessary repository problems and marking attended messages read after the work. This authorises scoped maintenance publication and CI; normal integration is conditional on the unchanged mandatory exact-source gates. No application release, installed-owner acceptance or live-provider use is claimed.

The two unread 10 October workflow notifications are historical failures of PR #41; its latest exact head `b459be9830b26c3ba3a7c4cc0d0c9f16d0469f9c` has run `38073897796`, all five jobs successful. PR #41 remains a draft partial v0.2.4 change and is not part of this stable maintenance integration.

## Existing requirements and acceptance

This maintenance implements existing SPEC-0001 NFR-003 / SEC-009 / AC-019: exact dependency inputs, preserved advisory gates, source-bound Windows tests, licence inventory/SBOM and honest release claims. No new IPC, credential, file, provider, UI or network authority is added. Supply-chain and TLS risks are bounded by using the published Dependabot diffs and official rustls crate-index metadata, retaining checksum verification and exact-source CI.

Acceptance requires: structural Tauri alignment and unchanged pinned WinRT ABI pair; frozen frontend installation/typecheck/full tests/build/advisory audit; codec and vendor fixtures; complete Windows check/Clippy/tests/advisory audit; generated notices/SBOM and the existing unsigned engineering package with exactly fifteen codec and six resource checks plus independent causal CPU receipt validation. All changed-source gates are NOT RUN at this pre-change checkpoint. Do not weaken checks, suppress advisories or replace recorded failures with a speculative PASS.

## Observed issues and intended correction

- Vite 8.3.2 fails the moderate advisory gate on five published advisories: GHSA-587h-7wmj-h5pm, GHSA-698v-4r3x-3vcv, GHSA-r825-w8mm-5pxw, GHSA-rq7h-c2jc-7f22 and GHSA-vfpm-58rq-9qcg. Use the reviewed Dependabot Vite 8.3.4 resolution, including every Vitest peer edge.
- rustls 0.23.45 fails cargo-audit on RUSTSEC-2026-0346 through RUSTSEC-2026-0349. Move only the locked rustls package to non-yanked 0.23.46 using the official crates.io Git index checksum. Version 0.23.46 has the same declared normal/build dependencies, features and MSRV as 0.23.45; frozen Cargo resolution/build remains the acceptance check.
- Consolidate the compatible existing Dependabot updates #42, #43, #45, #46, #47, #48, #50, #52, #53, #54 and #55 in one tested maintenance candidate. Preserve action commit-SHA pins, read-only workflow permissions and disabled credential persistence.
- Do not apply #44's isolated windows-future 0.100.0 upgrade. Its actual run `38712723895` fails compilation because windows 0.62.2 and windows-future 0.100.0 use incompatible windows-core types. Keep windows-future 0.3.2 with windows 0.62.2; document the deferral rather than bypassing compilation. A coordinated WinRT migration is separate work.
- GLib 0.18.5 / RUSTSEC-2024-0429 remains OPEN and UNRESOLVED under the existing owner decision. Upstream Tauri #16170 and Wry #1843 are still draft/unmerged at this review; stable Tauri 2.12.4 still declares GTK 0.18. No local backport, ignore, dismissal or architecture migration is included. proc-macro-error 1.0.4's unmaintained warning also remains visible.

## Provenance and remaining obligations

Every dependency change is attributable to the existing PR diff or official registry metadata. The local snapshot is reconstructed through the authorised GitHub connector, not the owner's Windows checkout. No local Rust toolchain or matching pnpm dependencies are available here; full native execution must be observed on hosted CI. Earlier evidence and the frozen current ledger remain historical; this document does not rewrite them.

Owner Windows 11 installation, flicker assessment, 42 compound protocols, live credentials/provider requests, signing and production qualification remain separate. Gmail message IDs and email bodies are not committed into the repository.

Primary references: [Vite advisories](https://github.com/vitejs/vite/security/advisories), [rustls 0.23.46](https://github.com/rustls/rustls/releases/tag/v%2F0.23.46), [official rustls index](https://github.com/rust-lang/crates.io-index/blob/master/ru/st/rustls), and [retained GLib decision](GLIB-UPSTREAM-DEFERRAL-20261009.md).

## 31 October expanded advisory discovery (before follow-up changes)

The first candidate head `ef33270fe277a2ec54b388b910110d7cd6b06827`, run `41243841045` attempt 1, passed source-policy, codec fixtures, frozen frontend installation, typecheck, all 100 frontend tests across eight files, and the renderer build. Its unchanged advisory gate then FAILED on eight undici advisories published after the original dependency runs. Windows and package jobs were SKIPPED, not passing.

Before the follow-up lock changes, the current official RustSec tree was also inspected against locked crate names. Extend the existing NFR-003 / SEC-009 / AC-019 maintenance acceptance to these compatible security refreshes:

| Locked input | Intended version | Reason |
| --- | --- | --- |
| undici 8.11.2 | 8.12.2 | Eight GHSA advisories: xjj6-f33q-929c, 4w7w-5xrc-9h26, 2m5v-pch6-74hx, qrxm-fqhp-jwvm, 4qv8-2p9j-2685, f72v-6w45-q472, 9fm3-97px-rjf6, 55wc-3pg8-jfw6. |
| rustls-webpki 0.103.15 | 0.103.16 | RUSTSEC-2026-0382, rejection of distrusted emSign intermediate keys. |
| webpki-root-certs 1.0.9 | 1.0.10 | RUSTSEC-2026-0383, remove the affected root. This and 0382 share CVE-2026-111943; do not count two distinct CVEs. |
| tokio 1.53.2 | 1.53.3 | RUSTSEC-2026-0386, Unix AsyncFd I/O safety warning. Avoid the separate yanked 1.54.0 interval regression. |

The three Rust updates have identical declared normal/build dependencies to their respective old versions in the official crates.io Git index. webpki-root-certs raises MSRV from 1.70 to 1.71, within the pinned Rust 1.97.0 toolchain. All selected crate versions are non-yanked. Verify fresh locked resolution and full CI; metadata compatibility alone is not runtime acceptance.

jsdom 30.1.2's official manifest accepts undici `^8.11.2`, and undici 8.12.2 has no runtime dependencies and retains Node >=22.19.0. The npm metadata endpoint was unavailable in this session. The candidate SRI was independently cross-checked in the published Prometheus and OpenAI Node SDK lockfiles; a fresh pnpm frozen install must validate the actual registry download against it. No override or fabricated checksum is permitted. Primary source versions are [jsdom v30.1.2](https://github.com/jsdom/jsdom/blob/v30.1.2/package.json), [undici v8.12.2](https://github.com/nodejs/undici/blob/v8.12.2/package.json), and the RustSec advisories listed above.

The earlier FAIL remains preserved. Follow-up final-source/native/package checks remain NOT RUN at this amendment. GLib and the existing unmaintained proc-macro-error warning retain their prior unresolved status; no ignore or audit suppression is added.
