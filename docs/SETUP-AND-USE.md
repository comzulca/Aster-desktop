# Set up and use Aster v0.2.4

This guide describes delivered engineering source `57339a6`, reconciled 5 November 2026. The Windows x64 installer is unsigned; automated qualification and the exact hashes are in the [engineering receipt](qa/FINAL-ENGINEERING-V024-20261105.md). The agent did not install or launch it on the owner's laptop.

## Configure before sending

1. Close Aster normally and install the verified engineering setup. Restart Aster using the installed application.
2. Open **Settings → Providers**. Configure/change the direct Meta Model API key only in the native Windows dialogue, then use **Validate key in Windows** as needed. Never paste a key in chat, a repository, a prompt or an ordinary settings text field. Successful model retrieval proves current-key authentication/model visibility, not billing, quota or territorial eligibility.
3. In **Settings → Usage**, enter a current conservative spending configuration: account alias, integer GBP-per-USD rate, safety margin, tax/payment assumptions and a valid expiry. Review it in Windows. The maximum is 31 days and the configuration cannot exceed Aster's current policy deadline. Changing an account alias does not reset charges and may invalidate earlier eligibility; configure the budget before the final terms review.
4. Return to **Providers → Review terms and eligibility in Windows**. Read the complete native proposal and independently check account, territory and direct-API rights before confirming. Coding-harness subscription credentials restricted to that harness must not be reused in Aster.
5. Use synthetic or eligible public content only. Classify the complete message and attachments, choose the required reasoning effort and send only after reading the separate complete native disclosure. Personalization is local-only; it is not sent as model instructions.

The terms button intentionally displays a host-authored JSON proposal in a native read-only reviewer. It is an eligibility/terms operation, not a Meta inference response. Review version 1 binds one exact native operation and is not model approval.

## Current review dates

Terms revision: **9 October 2026**. Geographic policy revision: **12 October 2026**. Engineering review: **5 November 2026**. Aster's next local policy deadline is **6 December 2026 at 00:00 UTC**, not expiry of the Meta account, key or subscription. The original 2 November boundary remains only for historical replay. Old terms acceptance or expired budget configuration is not silently extended by updating the app.

If Aster reports `contributor_policy_expired`, obtain an engineering update with a fresh primary-source review. Accepting the old proposal again cannot extend its deadline. Do not change the system clock or reset protected data to bypass validation.

## Add documents with +

`+` adds to the current unsent selection. Existing rows remain visible while Windows opens the selector. Cancelling retains the previous selection and draft. A known atomic rejection of the new addition retains the previous batch; an unknown/malformed receipt clears displayed IDs conservatively.

Select at most four files in the combined batch. For example, selecting two PDFs and later adding one text file produces three current attachments; it does not replace the PDFs. Remove one row explicitly to make room. The host validates the complete union, with 8-MiB per-source / 16-MiB combined source limits and a total of 20 PDF pages. Additional processed text/image/request limits can reject a send without truncating documents.

Custody remains temporary: the earliest five-minute deadline applies to the whole batch, and adding/removing/cancelling does not renew it. Expiry, chat switch, request consumption/review and restart can revoke originals; this is not cross-chat or disk persistence. Reattach when a new complete reading is required. Removal does not modify or delete the owner's original source file.

| Direct input | Extensions |
| --- | --- |
| Text | `.txt`, `.md`, `.json`, `.xml`, `.yaml`, `.csv`, `.tsv` |
| Images | `.png`, `.jpg`, `.jpeg`, static `.gif` |
| Documents | `.pdf` |

DOCX, XLSX and PPTX require safe external conversion to a complete PDF using the owner's trusted software. Check every page, sheet, slide and print range before selection. Aster does not invoke or require LibreOffice/Office conversion. Animated GIF, malformed/renamed files and over-limit input are denied. A PDF original may contain metadata or hidden content not obvious in the preview; a visible page is not a privacy guarantee.

Saved processed pages can be viewed locally through the native processed-page viewer. This does not restore an original or grant another provider disclosure. See the [attachment contract](providers/attachments.md).

## Spending, history and errors

The local ceiling is £89 per `Europe/London` calendar month, with an £80.10 warning. It is independent of external clients and Meta's invoice. Cancellation/failure after possible dispatch does not prove zero cost. Missing/invalid usage retains the reservation; any unresolved reservation blocks a new dispatch. Reconcile only against an independently checked final statement/dashboard amount for the exact pending request.

Saved conversations and processed derivatives use separate protected local domains. Conversation removal is a logical tombstone: encrypted recovery records and used quota may remain. It is not secure erasure or physical compaction. Restarting, reinstalling or changing a credential is not a spending reset. Private-store integrity/capacity failures require review rather than deletion of records.

## Local features and GitHub

Personalization is retained locally and excluded from provider, generation and tool payloads. Skills/Plugins are bounded inert packages; generated content is an unsaved draft until explicitly saved. Enabling an entry does not grant tools or execute scripts.

For the built-in connector, use **Customize → Plugins → GitHub**, enable its catalogue preference, then start the explicit dedicated-App connection flow. Aster's own account/repository confirmations and credentials are separate from Codex's connection. Reconnect explicitly after expiry/restart; automatic token rotation/account restoration is unavailable. GitHub authentication/read sources are implemented, but actual live connection is not established by hosted fixtures. Runtime file changes, issues, PR creation and merging remain unavailable (`mutationsAvailable=false`). The [registration guide](setup/GITHUB-APP-V024.md) distinguishes historical owner decisions from current unverified account state.

See [current status](ENGINEERING-STATUS.md) for the remaining full-window compaction, 4-GiB migration, startup/flicker, native/live and production obligations.
