# v0.2.4 GitHub recovery/status proposal — 10 October 2026

Status: **PROPOSED, NOT APPROVED, BEFORE CODE**. This document proposes only a local recovery/status vertical slice under SEC-033/034/036, AC-081/082/086 and T-091/092/096 in [SPEC-0006](../specs/SPEC-0006.md), TM-036/037/038 in [THREAT-MODEL-0006](../security/THREAT-MODEL-0006.md), and [ADR-0021](../adr/ADR-0021.md). Root must read the complete proposal and approve its exact source scope before any Rust, renderer, store-domain or IPC change. Drafting this document does not activate storage, preparation or dispatch, qualify a run/artifact, or authorise execution/publication.

Read the [mutation contract](V024-GITHUB-MUTATION-CONTRACT-20261010.md), [auth/read host contract](V024-GITHUB-HOST-CONTRACT-20261010.md) and [controlled integration contract](V024-INTEGRATION-CONTRACT-20261010.md) first. The frozen published head inspected for this proposal is `5008e9b70ea5f4317cf64023002578b8c4c512dd`; the new protected-backend working-tree fixtures are unpublished source, not that head's executed result.

## 1. First vertical slice and explicit non-goals

The smallest proposed runtime slice opens one fixed protected local metadata inventory, strictly replays it, and returns a bounded content-free status. It cannot append a dispatch/terminal record, prepare/review/apply a plan, send a request, clear/repair/reconcile uncertainty, change accounts/permissions, read credentials, or launch a native review, browser, process or installer. `preparationAvailable` and `mutationsAvailable` remain literal `false` on every receipt. Existing auth/read routes and their `mutationsAvailable=false` are unchanged.

Opening the store is a distinct runtime integration step requiring approval and native evidence; a `cfg(test)` backend adapter is not that step. The current `github_mutation` parent is test-only. Do not enable that entire module in production, suppress dead-code warnings, or expose its plan/dispatch methods to obtain recovery. Personalization, history, catalogue, budgets and GitHub token custody remain separate domains. None supplies recovery input or receives ledger contents.

## 2. Fixed domain, owner and bounded inventory

Proposed production domain: `Domain::GithubMutations`, fixed directory name `github-mutations-v1`, with the already reviewed limits of 16 KiB per **plaintext payload**, 4 MiB aggregate plaintext and 512 records. Host-generated names remain exactly 32 lowercase hexadecimal characters, derived from canonical record sequence. No renderer-selected base/domain/name/path is accepted.

The proposed host owns a separate recovery instance using only its existing application-local-data path service and the existing `PrivateStore` protections. Retain the current-user DPAPI envelope, domain entropy, protected DACL, ancestor/directory identity pins, no-reparse/single-link checks, immutable records and exclusive writer lease. No weaker memory fallback, repair, adoption of corrupt/foreign objects, or file copy into an owner store is permitted. Existing seven production domains and their bounds stay unchanged.

Main must approve where this instance is opened before code. The proposed first implementation owns one `Result<RecoveryInventory, StoreError>` for the host lifetime; `RecoveryInventory` retains its `PrivateStore` and writer lease together with the immutable reduced metadata, not an exposed backend or replayable record buffer. It does not repeatedly reopen or silently retry unavailable storage. A fresh `PrivateStore::open` may create only its ordinary fixed container/domain and empty writer lease as existing infrastructure, not a ledger record. That infrastructure effect must be stated in the native implementation review rather than calling opening side-effect-free. Opening an existing inventory never deletes, overwrites or repairs it.

These quotas are payload bounds, not an asserted 4-MiB ciphertext/disk limit, a latency guarantee or secure-erasure guarantee. The existing protected envelope/ciphertext bounds remain in force; any tighter domain ciphertext/elapsed-time contract needs a separately reviewed derivation and fixtures, not a guessed DPAPI overhead. DPAPI/DACLs do not defend against same-user compromise or undetectable rollback. No pre-restart cryptographic anchor is invented.

## 3. One private canonical parser shared with the test core

