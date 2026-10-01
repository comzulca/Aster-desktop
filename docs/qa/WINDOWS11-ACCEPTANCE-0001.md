# WINDOWS11-ACCEPTANCE-0001 — unsigned engineering acceptance

Created 2026-09-30 UTC; reconciled 2026-10-01 UTC for protocol 2 and the owner's latest test decision. **All target-laptop rows are NOT RUN until executed and recorded against the verified engineering setup.** Run this issue-#11/#12 procedure on this current primary Windows 11 laptop, under the owner's existing principal account only, after internal artifact verification and before final delivery. Do not create/use a guest or new account. The owner selected an unsigned engineering build; no self-signed certificate, trust-store change or production signature is required or claimed. Hosted Windows Server 2025 cannot satisfy this procedure. A test-only seam absent from the shipping app remains NOT RUN rather than being fabricated.

## Authority and environment

- Use only the owner's existing principal account `QUARK2\Rene Nehru Alvarez`; launch PowerShell and Aster Desktop normally/non-elevated. Read-only baseline `e6418f` at `2026-10-01T17:03:47.5227862Z` recorded SID ending `1001`, `ElevatedAdministrator=false`, Windows 11 Pro 64-bit `10.0.26300`. Accept installer UAC only if required and record it. Do not create/use a guest or new account.
- The owner reports personally enabling Developer Mode on 2026-09-30. Record that as an owner-made baseline exception, not an agent change, a PASS or authority for another security change.
- Leave all remaining Windows Security, SmartScreen/Smart App Control, ESET, account, trust and policy settings unchanged. Do not add exclusions, bypass a block, install a certificate, create/switch accounts or elevate the app. A protection block is a retained FAIL for the attempted step; dependent steps stay NOT RUN.
- Use disposable non-sensitive text only. Do not capture credentials, private prompts/project text, Authorization headers, raw request bodies or unrelated Credential Manager entries.
- Preserve the owner's existing NVIDIA credential. Never replace, export, display or delete it. Use only the isolated synthetic credential diagnostic. Exactly two minimal live requests—one per exact model—are authorized; no retries, extra live error/cancellation case, fallback or project upload is authorized.
- Baseline `e6418f` found a separate existing **Aster 0.10.0** installation: HKCU uninstall entry `Aster`, publisher `aster`, installed at `C:\Users\Rene Nehru Alvarez\AppData\Local\Aster`, executable product name `Aster 0.10.0`; the new `Aster Desktop` directory did not exist. The owner has chosen to uninstall the older product manually. Do not perform or assume that uninstall: block candidate installation until the owner reports completion and a read-only preflight confirms the old entry/directory/processes are absent. Preserve the NVIDIA credential. The candidate must still use the distinct `Aster Desktop` product identity/path; any attempt to target the old location is FAIL.

Historical local/CI results remain in `LOCAL-0002.md` through `LOCAL-0004.md` and `CI-0002.md` through `CI-0006.md`. In particular, run 85 and its `b0610c5` installer predate protocol 2 and are not the candidate for this procedure.

## 1. Bind one immutable candidate

Select the newly successful CI run for the protocol-2 source. Do not use a filename or version string as provenance. Record repository privacy, intended `main` revision/tree, ruleset/required-check state, run/attempt, manifest and artifact identity without changing permissions.

In normal PowerShell:

```powershell
(Get-Date).ToUniversalTime().ToString('o')
Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, BuildNumber
$asterSource = Join-Path $env:USERPROFILE 'Documents\comzulca\aster-desktop'
$asterGit = 'C:\Program Files\Git\cmd\git.exe'
& $asterGit -C $asterSource remote get-url origin
& $asterGit -C $asterSource rev-parse HEAD
& $asterGit -C $asterSource status --short
Get-FileHash -LiteralPath (Join-Path $asterSource 'public\assets\Aster_UI2.jpg') -Algorithm SHA256
```

Expected origin: `https://github.com/comzulca/Aster-desktop.git`. Expected image SHA-256: `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887`. Record any dirty checkout; do not overwrite it or infer the packaged source from it.

Fill these values from the **new** run:

