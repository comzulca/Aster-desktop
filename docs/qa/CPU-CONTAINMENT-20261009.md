# CPU containment repair — 9 October 2026

Status: before-code contract accepted for implementation after independent review on 2026-10-09; proposed separate PR; changed-source execution NOT RUN.

## Phase 0 — exact baseline and authority

Baseline main is `4e46b1bf126cd9937d68d1058ccfcc0f1c162339`, tree `ee96ce466deecabe9b330d64df5ba4ea67a9ea73`, Aster v0.2.3. PR #30 merged normally at 2026-10-09T15:19:39Z after all five final PR-source jobs and its actual 15-codec/6-resource diagnostics passed. Its distinct actual-main run [37950896361](https://github.com/comzulca/Aster-desktop/actions/runs/37950896361) is FAIL on the original package job `113901326653` and single fresh package retry `113909777038`. Four actual-main source jobs passed and were carried forward, not reexecuted. Both package receipts report 15 codec PASS, five resource PASS and CPU FAIL: user CPU 147656250/148437500 100-ns ticks; native limit terminations zero; exit1 around wall15029/15025 ms. Retain both failures. There is no qualified actual-main installer from that run.

The owner expressly authorises a separate CPU correction PR, preserving existing limits and without relaxing mandatory gates. Preparing this PR is not authority to merge it, release, sign, run the owner's app/installer, touch private credentials/history or make live provider calls. The owner separately forbids GLib remediation: follow [the upstream-only deferral](GLIB-UPSTREAM-DEFERRAL-20261009.md), keep alert OPEN and lock/manifest/vendor identity unchanged.

The prior native-only proof is insufficient for these failures: Windows documents periodic Job CPU checks, without a wall-clock enforcement deadline. That timing interaction is a plausible explanation, not a proved root cause. Correct100-ns conversion is retained.

## Phase 1 — fixed host-owned strengthening

Existing SEC-028 / NFR-009 -> TM-029 -> AC-072/AC-073 -> T-082/T-083 govern this correction. No new capability or reused/retired identifier is introduced.

Retain both native Job user CPU limits exactly10 seconds (100000000 ticks), wall15 seconds, memory256MiB, active-process-one, kill-on-close/no breakaway, restricted Low-IL token, empty explicit environment, fixed self executable/arguments, exactly three inherited stdio handles, suspended-before-Job assignment, immutable byte protocol, content validation, bounded I/O cleanup and poisoning. Preserve fixed single-thread CPU fixture workload and all15 codec/all6 resource cases. No extra monitor thread, child, completion port, caller input, fallback, environment or parser change.

Add trusted-parent cumulative user-mode CPU observation using `GetProcessTimes` on the exact retained process HANDLE. Sum is supplied by Windows across child threads; do not subtract startup CPU or silently change budget to include kernel time. Checked FILETIME conversion must reject overflow, negative/decreasing samples and API failure. Probe availability before ResumeThread, sample every existing control-loop cycle and again before admitting exit0/output. Never manufacture a successful zero/default sample. Query failure, exceeded limit or uncertain cleanup denies content.

Cancellation and the unchanged15-second wall deadline take precedence. Recheck wall after sampling immediately before CPU termination. If wall expired, cancellation occurred or child was already signalled, do not manufacture host CPU-kill proof. Once valid cumulative user CPU reaches100000000 before wall15, terminate only that exact Job with a fixed nonzero CPU-cause exit. Retain kill/handle owners until physical process signal and bounded owned-I/O cleanup complete. Avoid a second generic kill overwriting the CPU exit/cause; preserve fallback denial/poison semantics if primary termination fails. The outcome stays terminal and cannot become successful content.

Pin the CPU-only termination code to `0xA57E0001` (`2776498177`), distinct from generic teardown1, ordinary worker3 and PDF65..81. A closed sticky terminal-cause state must prevent reclassification or repeated credited CPU dispatch. Preserve RAII fallback/poisoning until physical exit is confirmed; suppress later generic JobKill/owned-I/O cancellation termination only when it would overwrite an already confirmed CPU termination. I/O cleanup failure still denies proof.

Sampling/dispatch/OS scheduling are not hard-real-time guarantees. Keep the20-ms existing wait cadence, but never promise exact10.000-second enforcement or20-ms worst-case overshoot. Record trusted trigger CPU/wall observations separately from final accounting/cleanup elapsed time. No lower fixture-only budget or larger production budget.

## Phase 2 — causal acceptance, not counter relabelling

The six mandatory resource gates still require real observed containment; a wall/cancellation/cleanup kill or status string alone is never CPU PASS. Keep the original native CPU branch exactly: exact marker/no extra output, admitted boundary/valid accounting, natural nonzero exit, Worker error, cumulative job user CPU >=100000000 and actual native `TotalTerminatedProcesses>0`.

The new enforcement branch may pass only with ALL of: exact CPU marker/no extra output; valid admitted boundary; monotonic successful parent sampling of the exact retained child; live child immediately before dispatch; trigger user CPU >=100000000; trigger wall strictly below15000ms with cancellation/wall precedence rechecked; successful termination of the exact Job using a fixed nonzero CPU exit; physically signalled child with that exact exit; completed cleanup; denied content and no natural-exit/wall/cancel/close/fallback uncertainty. Require final valid Job accounting >= trigger CPU and CPU threshold. Native accounting count remains its real value, often zero for explicit termination: never increment or relabel it.

