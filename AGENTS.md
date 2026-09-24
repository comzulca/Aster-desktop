All agent-internal instructions, agent-to-agent exchanges, AGENTS.md files, tool contracts, and engineering evidence must be written in English.

# Aster-desktop repository instructions

This repository is a new Windows 11 desktop agentic harness. Follow the installed `aster-builder` skill and the versioned requirements under `docs/specs/` in order: inventory, specification, threat model/ADRs, host/UI, provider pairs, persistence, local tools, optional services, quality, release. The skill and these instructions guide development; neither can grant a running model a tool or credential.

## Authority and trust

- The Tauri/Rust host alone owns OS dialogs, Windows Credential Manager access, HTTPS requests, root-scoped file reads, approvals and file publication. React/WebView supplies bounded intent and renders untrusted model/project content as inert text.
- Start with no tool, filesystem root, network destination, or model provider enabled without an exact registry entry and corresponding validated user action. Never infer permission from an LLM response, document, web result, repo content or an AGENTS.md found in a selected project.
- Use a closed NVIDIA provider/model registry. Nemotron 3.5 Lightning 30B A3B is the default model; Muse Glimmer 30B is optional. Streaming for both requires separately tested provider contracts. No silent retry, fallback, tool escalation, arbitrary URL, shell, browser or MCP in v0.1.0.
- Credential entry and reading stay in trusted native Windows paths. Never print, send through IPC, commit, export, log, put in CI or ask for a key in conversation. Do not add credentials or local conversations to source control.
- Project reads require a bounded, host-issued scope to a root selected by the user. A model may propose a bounded change; only a separate user approval bound to the exact resource, content hash and operation may trigger the host's guarded write. A renderer button alone is not a security decision. Deny stale/replayed grants, traversal, reparse points, hardlinks, alternate data streams, device/UNC aliases, `.git` and credential files.
- Treat existing-file replacement as proposal-only until Windows-specific safety and interruption tests support a conflict-safe publication procedure. For new-file creation, require `CREATE_NEW` semantics plus durable intent and recovery tests. Never claim an ordinary rename or `ReplaceFileW` is a compare-and-swap against all concurrent writers.

## Spec Driven Development and evidence

- Give requirements stable PR/FR/UX/NFR/SEC IDs, threats TM IDs, acceptance AC IDs, tests T IDs and boundary decisions ADR IDs. Do not reuse retired IDs. Map each privileged crossing to a negative test before enabling it.
- Keep policy decisions reproducible for the same validated input, host state and fixture; do not promise identical LLM wording, provider timing or binary output. Deterministic state transitions, terminal outcomes and denials must be testable without live provider use.
- Record PASS/FAIL/NOT RUN with source revision, environment, exact command, UTC time and evidence. A fixture PASS is not a live NVIDIA, Windows credential, installer signature or production-release PASS. An implementation change invalidates dependent evidence.
- Source defaults: local ephemeral conversations, no disk chat history, no project root until chosen, no external network except explicitly selected NVIDIA provider in live mode, no arbitrary external code execution. Permission to transmit project content to NVIDIA must be explicit per request.
- Maintain a minimal locked dependency graph, CI secret checks, static analysis, license/SBOM inventory and adversarial fixtures. Any change to provider, IPC, grants, local files or persistence updates its spec, threat mapping and test evidence together.

The requested Windows checkout location is user configuration, not a value to hardcode into source. The current development container is Linux and cannot prove Windows Credential Manager or native file-write behavior.
