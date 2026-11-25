# Set up and use Aster v0.2.5

This guide covers v0.2.5 source including the dictation and credential fixes in [PR #87](https://github.com/comzulca/Aster-desktop/pull/87) and the Windows version lifecycle in [PR #90](https://github.com/comzulca/Aster-desktop/pull/90). A previously installed v0.2.5 build may not include these changes. See the [dictation specification](specs/SPEC-0009.md).

## Dictate a draft

Select the microphone beside the reasoning selector and approve the native microphone prompt. Speak into the default Windows input device. Select **Finish dictation** to transcribe. Capture is limited to 30 seconds. Review the text in the composer before classifying and sending it. Dictation uses the bundled multilingual CPU model and works independently of Meta authentication.

Windows desktop microphone access and the x64 Microsoft Visual C++ runtime, including VCOMP140.dll, are required.

### After cancelling dictation

**Cancel dictation** discards the pending transcript. Aster waits for the original capture to confirm release; a stop acknowledgement alone is not enough. If release is unconfirmed, check the Windows microphone indicator and follow Aster's restart guidance before dictating again.

## Configure before sending

1. Close Aster normally and install the verified engineering setup. Restart Aster using the installed application.
2. Open **Settings → Providers**. Configure/change the direct Meta Model API key only in the native Windows dialogue, then use **Validate key in Windows** as needed. Never paste a key in chat, a repository, a prompt or an ordinary settings text field. Successful model retrieval proves current-key authentication/model visibility, not billing, quota or territorial eligibility.
3. In **Settings → Usage**, enter explicit integer GBP-per-USD conversion, positive safety margin and tax/payment assumptions. Choose a future expiry within the displayed local host limit: at most 31 days. Select **Review financial configuration in Windows**, confirm the exact native proposal and check **Policy status: Active** and its confirmed expiry. Filled fields or a cancelled review do not activate a policy. Account identity and earlier charges are retained, without reset. Confirm a new London calendar month before sending in that month, even if the previous review's displayed expiry has not passed.
4. Return to **Providers → Review terms and eligibility in Windows**. Read the complete native proposal and independently check account, territory and direct-API rights before confirming. Coding-harness subscription credentials restricted to that harness must not be reused in Aster.
5. Write a non-empty eligible message. Open the reasoning menu (initially **Medium**) → **Classify content** → **Synthetic** or **Eligible public**, choosing correctly for the complete message and attachments. Read the composer prerequisite notice if Send is disabled. Send opens a separate complete native disclosure review before dispatch. Personalization remains local-only.

The terms button intentionally displays a host-authored JSON proposal in a native read-only reviewer. It is an eligibility/terms operation, not a Meta inference response. Review version 1 binds one exact native operation and is not model approval.

### If a previously configured key needs attention

If Aster cannot read Windows Credential Manager, check access under the same Windows account and return to Aster to refresh the status. That warning does not mean the key is missing. If Aster reports the key as missing, check whether the generic credential `AsterDesktop/MetaModelAPI/v1` is listed.

**Validate key in Windows** checks model access with one request. A network interruption can leave a known key unverified without changing its identity. Revalidating that same known key does not itself reset Contributor consent or continuation. A changed or unrecognised key can require fresh consent and invalidate temporary attachments and continuation. Saved history, spending records and pending charges are preserved.

## Windows version lifecycle

A build with [the new lifecycle policy](qa/RELEASE-LIFECYCLE-20261125.md) checks public GitHub Releases for a strictly newer stable version with a matching Windows x64 installer. The 14-day grace period starts when that installer and its stable release are both available. A PR, CI artifact, prerelease or failed connection cannot start the clock. A connection failure does not erase a previously confirmed deadline. Aster does not download or install updates automatically.

When Aster reports `aster_version_expired`, use the current stable Windows installer. New sends stop, but saved history and charge reconciliation remain available. The deadline does not expire a Meta account, key or subscription. Older builds with the fixed **6 December 2026 at 00:00 UTC** cutoff retain it until replaced; a merge or CI artifact does not update an installed copy. Do not downgrade over protected stores after the updated build has written v2 terms or budget records.

Terms revision: **9 October 2026**. Geographic policy revision: **9 November 2026**. Engineering review: **25 November 2026**. Existing financial dates and historical replay windows are preserved. If the updated build reports `contributor_policy_expired`, check the Windows date and time rather than resetting protected data.

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