Before production linking, factor only the existing record-only types, exact canonical parser, schema validation and metadata replay reducer into one private leaf, proposed as `github_mutation_record.rs`. The existing test-only ledger and the proposed recovery reader must call that same implementation. Do not duplicate a permissive recovery parser or deserialize status/caller data into ledger authority. Metadata-only binding/resource/operation/effect validators may be factored with it only after root verifies their complete source correspondence and unchanged canonical representation.

The leaf must not import or activate `PlanBook`, plans/permits, native approvals, request/payload serializers, snapshots, transport/dispatch, credential custody or account restoration. A recovery inventory consists only of the validated record count and internal unresolved metadata needed to answer the conservative selected-repository predicate. Record objects and that internal index are never IPC/export/log/provider values.

Preserve the current strict replay rules: bounded records and aggregate; exact schema/version; unique names/sequences; exact sequence-derived names; contiguous increasing sequence; canonical byte equality on reserialization; valid operation/plan IDs, metadata and effect shape; one intent per plan; exact operation-ID/intent relationship; a terminal only after its matching intent; no duplicate terminal; operation-compatible verified IDs. Unknown fields/versions, duplicate JSON keys, whitespace/noncanonical encodings, foreign/corrupt ciphertext, gaps, contradictory resource/effect data and incomplete inventory deny the entire read. Do not return a valid prefix or reduce required evidence.

The root review must explicitly prove that factoring retains canonical field order, exact bytes/hashes and every original hostile assertion. Reuse existing source-bound fixtures; an old PASS does not qualify the factored source. Backend-decrypted UTF-8 bytes alone are not ledger evidence. Failed publication poisons a live test ledger as before; the recovery reader cannot clear that poison or claim whether an unacknowledged write was absent. Only independently read, complete canonical records can describe historical state.

## 4. Proposed fixed status route and receipt

Only the authenticated fixed main window may request a future `get_github_mutation_status` command. It has no input DTO and supplies no path, account/repository identity, epoch, approval flag, record, or repair option. There is no model/tool/package route. No renderer method is added by this proposal.

Proposed closed output schema, all fields present and nullable fields explicit:

```typescript
type GitHubMutationRecoveryStatus = {
  schemaVersion: 1;
  state: 'ready' | 'attention_required' | 'unavailable';
  recordCount: number | null;
  unresolvedCount: number | null;
  selectedRepositoryBlocked: boolean | null;
  preparationAvailable: false;
  mutationsAvailable: false;
};
```

Counts are integers from 0 through 512, with `unresolvedCount <= recordCount`. `ready` means only a complete valid local inventory with zero unresolved intents; it does not mean authenticated GitHub, known permissions, UTC attribution, spare write capacity or executable preparation. `attention_required` means a complete valid inventory has at least one unmatched intent or `attention_required` terminal. Other valid terminal outcomes are historical observations, not permits. An empty valid fresh inventory has both counts zero.

`unavailable` means no complete trustworthy inventory is available. Both counts and `selectedRepositoryBlocked` are then `null`, never manufactured zero/false. Store errors remain closed; do not emit paths, native errors, ciphertext, account/install/repository IDs, plan/hash/sequence values, bodies, tokens, timestamps, profile content or record arrays. Missing/unknown receipt is not evidence of an empty store, absent account, no uncertainty or permission.

For valid inventory, `selectedRepositoryBlocked` is `null` unless the currently enabled connected host has an independently authenticated current selection. It is then true if any unresolved entry has that exact stable repository ID, conservatively matching the existing ledger rule across account/install replacements; otherwise false. The current host selection is not restored from the ledger. An unknown/stale/unavailable installed-scope inventory cannot establish a selected identity or eligibility, even for a public repository. No network enumeration is performed to complete this status call.

Capture the existing host operation/connection generation and current selected identity before any queued blocking work or await; recheck after each await before publishing a selection-bound receipt. Do not hold a state mutex across an await. Disable/disconnect/account or repository replacement invalidates selection-bound work; a late result is discarded with a closed stale/unavailable outcome, not used to restore selection. Any blocking store worker owns the existing native-operation lease until its real return/unwind, even if its async waiter is dropped. This does not require a credential read or native approval and grants neither. The exact guard/ownership callsites require root review before code.

## 5. Historical monotonic schema and future UTC prerequisite

