# v0.2.6 preparation: Projects and local Markdown memory

Baseline: `main` at `f09f94cbfda83e7d4d371349e8cd8d3d5d41c316`. This is a development contract, not release evidence. The owner supplied `Aster_UI_v0.2.6.png` (1657 × 949, SHA-256 `35a6ee1de48d18a2271827c3a347f628f710704437cd5dcb957bf1d1d18c657f`) as the visual reference. Use existing Aster assets and dark adaptive layout; the PNG is a reference, not a runtime asset.

## Projects

- The left rail presents **Projects**, a collapse chevron, project controls and a distinct **+** action. An empty catalogue says **No projects**. The composer attachment + is unrelated.
- Creating and selecting a project does not send a request, create a vault, import a folder, open a note or create a conversation. Keep access to existing conversations and preserve unsent state. Conversation association is separate from file access or request context.
- Project names and associations remain local to the Windows account. Do not add them to Meta payloads, generated packages or GitHub operations.

## Aster_obsidian

- A skill import remains inert. Build a separate, narrowly typed, owner-operated local vault capability. A project does not automatically become a vault nor authorise access to it. No shell, script, general filesystem API, machine search or automatic cloud fallback.
- Bind file operations to one folder chosen through native Windows UI. If the folder cannot be safely bound, fail closed. Restrict lookups to an explicitly chosen topic; restrict notes to bounded Markdown and reject escapes, reparse points and aliases that undermine custody. Read only on explicit user interaction and keep note bodies out of the web view, telemetry and all provider payloads. Notes are data, never tool instructions.
- A write must show the exact relative path and contents in native UI and require separate confirmation. An existing file must not be silently overwritten. If an edit with a stale-change guard cannot be qualified in this slice, offer creation only and record updates as pending.
- Do not claim automatic recall from conversation text or project switching. Any future use in a model request needs a separate qualified scope, existing Contributor classification and exact disclosure review. Private, personal or confidential content must not go to Meta.

## Acceptance and release boundary

Verify empty and populated navigation, keyboard access, project creation and persisted association, and preservation of drafts and unassigned history. Verify folder selection and cancellation, bounded lookups, native note preview, confirmed creation, no side effects on cancellation, data isolation and no unexpected network activity. Include Windows acceptance using disposable synthetic content, not the owner's vault or credentials. Keep existing release guards, dependency baselines, checks and historical evidence. This preparation does not publish a release, sign or deliver a stable installer, or promote prior CI into qualification of changed source.

## Local verification of this candidate

- TypeScript, production web build and 218 frontend tests passed. Source guards and Rust formatting passed.
- Two Projects store fixtures passed with the in-memory private-store backend. Four portable vault fixtures passed. The vault filesystem/review modules and their test cases type-checked for Windows x64 in a focused harness, not through the full Tauri command router or an installed application.
- Full Windows-target Cargo checking in the Linux workspace stopped in the AWS-LC native dependency build. The full Windows CI, installed visual comparison and native vault acceptance remain pending. Do not infer Windows execution from a cross-target type check.
- The current package/application version was deliberately not bumped in this preparation. No GitHub or release changes were made.

Before integration, use a disposable local vault to check cancellation, title-search limits, an exact native preview, attempted duplicate creation with no overwrite, and restart/reselection. An external edit to an Aster-created note can put the conservative write journal into a verification-required state; check the local note instead of retrying blindly. Existing notes remain available to the native viewer if they meet the read bounds. A separate reviewed change would be needed for editing existing notes with conflict detection or for bringing selected eligible context into a request.
