# GitHub setup and disposable-repository checkpoint — 10 October 2026

Scope: owner-controlled setup and read-only Codex connection verification, not Aster live acceptance. The document date uses Europe/London (BST); the actual UTC observations below occur on 9 October before UTC midnight. Local development baseline is main `d8b9422eab83526a67bd16a23fbb0d029870d709`, branch `codex/v0.2.4-engineering`, with uncommitted v0.2.4 work. No new commit, installer or release is qualified by this receipt.

Current applicability — later 10 October source: the [controlled integration contract](V024-INTEGRATION-CONTRACT-20261010.md) and [implementation checkpoint](V024-IMPLEMENTATION-CHECKPOINT-20261010.md) extend the earlier offline-only module into partial closed host adapters/local domains and record PR/check permission, the inert-bundle rights decision and owner-run diagnostic. The dated observations, hashes, counts and earlier NOT RUN limits below remain bound to their original cutoff; they are not rerun or relabelled. They still do not prove Aster OAuth, repository mutations or installer acceptance.

## Owner setup evidence

The owner manually registered **Aster Desktop Engineering** under `renenehru`, App ID `5256613`, slug `aster-desktop-engineering`, public page `https://github.com/apps/aster-desktop-engineering`. The owner copied the public Client ID into this chat as text: `Iv23liKA8uYEYklVfzFF` (20 ASCII characters, case-sensitive). This resolves the public registration identifier dependency without requesting any credential. An identifier is not authentication or operation authority.

The supplied registration screenshots show Device Flow enabled, installation-triggered OAuth disabled, blank redirect/setup URLs, wildcard matching disabled, and webhook Active disabled. The optional-feature screenshot displays **Opt-out** for user-to-server token expiration, consistent with expiration being enabled. Permission screenshots show Contents/Issues/Pull requests read and write, Actions/Checks/Commit statuses read-only, and mandatory Metadata read-only. Other displayed repository permissions remain No access; no assertion is made about unexpanded or unobserved settings.

The initial portal screenshot displayed a private-key installation warning; later screenshots reached installation review. This receipt neither dismisses the warning nor infers that a key was generated. The distributed native device-flow design embeds no Client Secret/private key, and this work requested, received and handled none. Portal owner setup and distributed client credential design are distinct.

The owner reports installation on COMZULCA and expressly chooses **All repositories** temporarily, with a manual change to **Only selected repositories** at process end. The screenshots/owner report are setup evidence, not an independent saved-installation API read or Aster token proof. The ambient installation URL is not treated as authoritative proof. The default narrow-installation recommendation is preserved in the [setup guide](../setup/GITHUB-APP-V024.md) alongside this dated owner exception.

The owner manually created `comzulca/aster-v024-github-test-20261010` as private with an initial README on `main`. The agent did not create it. The owner authorises eventual removal after tests while preserving generated documentation; that authority targets this disposable infrastructure only, not production-source publication or a general Aster delete tool.

## Actual read-only Codex observations

At `2026-10-09 23:48:06 UTC`, three completed read-only connector calls returned successfully:

| Read | Observed non-secret result | Evidence limit |
| --- | --- | --- |
| Authenticated user login | `renenehru`, user ID `258826763` | Codex connection identity, not Aster's App identity/token. |
| Repository metadata for exact full name | `comzulca/aster-v024-github-test-20261010`; repository ID `1412554572`; owner `comzulca`, organisation ID `325269963`; visibility `private`; default branch `main`; archived `false` | Exact target readable through Codex; not an Aster account connection. |
| Exact `GET /repos/comzulca/aster-v024-github-test-20261010/contents/README.md?ref=main` | Heading `# aster-v024-github-test-20261010` | Read-only content access; no branch/file mutation. |

The metadata reports `pull=true` and `push=true` for the active Codex connection. These are reported permissions, not proof of an executed write or permissions held by the dedicated Aster App. No write was attempted or performed.

At `2026-10-09 23:48:11 UTC`, an independent file fetch for `README.md` at known ref `main` returned the same heading and blob SHA `f74dcfe2df51ba3e188e50db980fa6118b6704c6`. A local `git rev-parse HEAD` returned the baseline SHA above. The receipt retains compact results, not raw credentials, private payload archives or unrelated repository content. Read-only access checks are PASS only within this explicitly stated Codex scope.

## Test and cleanup obligations