Use closed parent-authored evidence distinguishing `native-job-cpu-limit`, `parent-user-cpu-watchdog` and unproved/none. The native-only mechanism predicate is honestly amended to causally verified CPU containment, not claimed literally unchanged; the resource budget and functional/security gate strength are preserved and reinforced. Historical failed receipts contain no such host proof and remain FAIL. `qualificationComplete` remains false; this is not full product, manual, hard-real-time or production qualification.

Serialise only bounded additive CPU-proof fields: a closed mechanism label, trusted trigger user ticks and wall milliseconds, closed booleans for sampling validity, live-child observation, priority recheck, exact-Job dispatch, physical expected exit, completed cleanup and terminal denial. Keep actual native counters unchanged. Independently mirror the complete pure receipt predicate in CI, reject malformed/missing/extra proof fields and exercise one-field-at-a-time negative controls. This is supplementary evidence, not a child assertion or a replacement for existing receipt guards.

The implemented additive `cpuProof` has exactly sixteen keys: `mechanism`, `triggerUser100ns`, `triggerWallMillis`, `samplingValid`, `liveChildObserved`, `priorityRechecked`, `exactJobDispatched`, `physicalExpectedExit`, `cleanupComplete`, `terminalDenied`, `exactMarker`, `statsValid`, `naturalExit`, `jobClosed`, `workerError`, `fallbackUsed`. Non-CPU cases use null. Common proof checks strengthen both mechanisms with valid sampling, completed cleanup, terminal Worker denial and no fallback; all original native marker/boundary/accounting/natural/nonzero/counter obligations remain. Explicit parent dispatch cannot impersonate the native mechanism.

Threats covered under TM-029: stale/reused/forged identity; unauthenticated child claimed CPU values; invalid/nonmonotonic accounting; successful over-budget output; wall/cancel misclassification; repeated kill/cause overwrite; teardown uncertainty; invented native counters; status-only/changed-source promotion. The host exclusively owns handles, clock, measurements and decisions. Do not serialise native errors, paths, private source/output, environment, tokens or credentials.

## Mandatory regressions and fresh evidence

- Checked FILETIME conversion across32-bit boundaries and signed bounds; monotonic cumulative state.
- Below threshold continues; exact threshold denies; cancel/wall take precedence; failed/decreasing/invalid query denies even exit0 success.
- Exact CPU-trigger proof plus one-at-a-time negative mutations of every required causal field.
- Final CPU/exit1 alone, wrong/missing marker, readiness/accounting failure, extra output, zero native count without host proof, wall/cancel/close, wrong expected exit, failed kill, unobserved physical exit and failed cleanup all reject CPU PASS.
- Original native branch retains its original requirements; neither branch can become a generic nonzero-exit acceptance.
- Sticky denied state and at-most-one credited CPU dispatch; no over-budget successful codec output admitted.
- Keep other five resource predicates, complete parallel Rust suite, original PDF diagnostics and actual release-executable15+6 unchanged in strength.
- If CI receipt checks change, additions only: retain native exit, schema1, allPassed, exact counts/allPASS, all existing source/locks/payload/licence/hash/NotSigned guards. Require closed CPU proof independently; no skipped job/test, continue-on-error, weakened advisory severity or artifact substitution.

Fresh final-source source/frontend/Linux-source/complete Windows/audit/licence/SBOM/package jobs are required. Record commands, source/ref/run/attempt, host, full logs and each actual probe. Package checks and source gates start NOT RUN. A later actual-main build remains distinct; neither PR-source success nor library success transfers to it. All42 compound/manual/owner Windows11 visual/startup/live/signing/production obligations remain NOT RUN, with historical owner flicker and Code Integrity failures preserved.

## Implementation review and local-only checks

At the 2026-10-09 17:35 UTC preparation cutoff, root and independent reviews of the changed worker/fixture and CI/verifier found no blocking issue. Five new Rust test definitions cover checked timing, monotonic/sticky decisions, each causal proof field, original native requirements and the exact serialised schema; their native execution remains NOT RUN. Root independently observed `rustfmt --edition 2021 --check` exit0 and Python3.12.14 `-I -B scripts/ci/verify_cpu_receipt.py --self-test` exit0 with184 negative controls. The independent reviewer additionally observed1242 mutation rejections and both valid proof mechanisms. These are static/pure checks, not real CPU containment or a qualified installer. Removing the two additive CI blocks restores the complete original workflow byte-for-byte; no previous gate is waived. Fresh hosted results belong to the separate CPU PR's live receipt, including its exact tested source and run/attempt; no merge or actual-main success is implied here.

## Primary API references

- [GetProcessTimes cumulative user CPU and handle requirements](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes).
- [Periodic native Job CPU limits](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information).
- [Native termination accounting](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_accounting_information).
- [Exact Job termination](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-terminatejobobject).