| Identity | Current result |
| --- | --- |
| Source commit/tree | NOT RUN |
| Workflow run/attempt; all four jobs | NOT RUN |
| Installer artifact ID/retention | NOT RUN |
| Artifact ZIP bytes/SHA-256 | NOT RUN |
| Manifest bytes/SHA-256 | NOT RUN |
| Manifest executable filename/bytes/SHA-256 | NOT RUN |
| Authenticode | NOT RUN (expected `NotSigned`) |
| Schema-2 notice inventory; supplemental and bundle-resource hashes | NOT RUN |

Product identity remains `Aster Desktop`; the manifest must identify `Aster Desktop_0.1.0_x64-setup.exe`. The requested handoff `Aster_0.1.0_x64-setup.exe` may be created only as a byte-for-byte delivery alias. Record its size/SHA-256 and confirm they equal the manifest executable's values; do not rewrite the manifest or claim that the alias is a separate build.

## 2. Download and verify the unsigned installer

Download the private artifact from the exact run, retain its ZIP, and extract to a new empty evaluation directory. Do not mix manifests or binaries from other runs.

```powershell
$asterZip = Read-Host 'Full path to downloaded artifact ZIP'
$asterExe = Read-Host 'Full path to extracted manifest-named setup EXE'
$asterAlias = Read-Host 'Full path to Aster_0.1.0_x64-setup.exe delivery alias'
$asterManifest = Read-Host 'Full path to manifest.json'
Get-Command 7z.exe -ErrorAction Stop
Get-FileHash -LiteralPath $asterZip -Algorithm SHA256
Get-FileHash -LiteralPath $asterExe -Algorithm SHA256
Get-FileHash -LiteralPath $asterAlias -Algorithm SHA256
Get-Item -LiteralPath $asterExe,$asterAlias | Select-Object Name,Length
Get-AuthenticodeSignature -LiteralPath $asterExe | Select-Object Status
Get-Content -LiteralPath $asterManifest -Raw | ConvertFrom-Json
```

Compare GitHub artifact digest, manifest fields, ZIP/EXE/alias byte counts and hashes. Verify `NotSigned`. Any mismatch is FAIL and blocks installation. Also inspect the schema-2 notice inventory and require zero `requiredMissingTextCount`/`requiredUnknownLicenseCount`; inspect the `verified-extracted-nsis-payload` listing and manifest `bundle_resource_sha256` for installed-root `THIRD-PARTY-NOTICES.txt` and `LICENSE.txt`. These structural results are not legal approval.

Prefer the repository verifier before creating the alias; substitute the recorded values and a new absolute destination:

```powershell
$asterEvaluationParent = Join-Path $env:USERPROFILE 'Documents\AsterEvaluation'
New-Item -ItemType Directory -Force -Path $asterEvaluationParent | Out-Null
$asterVerifiedDestination = Join-Path $asterEvaluationParent 'verified-run'
& (Join-Path $asterSource 'scripts\qa\Test-EngineeringInstallerArtifact.ps1') `
  -ZipPath $asterZip -ExpectedZipSha256 '<64 lowercase hex>' `
  -ExpectedSourceRevision '<40 lowercase hex>' -ExpectedRunId '<digits>' `
  -ExpectedRunAttempt '<positive digits>' -Destination $asterVerifiedDestination
