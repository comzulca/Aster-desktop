# Specifications and threat chain
## Stable identifiers
Use PR/FR/UX/NFR for outcomes, features, experience and quality; SEC for a security obligation; TM for threats; AC for acceptance; ADR for boundary decisions; T for executable tests. Allocate ascending identifiers within each namespace, retain retired IDs, and never rewrite a historical migration or reuse an ID. In an existing repo honor its established scheme instead.

For each normative requirement, provide at least one AC; for each high-risk trust boundary provide a SEC, TM, AC and adversarial test. A trace chain should read: FR-001 / SEC-001 -> TM-001 -> AC-001 -> T-001 -> one evidence record. If a requirement has no relevant threat, record the reason rather than fabricating one. Store an architecture/data-flow sketch, classification, actors, abuse cases, mitigation owner, residual risk and ADR for consequential choices.

## Change order
1. State intended behavior, non-goals and data classes.
2. Model authority crossings; decide platform, consent, storage and egress in ADRs.
3. Map requirements to ACs and tests, including denial and crash cases.
4. Write a failing test or exact manual procedure for new behavior.
5. Implement only granted behavior, run checks and record actual outcomes.
6. Reconcile source, registry, docs and evidence; preserve retired history.

Scope questions should focus on facts that cannot safely be inferred. Defaults are mock provider, no network/tools/web, OS-backed secret custody for live use, local history only after explicit data decision, and explicit confirmation for writes. These defaults allow progress without representing mock behavior as provider acceptance.
