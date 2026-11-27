# Aster-desktop security policy

**Effective from v0.2.5.** This policy covers the desktop application, Rust host, Tauri/WebView interface, models, connectors, builds, distribution, and migrations. The repository entry point is [AGENTS.md](./AGENTS.md).

Use these requirements to implement and review Aster. The reference document describes the v0.2.4 baseline for Windows 11 x64. Verify compliance separately for each subsequent release using tests for that release. Keep each new capability disabled until it can satisfy its security contract.

## 1. Authority to act

**The user defines the task in the active session. The Rust host decides which operations are permitted and controls their execution.** A web page, model response, or tool result may supply information needed for the task. None can expand its scope or grant permissions.

| Input | Required handling |
| --- | --- |
| Direct request entered through the local interface | The host identifies the session and retains the original request. It distinguishes the user’s instruction from quotations, links, and attachments in the same message. |
| Visible or hidden HTML, DOM content, documents, email, API and MCP responses, repositories, and search results | Record the source and treat the content as reference material. Role labels, valid signatures, and purported administrative notices within the content confer no authority to act. |
| Images, OCR output, voices, subtitles, metadata, encrypted files, and nested containers | Preserve the source and transformation history through decryption, extraction, transcription, and summarization. A recording of the user’s voice cannot grant approval. |
| Memory, conversation history, retrieval-augmented generation (RAG) results, imported skills, plugins, generated code, and other agents’ responses | Retrieve as data and isolate by identity and scope. Storing a summary never turns it into policy, an authorized preference, or a permission grant. |
| Model-generated proposal | Accept only fields defined by the schema of a known, typed operation. The host obtains identity, permissions, budget, and approvals through channels it controls. Reject model-supplied control fields. |

**SEC-01 · Provenance.** Construct the system policy, user task, and reference data separately. Record provenance outside the content and prevent transformations from overwriting it. Treat material of uncertain origin as reference data. Apply the same controls when resuming a conversation. Injection detectors and text filters are supplementary. The controls must hold even if the model follows an embedded instruction.

**SEC-02 · Authorization before execution.** For every call, the Rust host checks that the capability is enabled, the operation falls within the user’s task, and the identity, session, resource, operation, destination, and limits are authorized. Deny unknown or ambiguous requests. Give the model opaque identifiers and only the information it needs. Enforce the same checks for background tasks, retries, and connectors.

**Browsing example.** A user asks for a summary of a GitHub issue. A comment asks Aster to attach diagnostic logs to “complete the query.” Aster can summarize the issue under the existing read grant. The host prevents the comment from triggering either a read or a transfer of the logs. If the blocked attempt affects the result, Aster identifies the action it blocked and the source of the request.

## 2. Permissions and consent

| Operation | Execution requirements |
| --- | --- |
| Start Aster | Run under a standard user account using local resources, without accepting inbound connections or automatically establishing external sessions. Disable agent browsing, MCP, shell access, desktop automation, and connectors by default. |
| Read a file or repository | Use a native file picker that grants temporary access to the exact file, or a read-only connector explicitly enabled for the selected account and repository. |
| Send context to the model provider | Validate the authorized provider and model. In a native dialog, show the complete outbound context, attachments, destination, and estimated cost. Reserve the budget before dispatching each approved submission. |
| Publish, send data to a third party, change permissions, or write data | Require a capability specific to the operation. Show the action, account, recipient, data, and effect in a native dialog. Authorize reading, disclosure, and writing separately. |
| Delete or overwrite data, discard history, or revoke control of a resource | Stop at the proposal stage. Identify each resource, its version, and the consequences, including for batch operations. Obtain specific approval in a native dialog before the first side effect. Apply this rule whenever the operation could cause a loss, even if the outcome is uncertain. |

