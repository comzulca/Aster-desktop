# Current attachment contract — v0.2.4

The attachment behaviour from `57339a6` is unchanged by the [spending/Send correction](../qa/SEND-READINESS-20261105.md). Current IPC protocol is 5; financial policy remains 4. The earlier receipt qualifies only its recorded source. [SPEC-0007](../specs/SPEC-0007.md), [ADR-0022](../adr/ADR-0022.md), [threat register](../security/THREAT-MODEL-0007.md), [operating guide](../SETUP-AND-USE.md).

## Selection and custody

The renderer sends only a bounded conversation intent. The native host authenticates the caller/current key, selects original paths through Windows, reads identity-safe bounded immutable copies and processes them in the existing restricted worker. No source path or original binary data crosses renderer IPC. Metadata receipts contain opaque IDs, labels, kinds, byte/page counts; they are not filesystem or disclosure authority.

`+` snapshots the exact current conversation's host-owned IDs, opens the selector while previous rows remain visible and atomically validates the full old-plus-new allocation before commit. Cancellation returns the complete current metadata. Known pre-commit read/format/capacity rejection retains the prior batch and draft; an unknown/malformed receipt clears renderer IDs conservatively. A renderer-created merge never grants custody.

The combined batch has the earliest original five-minute deadline. Adding, removing or cancelling cannot renew it; newly appended sources are capped to that deadline before later consumption. Expiry during selection or a changed snapshot cannot resurrect the previous grant. Chat switch, one-use send/review consumption, restart and expiry retain their revocation rules. Explicit removal affects one temporary item, not the original source file.

## Formats and bounds

| Category | Admission |
| --- | --- |
| Text | `.txt`, `.md`, `.json`, `.xml`, `.yaml`, `.csv`, `.tsv`; strict complete eligible text |
| Images | `.png`, `.jpg`, `.jpeg`, static `.gif`; normalised complete image |
| PDF | `.pdf`; bounded complete original for native reviewed egress, derived pages for local preview/history |
| Office | DOCX/XLSX/PPTX require safe external PDF conversion; no internal Office/LibreOffice execution or dependency |

| Boundary | Current value |
| --- | --- |
| Combined files | Four |
| Original source | 8 MiB each / 16 MiB total |
| Processed content | 4 MiB each / 16 MiB total |
| Processed text parser | 128 KiB each / 256 KiB batch |
| Visuals | 4,096-pixel maximum dimension, 4,194,304 pixels per page, 64 MiB batch RGBA |
| PDF | At most 20 pages per PDF and 20 PDF pages across the combined batch |
| Temporary custody | Earliest immutable 300-second deadline |

All applicable limits apply together. The composed text prompt still has its smaller 32-KiB message bound; a parser-admitted file is not a promise the whole conversation/send will fit. Complete media serialisation is separately capped at 24 MiB, including base64/JSON overhead. No content is silently truncated to fit.

## Disclosure, processing and restoration

Only synthetic/eligible public content is admitted. A source can contain sensitive/hidden metadata not evident in its preview; native confirmation is not a privacy-policy exemption. The host presents complete immutable outbound material and native visual pages, then separately revalidates deadline, key/terms/financial state before dispatch.

Text becomes inert processed context; images use normalised image payloads and PDFs use the complete temporary original with host-selected safe filenames. The host preserves processed derivatives/responses in a separate protected-media domain; originals and grants do not become durable history. Local processed-page viewing is not original reconstruction or another disclosure approval. Reattach the original for a new complete native reading.

Malformed, renamed, oversized or animated input is denied. Source reading/processing never authorises a shell, Office macro, generic project read, arbitrary process/MCP or model-directed filesystem mutation. Zeroising owned buffers is not a guarantee of secure erasure from OS paging/dumps or other copies.

## Evidence

Host/Rust and frontend regressions cover additive selection, cancellation/rejection retention, aggregate source/text/count limits, duplicate IDs, cross-chat/stale snapshots, consumed deadlines and expiry. Main run `42081672227` passed the actual fifteen codec and six resource cases, including native two-page PDF and causal CPU evidence. [Exact receipt](../qa/FINAL-ENGINEERING-V024-20261105.md).

This is engineering automation, not owner/live multimodal, native GUI or all-compound qualification. Earlier v0.2.3 failures/contracts are preserved in [meta-attachments-v0.2.3.md](meta-attachments-v0.2.3.md); historical replacement-selection and NOT RUN statements are superseded only within the current explicitly verified scope.
