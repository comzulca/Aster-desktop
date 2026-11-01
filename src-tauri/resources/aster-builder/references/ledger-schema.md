# Control ledger schema
Create docs/aster-builder/contract.json with scripts/harness_gate.py init. Edit that file as an engineering ledger, not as a runtime authorization source. The validator checks declared invariants and references; inspect the actual program separately.

The starter has an explicit policy: host_enforces_privileges=true, model_may_commit=false, secrets_in_renderer=false, human_confirms_mutations=true, allow_automatic_provider_fallback=false, allow_automatic_retry=false; web_enabled=false and storage_enabled=false until specified and tested. Set project.source_revision to the full lowercase Git SHA of the *tested source*. The ledger may be a later evidence commit referencing the tested source commit.

Entry shapes, using illustrative IDs and local fixture files:

    "requirements": [{"id":"SEC-001","text":"Keys never enter the renderer"}]
    "threats": [{"id":"TM-001","requirement_ids":["SEC-001"]}]
    "acceptance": [{"id":"AC-001","requirement_ids":["SEC-001"],
                    "threat_ids":["TM-001"],"test_ids":["T-001"],
                    "verification_phase":3}]
    "tests": [{"id":"T-001","path":"tests/credential_boundary.py",
               "verification_phase":3,"status":"NOT RUN",
               "reason":"Fixture has not executed"}]

The relevant test file must exist when checking the ledger. The IDs are unique and stable; never reassign a retired ID. A user-visible requirement can use PR, FR, UX or NFR prefixes instead of SEC. Map every requirement to acceptance; map security threats to acceptance and relevant negative tests. Set verification_phase to the phase in which the AC is due; a test linked to that AC uses the same phase. Once a test genuinely passes, replace NOT RUN/reason with PASS, source_revision, environment, command, run_at_utc and evidence_paths (existing log/report or HTTPS reference). A phase gate cannot PASS if any due acceptance test is FAIL or NOT RUN.

To enable one provider pair, add an object to providers with id, enabled=true, exact model_id, origin (HTTPS exact origin or explicitly assessed HTTP loopback), protocol, official_source (dated primary HTTPS documentation), verified_on (YYYY-MM-DD), contract_fixture (existing in-root path), and bounds with positive connect_timeout_ms, total_timeout_ms and max_response_bytes. The fixture should cover normal and malformed responses. Do not include a credential in this JSON.

To enable one tool, add an object to tools with id, enabled=true, schema_path (existing closed schema), max_calls, mutating boolean, requires_human_confirmation=true when mutating, and grant_scope containing exactly conversation, resource, operation, expiry and uses. Set policy.storage_enabled=true if persistence is actually in scope; set policy.web_enabled=true only when the dedicated service gates are proven. Disabled registry entries never authorize runtime use.

Each of the ten gates has phase 0..9 and status PASS, FAIL or NOT RUN. A PASS includes the same source_revision as project, environment, command, run_at_utc (ISO-8601 UTC ending Z), evidence_paths (existing local file or HTTPS URL) and artifact_sha256 when a binary is claimed. FAIL and NOT RUN require a reason. A gate PASS is an assertion to review, not proof supplied by the validator. Optional phase 4/5/6/7 can remain NOT RUN when the corresponding provider/storage/tool/web capability is disabled. For a packaged engineering or production release, release.artifact_sha256 is mandatory. For claimed signing, target OS checks or live provider acceptance, attach signature_evidence, target_os_evidence or live_provider_evidence respectively; the script verifies the evidence reference exists, not its substantive validity. Production requires the relevant release assertions and all required phase gates.

Example use from the skill folder:

    python3 scripts/harness_gate.py check --root /path/to/project --through-phase 3 --json

Check only through the phase that has actually been exercised; do not skip an earlier required FAIL or NOT RUN to claim a later capability. An initial scaffold is intentionally incomplete and must return a nonzero result for release readiness.