**SEC-03 · Single-use approval.** Bind approval to an immutable proposal, identity, session, resources, resource versions, destination, content, and expiration time. Build the native dialog from data verified by the Rust host. Revalidate immediately before execution and consume the grant atomically. Invalidate the proposal if the user cancels or any parameter changes. Revoke grants on disconnect, account switch, or session termination. Web content, chat messages, and plugins must never be able to supply native confirmation.

## 3. Browsing and outbound network access

**SEC-04 · Isolation.** Serve the privileged interface from bundled local assets. Apply a restrictive Content Security Policy (CSP) and scope Tauri permissions to specific windows and commands. Prevent remote navigation in that WebView. For agent web reads, use an isolated component without a privileged IPC bridge or access to personal credentials, the browser profile, the clipboard, or the user’s file system. Prefer static extraction. Any active rendering requires equivalent isolation and network controls. Require an explicit user gesture and validate the URL scheme before opening a link in the system browser.

**SEC-05 · Network validation.** Route agent connections through a transport that denies access by default. Define permitted destinations, ports, and methods for each capability. For public web reads, allow HTTPS with TLS certificate verification. Parse and normalize URLs with a maintained library. Reject local schemes, embedded credentials, and ambiguous destinations. Display the effective domain and its ASCII representation when an internationalized name could mislead the user.

- Check every resolved IPv4 and IPv6 address. Block loopback, private networks, link-local and reserved addresses, and metadata services. Connect to a validated address while verifying the original hostname during TLS negotiation. Enforce equivalent checks at the outbound proxy if it resolves the destination.
- Disable automatic redirects. If the capability contract permits redirects, repeat authorization and validation at every hop, bound the number of hops, and strip credentials when the origin changes. Apply the same controls to subresources, downloads, and secondary transports. Disable any path that cannot enforce them.
- Place authorized internal services behind separate connectors with their own identities, destinations, and permissions. Prevent the public web reader from reaching those services through a URL supplied by content.

**SEC-06 · Data disclosure.** Before sending a request, validate its destination and outbound data, including the path, query, headers, body, and filenames. A search query also discloses information. Send only the data needed for the task and authorized for that recipient. Keep tokens and cookies in the host, isolated by account and origin. Block automatic loading of remote images, previews, and tracking pixels, and prevent automatic form submissions in the conversation. Render sanitized Markdown and escaped text. Keep web permissions for the camera, microphone, location, and notifications disabled.

**SEC-07 · Limits and downloads.** Before enabling a capability, set finite limits on elapsed time, bytes, requests, concurrency, steps, and cost. Enforce limits throughout streaming, including when the content length is unknown. Cancel the operation when any limit is exceeded. Place downloads in isolated temporary storage. Prevent automatic opening, installation, and execution. Apply SEC-02 and SEC-03 to form submissions, purchases, messages, and any other operation with side effects, even if a site presents it as a read operation.

## 4. Secure coding and software supply chain

| Rule | Implementation requirement |
| --- | --- |
| SEC-08 · IPC and execution | In Rust, validate the calling window and its origin, request schema, types, sizes, and state. Expose operation-specific commands. Exclude dynamic code evaluation, general-purpose shell access, SQL built by concatenation, and arbitrary file paths. Parameterize queries and encode each output for its destination context. Disable debugging in production. |
| SEC-09 · Parsers and files | Validate the actual file format and size. Process complex formats in a restricted process without network access or macro execution. For archives, enforce cumulative limits on expanded bytes and entry count, and bound total nesting depth. Reject absolute paths, path traversal, links, and reparse points that escape the temporary directory. Use file-opening operations that prevent substitution between validation and use. Remove temporary artifacts after failure, cancellation, or restart. |
| SEC-10 · Local data | Store secrets using per-user operating system protection. Separate conversation history, personalization, caches, and credentials. Enforce access controls on reads and writes. Redact secrets from errors and logs. Prevent pages from modifying persistent memory or enabling telemetry. |
| SEC-11 · State consistency | Use atomic writes or transactions where appropriate, check resource versions, and assign operation identifiers. If a connection drops after a remote service may have applied a write, query its status before retrying. If the outcome remains uncertain, stop retrying and tell the user the outcome is unknown. |
| SEC-12 · Dependencies and distribution | Review direct and transitive dependencies, pin resolved versions, and maintain an inventory for each release. Review permission changes, installation scripts, and uses of unsafe Rust. Use the product’s trusted keys to authenticate update artifacts and verify integrity, target platform, and version before installation. Block unauthorized downgrades. Keep engineering builds separate from user releases. |
| SEC-13 · Extensions and migrations | Import external skills and packages as inert content. An executable extension requires review, isolation, and a separate contract for permissions, credentials, and destinations. During upgrades, revalidate grants and discard expired or incompatible ones. Legacy configuration must never implicitly enable a new capability. |