Current canonical record schema 1 records process-monotonic `issuedMs`, `expiresMs` and `dispatchMs`. Valid replay preserves them as historical schema-1 numbers; they cannot be converted to a UTC date, ordered across process restarts as wall time, or represented as actual dispatch attribution. Recovery never tests a historical monotonic expiry to revive authority. Status's schema version identifies the output DTO, not a claim that stored dispatch timestamps are UTC-qualified.

Before any real dispatch, require a separately reviewed record-schema version with explicit host-authored UTC attribution alongside the existing same-process monotonic freshness and connection-generation binding. Review the UTC representation, bounds, clock-source/error/rollback treatment, canonical field order, schema-tagged intent/terminal association, mixed-version replay or new fixed-domain policy, quota accounting and interruption behaviour before code. No UTC field may be inferred/backfilled into schema 1. Never rewrite old encrypted bytes or automatically clear old uncertainty while migrating. UTC evidence is attribution only and must not replace monotonic expiry or restore a grant. This proposal chooses no schema-2 field layout/domain and performs no migration.

Native exact-consent byte binding, complete current installed-scope/operation permissions, after-await generation cancellation, single-flight dispatch, definitive rejection/readback parsing and the stronger installer receipt are still independent prerequisites. A recovery/status PASS cannot satisfy them.

## 6. Minimum deterministic negative fixtures before linking

All new fixture execution is **NOT RUN** in this proposal. Main must approve implementation, review exact source and then obtain fresh qualified results. Minimum cases:

- One shared parser with exact old canonical byte/hash compatibility; the original 27 executable fixtures remain unchanged. Production configuration has only record/recovery code, never the whole test mutation core, transport or plan/permit constructors.
- Fresh valid zero inventory; unmatched intent; explicit attention terminal; all supported valid terminal shapes; verified historical recovery without restored account, repository, plan, approval or active work.
- Every name/schema/sequence/duplicate/canonical-byte/metadata/effect failure above; a valid prefix followed by corruption returns wholly unavailable. Exactly 16-KiB/4-MiB/512 payload boundaries and overflow denial retain the old limits. Foreign-domain ciphertext and writer-lease conflict do not trigger repair or retries.
- Before/after-visible append-failure fixture receipts retain existing poison and no automatic dispatch/retry; complete visible historical terminal may be replayed only as observation. Recovery itself performs zero record appends/clears/mutations.
- Unavailable/unknown status has null counts/predicate; ready/attention counts and predicate are bounded/consistent. No authenticated selection means null, not false. Restored metadata cannot manufacture selection; epoch changes before/after a queued worker/await discard stale binding; dropped waiter retains worker lease.
- Private/profile/body/token/path canaries cannot occur in status, errors, logs, prompts, generation, summaries, exports or tools. Fake counters assert zero network, credential, native-review and mutation calls.
- Any future native recovery fixture uses only a fresh opaque hosted Windows synthetic root, verifies the exact cleanup identity and releases all handles first. Never open owner application data or Credential Manager; test-build memory/native adapter checks are not proof that runtime setup/status callsites opened or qualified a store.

## 7. Evidence and owner-authority boundaries

This proposal creates no runtime source change, executed result, OAuth/live-resource acceptance, causal flicker remedy or qualified installer. Preserve original failures and the later protected-backend source-only review separately. New source invalidates dependent evidence and requires all five exact-source jobs, exactly fifteen codec and six native resource PASS, the independent causal CPU proof and `qualificationComplete=false`; recovery fixtures cannot replace any of these. All 42 compound obligations, unresolved GLib and owner-installed/manual/live/signing/production acceptance remain separate.

Owner authority remains source/documentation PR and checks, not integration/release/signing or agent app/setup execution. Any later live synthetic GitHub operation stays confined to private repository ID `1412554572`, `comzulca/aster-v024-github-test-20261010`, with separate exact native confirmation and evidence preserved outside it before eventual owner-authorised cleanup. Product runtime gets no repository deletion/admin route. Temporary All repositories remains the owner's decision; no permission change follows.

Related: [owned worker leases](V024-NATIVE-DIALOG-LEASE-20261010.md), [diagnostic artifact boundary](V024-DIAGNOSTIC-ARTIFACT-CONTRACT-20261010.md), [local verification successor](V024-LOCAL-VERIFICATION-SUCCESSOR-20261010.md).