```

It requires exactly five archive entries (manifest, setup, notices, notice inventory and payload listing), validates schema-2 counts/provenance and supplemental manifest hashes, writes to a new directory without executing anything, independently extracts the NSIS with `7z.exe` into `verified-installer-payload`, verifies both resource hashes and reports `VERIFIED_NOT_INSTALLED`. Create the requested delivery alias only from that verified setup and confirm identical bytes.

## 3. Install, launch and preserve the credential

1. Record UTC and launch only the verified alias/setup normally. Record Windows/ESET prompts and UAC. Do not bypass a block.
2. Launch the installed Aster app normally/non-elevated. Record installed executable path/hash and empty initial session. Bind it to the installer manifest; `v0.1.0` alone is insufficient.
3. Confirm only the Boolean credential state. Do not use normal configure/change/delete controls on the existing real entry.
4. Follow [CREDENTIAL-GUI-FIXTURE-0001.md](CREDENTIAL-GUI-FIXTURE-0001.md) with its generated synthetic target and known fake value. Record native cancel/default-deny, Control Panel synthetic entry/read/remove, wrong/foreign-state denial and exact-target deletion consent. No GUI automation substitutes for human observation.
5. Inspect renderer IPC/status, source configuration and app-created log/data locations using synthetic state only. Confirm no credential bytes cross IPC or enter files/logs. Do not dump or search for the real key.

Another process under the same Windows user can read Generic Credentials; no app-exclusive custody guarantee is claimed.

## 4. UI, accessibility and ephemeral ordinary chat

1. At client viewports 1917×1137 and 1366×768, record actual window dimensions/display scale. Inspect original-reference composition, composer, sidebar and status without substituting the JPG dimensions for a rendered test.
2. Use Tab/Shift+Tab, visible focus, Ctrl+N, Ctrl+K, Escape, settings and model selection. Check names, state text, focus return, clipping and keyboard-only reachability; use Narrator where available.
3. Exercise only states reachable through the normal installed UI without modifying the owner's real credential or making extra live requests. The shipping app has no synthetic-provider or renderer-injection diagnostic: retain exact-source fixture evidence for unreachable busy/failure/markup cases and mark their installed observations NOT RUN rather than inventing an injection path.
4. The shipping provider path is fixed to NVIDIA, so there is no installed synthetic ordinary conversation. If the two authorized canaries in section 7 are executed, close/restart afterward and confirm those conversations are absent without recording their content. Otherwise mark the restart/no-history row NOT RUN rather than treating an empty first launch as proof. Inspect app-created locations without recording contents.

## 5. Protocol-2 project journey

Use a freshly created disposable non-sensitive root. Record the exact four capability Booleans and require `applyExistingFile=false`.

1. Select the root and a text file through native dialogs. The renderer may show relative path, request ID, SHA-256 and byte count only. Confirm selected plaintext is absent from renderer IPC/DOM/logs. Oversize and exact 32-KiB/+1 denial are credited only to exact-source fixtures unless they can be reached without disclosure; do not upload a real/private project to force them.
2. You may open the project-send native review with disposable non-sensitive text and cancel/close it; do not approve transmission. Record visible owner-window ownership, complete review/end marker, cancel behavior and zero user-observed continuation. This is a manual cancel-path observation, not proof of exact-body equality or zero network bytes.
3. The shipping build has no fake/loopback provider endpoint. Therefore positive exact-body dispatch, post-review drift, one-use receipt comparison, derived-answer continuation exclusion, completed-egress proposal retention and provider-backed expiry are **NOT RUN** on the installed artifact. Retain the exact-source automatic T-039 evidence; do not substitute a real project upload or either plain-text canary.
4. The normal UI has no proposal-injection diagnostic; proposals arise only from provider output. Positive new-file and existing-file proposal journeys are therefore **NOT RUN** on the installed artifact unless a separately reviewed diagnostic is added. Do not prompt a live model to manufacture test authority.
5. Independent helper-process termination/restart is exact-source build-fixture evidence, not an installed-UI feature. Retain its source/run identity separately. Physical power-loss recovery remains NOT RUN; do not terminate the installed app around a real write to simulate an unreviewed fixture.
6. Confirm the installed capability report still has `applyExistingFile=false` and the UI makes no existing-file apply claim. This does not replace the automatic proposal-only/unchanged-target fixtures or prove a positive installed proposal review.

No real/private project upload is authorized. Exact-body disclosure uses the exact-source build fixture, because no loopback seam ships in the installer; the two authorized live canaries are plain text only.

## 6. Performance observation (T-041)

Use the same installed artifact, display scale and Windows power mode. Record the measurement tool and raw values; these are engineering observations, not a universal performance guarantee.

1. Reboot only if the owner chooses to establish a normal cold baseline; otherwise label cold launch NOT RUN. Measure from launch action to usable composer for cold and three warm launches.
2. After five idle minutes, record Aster process working set/private bytes and average/peak CPU over at least 60 seconds.
3. Record keystroke/composer responsiveness, Ctrl+K/settings open/close and native-dialog open/cancel response. Note any visible frame stall, input delay or focus loss.
4. Installed cancel-to-terminal timing with synthetic loopback data is **NOT RUN** because no loopback seam ships. Do not spend an extra live request or cancel either one-shot canary to manufacture this measurement.
5. A bounded 20-turn installed synthetic conversation is **NOT RUN** for the same reason and because only two one-shot live canaries are authorized. Record start/end resource observations for non-network UI activity where useful, but do not relabel them as the T-041 multi-turn row. Restart/no-history may be observed after the authorized canaries as described above.

Retain raw observations and classify any freeze, crash, unbounded growth trend or unreachable control as FAIL. If a measurement cannot be obtained safely, mark it NOT RUN; do not invent a certification threshold after seeing results.

## 7. Exactly two authorized live canaries

Initial status: **NOT RUN**. Reserve these for the final reviewed installed artifact. Use one short non-sensitive plain-text prompt per exact model, no file/project context:

- `nvidia/nemotron-3.5-lightning-30b-a3b`
- `meta/muse-glimmer-30b`

Make exactly one streaming request to each model, two requests total. Record UTC, source/installer hash, requested and safely observable response model identity, whether partial content streamed, one terminal classification, usage if reported, and any unexpected retry/redirect/fallback. Do not record prompt/response/header/key content. Unavailability is FAIL for that attempted canary, not permission to retry or switch. Synthetic fixtures cover cancellation/errors; extra live calls remain unauthorized and NOT RUN.

## 8. Notices, decision and uninstall

Review current SBOM/inventories against the independently extracted resource evidence. Expected schema-2 counts are 575 installed source packages, 291 target-required, 45 without package-owned text, 10 peer-covered, four reviewed-overlay-covered, 31 non-required missing, zero required missing and zero required unknown. The 31 are 30 other-target/build Cargo packages plus npm dev `saxes@6.0.0`; they are not proof that those packages ship. Inspect every peer/official overlay's provenance/hash. Confirm generated notices/license hashes, review all current RustSec warnings and record the owner's scoped internal-engineering notice/risk decision. This does not grant external distribution or constitute legal advice; any incomplete review must appear in the handoff limitations.

Record an explicit unsigned-engineering decision against the exact source/setup hash and residual risks. Then close and uninstall **Aster Desktop v0.1.0 only** through Settings > Apps > Installed apps. Record removal of its app/launch entry, remaining app-created files and running processes. Do not select, modify or remove the separate `Aster 0.10.0` entry/directory, unrelated files or the production credential; uninstall is not expected to remove the credential automatically.

## Result record — complete only after execution

| Field | Value |
| --- | --- |
| UTC/tester; Windows version/build; account/token/UAC | NOT RUN |
| Display scale/power mode/ESET/protection state before/after | NOT RUN |
| Developer Mode owner baseline; no further changes | NOT RUN |
| Source/tree/ruleset/run/attempt | NOT RUN |
| ZIP/manifest/EXE/alias bytes and hashes; `NotSigned` | NOT RUN |
| Notice/license/payload inventory hashes and review | NOT RUN |
| Install/normal launch/installed EXE hash | NOT RUN |
| Synthetic credential GUI and renderer secret boundary | NOT RUN |
| Full/compact UI, keyboard, Narrator/inert rendering | NOT RUN |
| Ordinary session restart/no history | NOT RUN |
| Four protocol-2 capabilities; metadata-only selection | NOT RUN |
| Exact-body cancel/drift/one-use loopback/no continuation | NOT RUN |
| Proposal/new-file/no-clobber/process-kill/restart recovery | NOT RUN |
| Existing-file proposal-only; target unchanged | NOT RUN |
| T-041 launch/idle/responsiveness/cancel/20-turn observations | NOT RUN |
| Nemotron one-call canary | NOT RUN; authorized once |
| Muse one-call canary | NOT RUN; authorized once |
| Additional live calls | NOT RUN; not authorized |
| Uninstall/residual app files/processes; credential preserved | NOT RUN |
| FAIL/NOT RUN list and unsigned-engineering decision | NOT RUN |

For every executed row retain PASS or FAIL, exact step/command, UTC, source/environment/artifact identity and redacted evidence path. A blocked dependent row remains NOT RUN with its reason. Do not commit private evidence, credentials or raw app/project/provider data.