Actual testing must target only stable repository ID `1412554572` under `comzulca/aster-v024-github-test-20261010`. Broad installation availability does not authorise tests or mutations in `comzulca/Aster-desktop` or any other repository. Fixtures must be synthetic and non-secret; no owner data, tokens, Personalization or private code may enter Meta Contributor. All product changes retain their independent native exact confirmation/revalidation requirements.

Before eventual deletion:

1. Preserve generated documentation, operation/results receipts, synthetic fixture revisions and relevant issue/PR/check evidence outside the disposable repository. Record exact source/resource identities and hash the retained evidence; do not export tokens or unrelated private content.
2. Re-read the exact repository's stable ID/full name and inventory. If owner-added unrelated content or uncertain identity is found, stop rather than deleting it under the original disposable scope.
3. Verify the evidence archive is complete and readable before cleanup. A failing or incomplete test remains recorded as such; deletion must not erase the only failure evidence.
4. Remove only the authorised disposable repository through the owner's manual action or a qualified available management route, and verify/report the outcome. Currently available tools provide no repository-deletion operation. **All repositories** does not grant Aster creation/administration/deletion functionality, and no such product privilege is added to solve infrastructure cleanup.

No fixture creation, mutation, PR, issue, repository deletion or permission change occurred in this checkpoint. The temporary repository still exists at the observations above.

## Narrow source verification

The dirty source now preserves the exact public Client ID in `REGISTERED_CLIENT_ID` and exposes fallible `AppClientId::registered()` through the existing closed parser. The new authored offline regression is `registered_client_id_preserves_case_without_enabling_connection`; the module now contains 12 authored offline Rust tests, **not executed tests**. The independently read module SHA-256 is `879451c8479aff923413bd348056b0aa508e437fb5f9e43fb01a0edc948099be`. No new IPC, network, credentials or dependency is enabled by this identifier change.

At `2026-10-09 23:49:38 UTC`, the main agent completed these separate local source checks with exit code 0:

- `rustfmt --edition 2021 --check src-tauri/src/github_connection.rs`, using `rustfmt 1.9.0-stable (2d8144b788 2026-07-07)`, toolchain `1.97.0-x86_64-pc-windows-msvc`.
- `git -c safe.directory=C:/Users/Rene Nehru Alvarez/Documents/aster-desktop-premium/Aster-desktop-v0.2.4 diff --check` (tracked whitespace only).

These are parse/format and whitespace checks, not Rust compilation, unit execution, host/CI/live or package qualification. The documentation worker's independent whitespace check also exited 0. Its first relative-link checker command failed on a PowerShell string-interpolation syntax error before running any file checks; after correcting the checker, the completed check at the `2026-10-09 23:49:47 UTC` observation returned three owned documentation files, 15 relative links and zero missing targets or trailing-whitespace failures. This documentation check does not validate external links, native behaviour or product acceptance.

The main agent cross-read the setup guide, index and this receipt, and reconciled SPEC-0006 revision 2, THREAT-MODEL-0006, ADR-0021, AGENTS.md and the README. At the `2026-10-09 23:51:57 UTC` observation, the expanded local documentation check completed with exit 0 for eight files and 95 relative file links, with no missing targets or trailing-whitespace failures; tracked `git diff --check` also exited 0. An independent source reviewer confirmed the identifier/factory/non-activation fixture and the module hash above with no blocking static finding. This is cross-document/static evidence only, not compiled or executed Rust, native acceptance or GitHub publication.

## Product and qualification limits

Aster's GitHub module remains an offline-only connection/proposal contract, without live OAuth, HTTP/token custody or a privileged runtime route. The public Client ID is preserved in the closed offline host contract; recording it does not enable network or account authority. Rust compilation and the new GitHub Rust unit execution, native/live account authorisation, credential rotation, writes/integration, account switching, installed UI, native flicker, hosted CI, package gates and v0.2.4 installer qualification are **NOT RUN** by this checkpoint. Earlier separately scoped headless/source evidence is not rerun or promoted here.

The required five-job, fifteen-codec and six-resource gates, all 42 compound obligations, unresolved GLib and separate owner-installed/manual/live/signing/production obligations remain intact. No earlier v0.2.3 PASS transfers to changed source. No v0.2.4 production-source PR/merge, live OAuth/Meta, security/protection change, signing, release or redistribution authority is inferred from the public Client ID or disposable-repository permission.

References: [SPEC-0006](../specs/SPEC-0006.md), [THREAT-MODEL-0006](../security/THREAT-MODEL-0006.md), [ADR-0021](../adr/ADR-0021.md), [v0.2.4 documentation index](../V0.2.4-DOCUMENTATION-INDEX.md).