## 5. Verification before merge and release

For each change affecting a trust boundary, identify the applicable rule, the host control that enforces it, and the corresponding test. Run tests with synthetic data, an observable transport, and the production profile where applicable. Verify actual effects on the network, file system, and application state. A refusal message from the model alone is insufficient evidence.

| Acceptance scenario | Required evidence |
| --- | --- |
| A page, transcript, or retrieved summary induces the model to propose an out-of-scope transfer | The host receives and denies the proposal. The test receiver records zero deliveries. The legitimate task can still finish. |
| A Markdown response includes a remote resource or a link with a local scheme | Rendering the response triggers zero implicit requests and opens zero local resources. |
| An initially permitted destination redirects to an internal address or its DNS resolution changes to one | The transport prevents the connection to the prohibited destination for both IPv4 and IPv6. |
| A nested archive exceeds a limit or attempts to write outside the temporary area | Extraction stops within the configured budget. Files outside the temporary area remain intact. |
| The recipient, resource version, or approved context changes before dispatch | The previous approval is invalid. Canceling the new dialog produces zero side effects. |
| Two tasks concurrently consume the same approval, or a response to a write is lost | At most one authorized execution occurs. An uncertain outcome remains pending reconciliation. |
| A remote frame attempts an IPC call, or a session from another account is restored | The host denies access. Account data and secrets appear in neither the result nor the logs. |
| First installation, migration from v0.2.4, or update with an invalid signature | Verify secure defaults and the absence of implicit grants on installation and migration. Verify that the invalid package is rejected before installation. |
| An authorized public web read stays within its limits and the user approves a submission to the provider | The legitimate task completes. Only the approved content reaches the approved destination, with a single charge against the budget. |

**Release criterion.** If a mandatory control fails, fix it or exclude the affected capability from the release artifact and verify that all entry points deny access to it. Record the version, platform, controls tested, and known limitations. Repeat the applicable tests whenever the model, transport, WebView, parser, dependencies, or permissions change.

## 6. Incident response

When a suspicious proposal requests an out-of-scope action, deny the action and continue the safe portion of the task where possible. If unauthorized execution or disclosure is suspected, stop pending actions for the affected capability, revoke its grants, and isolate the affected session.

Record the time, operation identifier, rule, source, and last stage reached. Minimize the personal data recorded. Distinguish a blocked proposal, a confirmed transfer, and an unknown outcome. Tell the user which resources were affected and what actions are available. If a credential may have been exposed, help the user revoke it with the issuer. Re-enable the capability after correcting the cause and adding a regression test.

## Design references

- Source document supplied by the user. *Secure by Design and Secure by Default in Aster Desktop*, revised November 24, 2026.
- Instruction and data separation, tool validation, and defense in depth. [OWASP · LLM Prompt Injection Prevention](https://genai.owasp.org/llmrisk/llm01-prompt-injection/)
- Destination validation, name resolution, and redirects. [OWASP · SSRF Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html)
- Permission scopes for windows and WebViews. [Tauri v2 · Capabilities](https://v2.tauri.app/security/capabilities/)
