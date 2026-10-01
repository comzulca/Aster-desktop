# WINDOWS11-ACCEPTANCE-0001 — unsigned engineering acceptance

Created: **2026-09-30 UTC**. Initial status: **NOT RUN** for all target-Windows-11 checks. This is an exact manual procedure and result template for issue #11, not a record of completed laptop tests. The owner selected an **unsigned engineering build** for this early stage. No self-signed certificate, certificate trust change or signed production release is needed or claimed. Hosted Windows Server 2025 CI does not satisfy this procedure.

## Authority and environment

- Use the owner's **existing administrator account**. Do not create or switch to a guest/new account.
- Open PowerShell and Aster normally, **without Run as administrator**. Accept installer UAC only if the installer needs it; record whether it occurred. The app itself must be exercised non-elevated.
- On 2026-09-30 the owner reported **personally enabling Windows Developer Mode** and requested continuation. This is an explicit owner-made baseline exception, not a setting changed by the agent, a test PASS, or authority to change anything else. Record the reported change and the observed mode state separately; configuration verification and rerun results are not assumed.
- Keep all remaining Windows Security, Smart App Control/SmartScreen, ESET and other existing protections enabled and unchanged from that documented baseline. Do not add exclusions, change further policies, install a trust certificate, create/switch accounts, elevate the app, or bypass a block. If a step is blocked, retain the message, mark **FAIL** for that attempted step (dependent checks **NOT RUN**) and stop that dependent test.
- Use only disposable, non-sensitive text fixtures. Do not publish key bytes, private prompts, project contents, credential-manager exports, raw Authorization headers or unredacted diagnostic files. Record classifications, counts, hashes and paths only where non-sensitive.
- The owner confirms an existing real NVIDIA credential **must be preserved**. Never put synthetic data at its target or use normal change/delete controls for testing. Native GUI tests use only the isolated diagnostic in ADR-0005 and CREDENTIAL-GUI-FIXTURE-0001.md after it is implemented, reviewed and packaged. Exactly two minimal live canaries remain separately authorized; no extra live cancellation/error request, retry or fallback is authorized.
- Tests for issue #12 are independent. Do not enable gated file IPC or mutate real project files merely to complete release acceptance.

## 1. Bind the evidence to the exact candidate

Historical owner-context evidence is revision-specific: [LOCAL-0002.md](LOCAL-0002.md) records the initial workspace candidate and reported Developer Mode baseline; [LOCAL-0003.md](LOCAL-0003.md) records the merged PR15 credential-diagnostic runtime with 80 offline Rust fixtures. [LOCAL-0004.md](LOCAL-0004.md) tracks the separate later journal-privacy candidate and its own current status. None passes this manual procedure or permits reusing an older installer for changed behavior.

For the verified **pre-diagnostic** main reference (not an artifact of the new credential diagnostic source):

| Field | Expected baseline |
| --- | --- |
| Repository | Private `comzulca/Aster-desktop` |
| Source | `4e4cd6c6208d955782d8cbe8aea2df6aada3d8c7` |
| CI | [Run 36752590834](https://github.com/comzulca/Aster-desktop/actions/runs/36752590834), all four jobs PASS |
| ZIP | Artifact `11117506071`, SHA-256 `734ffd95656633057c556155c1b46175a9b534c97724f1e8ebae3240c42c387c` |
| EXE | `Aster Desktop_0.1.0_x64-setup.exe`, 3,540,127 bytes |
| EXE SHA-256 | `b2750db4b00b786a49b12f8c488ee746592174a6d84cd8433725d8a2b77f107e` |
| Authenticode | `NotSigned` (expected for engineering evaluation) |

After any runtime change, including ADR-0005 or issue #12 integration, **select the newly successful main run and replace every identity field in the result record with that run's source and artifact hashes**. Run 62 does not contain `--aster-credential-gui-fixture` and cannot certify it. A filename/version alone is insufficient.

In normal PowerShell, record UTC and the OS identity:

```powershell
(Get-Date).ToUniversalTime().ToString('o')
Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, BuildNumber
```

Record the ESET version and Windows protection state as observed in their normal interfaces, without exporting private configuration. Include the owner's reported Developer Mode change as a separate baseline field; do not describe all Windows settings as unchanged. Observation of the mode must not become a new configuration change. Confirm that the OS is actually Windows 11 rather than the hosted Windows Server CI environment.

Historical native-fixture evidence includes a non-elevated symlink-creation **FAIL with Windows error 1314** before the reported Developer Mode change. Preserve it unchanged. The setting report alone does not establish a PASS: LOCAL-0002 separately records the original owner's non-elevated 59-fixture rerun, and LOCAL-0003 records the later 80-fixture rerun including reparse/hardlink/symlink denials, each at its own source. Changed behavior still requires fresh source-bound results; manual UI/live acceptance remains independently gated.

In GitHub, verify the repository is private, the intended source revision is `main`, and all expected workflow jobs passed for **that revision**. Inspect the active `main` ruleset and the owner's repository role; record the ruleset name, enabled status and required checks without changing permissions. Existing historical repository-visibility evidence is not a new ruleset inspection.

On 2026-09-30 at 21:41 UTC, the requested local folder was independently verified as a genuine clean checkout of main `4e4cd6c6208d955782d8cbe8aea2df6aada3d8c7`, tree `400027f758b85e5f8118a8851b729ee5b0e4d724`; all 71 tracked file blobs matched the authenticated GitHub tree. The owner performed the directory renames after closing Codex. This is actual clone provenance, not an installer/UI PASS. Check the current state without overwriting it:

```powershell
$asterSource = Join-Path $env:USERPROFILE 'Documents\comzulca\aster-desktop'
Test-Path -LiteralPath (Join-Path $asterSource '.git')
$asterGit = 'C:\Program Files\Git\cmd\git.exe'
& $asterGit -C $asterSource remote get-url origin
& $asterGit -C $asterSource rev-parse HEAD
& $asterGit -C $asterSource status --short
Get-FileHash -LiteralPath (Join-Path $asterSource 'public\assets\Aster_UI2.jpg') -Algorithm SHA256
```

Expected JPG SHA-256: `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887`. Origin must be `https://github.com/comzulca/Aster-desktop.git`, `.git` must exist, and current source/status must be recorded. No global safe.directory exception is needed when checks run under the owner rather than the isolated technical sandbox. The owner expressly declined another backup: after verifying both trees and absence of unique files/reparse entries, only the 71-file duplicate `aster-desktop.snapshot-temporal` was removed at 21:43:21 UTC. Active checkout and historical backups were untouched; no security/ACL change was made.

At `2026-09-30T22:49:06.1532939Z`, authenticated owner non-elevated checks verified the same checkout clean on main `50837e157e009b5d1135fbe8fa18d29e9fe2975a`, tree `1799e08bb3313a8417ad78d2d692325e3d9407d5`, after PR15 passed required run63 checks and was merged. Main run64 started packaging this intermediate credential-diagnostic source. Later journal-privacy/runtime candidates invalidate dependent package evidence; reserve the two live canaries for the final selected reviewed artifact, not each intermediate build. No installer or owner GUI acceptance is inferred from this clone/CI observation.

On 2026-10-01 UTC, all four main run64 jobs were observed PASS and its intermediate installer was independently verified without execution: artifact `11131148289`, ZIP SHA256 `dba8fd3f16dde4e460e872310bd33d540aec2d74a2d859b8830f84b751c0e268`, EXE 3,551,847 bytes / SHA256 `752928d145ca7a7e78150266a04c0bf9c8fd057efcbf544a56f1fe9d866348c7`, Authenticode `NotSigned`, verified `2026-10-01T09:34:37.7459336Z`. Classification is VERIFIED_NOT_INSTALLED. It includes the PR15 diagnostic but **not** the later journal-privacy change; it is not a final artifact for that changed source and consumes no live acceptance budget.

## 2. Download and check the unsigned installer

Download the installer artifact from the exact successful private Actions run. Check that it is still available (14-day retention). Keep its ZIP intact and extract into a new empty evaluation folder. Do not replace an older candidate or mix `manifest.json` files from different runs.

Use literal paths to the actual downloaded files in normal PowerShell:

```powershell
$asterZip = Read-Host 'Full path to the downloaded installer artifact ZIP'
$asterExe = Read-Host 'Full path to the extracted Aster setup EXE'
$asterManifest = Read-Host 'Full path to the extracted manifest.json'
Get-FileHash -LiteralPath $asterZip -Algorithm SHA256
Get-FileHash -LiteralPath $asterExe -Algorithm SHA256
Get-Item -LiteralPath $asterExe | Select-Object Name, Length
Get-AuthenticodeSignature -LiteralPath $asterExe | Select-Object Status
Get-Content -LiteralPath $asterManifest -Raw | ConvertFrom-Json |
    Select-Object classification, source_revision, workflow_run_id,
        workflow_run_attempt, filename, byte_count, sha256, authenticode_status,
        built_at_utc
```

Compare ZIP digest with GitHub's artifact digest; compare EXE digest/length and every manifest identity with the selected run. The reference values above apply only to source `4e4cd6c...` before the new diagnostic. Expected signature status is `NotSigned`; do not treat it as proof of public trust or production readiness. Any mismatch is **FAIL** and blocks install. Keep hashes and the non-secret manifest as evidence.

## 3. Install, launch and uninstall under the documented owner baseline

1. Record UTC before starting and confirm the installer hash just checked is the executable being launched.
2. Launch the installer normally. Record any Windows/ESET prompt and whether UAC was required. If protection blocks it, record the block and stop; do not disable protection or bypass it under this procedure.
3. If installation succeeds, launch Aster normally from the installed entry. Record its version and installed executable path/hash. Verify the expected empty session and honest credential/project status. Do not infer the source hash from the displayed `v0.1.0` alone; retain the installer manifest binding.
4. Perform sections 4–6 while that exact candidate is installed. The existing real credential must remain unchanged throughout install, diagnostics and uninstall; no cleanup decision may silently override the owner's preservation requirement.
5. Close Aster and uninstall using Windows **Settings > Apps > Installed apps**. Record successful removal of the installed app entry and launch entry, any remaining app files, any running Aster process, and whether UAC occurred. Do not remove unrelated files or assume uninstall deletes Credential Manager secrets.

Record install, normal launch and uninstall independently. A blocked install leaves runtime checks **NOT RUN**, not PASS. The resulting artifact remains an unsigned engineering candidate even if these checks pass.

## 4. Native credentials and secret-boundary checks

Related requirements: FR-011, FR-013 / SEC-001, SEC-002, SEC-010; AC-013, AC-020, AC-026; T-015, T-022, T-028, T-033, T-034.

The owner already has a real NVIDIA credential and expressly requires preservation. Do not overwrite it with a fake key, use normal configure/change/delete controls, display its password, export credentials or re-enter the real key for this procedure. Ordinary configured-state observation is a Boolean, not a secret-read proof.

1. Obtain the **new** reviewed diagnostic candidate, not run 62. Follow [CREDENTIAL-GUI-FIXTURE-0001.md](CREDENTIAL-GUI-FIXTURE-0001.md) under the existing owner account/non-elevated token. It generates one distinct synthetic target, asks for manual Control Panel fake entry/read/remove, then exercises shared native prompt cancel/entry/read and separate default-cancel deletion denial/confirmation. Supply no target/key override. No production credential operation, network or normal host initialization occurs in that diagnostic.
2. Record actual GUI representation and native observations separately from the T-033 fake-backend tests and T-028 API/decoder fixtures. No automated GUI consent is acceptance. Wrong/malformed/pre-existing/foreign state fails closed; there is no blind cleanup on error or exit. If blocked by Windows Security, retain the block and mark dependent GUI checks NOT RUN without changing protections.
3. From reviewed source with locked dependencies and fixture-only execution, the existing disposable API round-trip test may be run separately. It uses a unique synthetic target, not the production credential. Record source, command, UTC, environment and sanitized summary:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --locked --offline synthetic_credential_round_trip_uses_a_disposable_target
```

4. Independently inspect bounded renderer IPC/status, source/configuration and app-created file/log locations using only synthetic fixtures. Do not enable raw request/header/body logging, dump Credential Manager or search/export the real key. Record inspected locations/results without private contents. If capture cannot safely be completed, mark it NOT RUN. A standalone native diagnostic does not prove the complete production Tauri deletion handler or renderer capture boundary.

Retain the existing real credential unchanged for the two separately authorized final plain-text canaries. Another process under the same Windows user can access Generic Credentials; no app-exclusive custody guarantee is claimed. Removing the isolated fixture is not authorization to remove the production credential.

## 5. Rendered UI, honest scope and ephemeral session

Related requirements: UX-001, UX-002 / SEC-008; AC-006, AC-007, AC-012, AC-015, AC-016; T-007, T-008, T-014, T-017, T-018.

1. At client/window viewports **1917 x 1137** and **1366 x 768**, inspect the original-reference composition, composer, connection status and sidebar. Record actual measured viewport/window dimensions and display scale; do not substitute the JPG size for a rendered test. Capture only non-sensitive empty/synthetic UI evidence.
2. Use keyboard only: Tab/Shift+Tab, visible focus, **Ctrl+N**, **Ctrl+K**, settings open/close, Escape, and model selection. Check named controls, readable state text and focus return after closing settings. Record any clipping, unreachable control or incorrect focus order as FAIL.
3. Select only a disposable non-sensitive root using the native folder dialog. At the baseline, attachment, proposal, disclosure and apply remain unavailable: root selection must not silently enable content access or transmit a file. A newer candidate must match its separately proved capability flags and issue #12 evidence; absence of such proof cannot be papered over by enabling controls.
4. Verify absent-key, configured-but-connection-unverified, busy, cancel and failure messages are truthful when their conditions can be exercised. A configured credential Boolean is not verified NVIDIA connectivity. Test injected markup as **inert text** with a synthetic/fake-host fixture; do not upload a private project or invent a live response to produce that state.
5. Exercise a synthetic session using the fixture/fake-host path if available; otherwise a non-sensitive message is contingent on section 6 authorization. Close and restart Aster. Expect an empty conversation and no recovered chat text. Record inspected app-created file locations and restart result without exporting private content. If no safe message path is available, mark the restart-after-message test NOT RUN rather than calling an empty first launch a persistence test.

## 6. Separately authorized NVIDIA canaries

Related requirements: FR-001–FR-003 / SEC-003, SEC-007; AC-003–AC-005, AC-021, AC-025; T-003–T-006, T-023, T-027.

**Initial status: NOT RUN.** The owner has separately authorized **exactly two minimal live canaries: one request per exact model below**. Authorization is not execution evidence. The existing real key remains in Windows Credential Manager and must be preserved; no re-entry/export is required. Reserve these calls for the final reviewed exact candidate. Use a short non-sensitive synthetic prompt, include no files, and keep each request within fixed ceilings. Do not log credentials or raw private payloads. There is no authority for retries, a third request, live cancellation/error cases, fallback or another model.

For **each model independently**, select its exact registered ID:

- `nvidia/nemotron-3.5-lightning-30b-a3b`
- `meta/muse-glimmer-30b`

Make one minimal completion/streaming request for each model, **two requests total**. For each request record UTC, source and installer hash, exact requested model ID, observed request/response model identity where safely observable, whether partial content streamed, one terminal result, and whether any unexpected retry, redirect or provider/model fallback occurred. Do not log response/prompt content or headers. If model identity cannot be independently observed, record that limit; the selector label alone is not proof. Cover cancellation and malformed/error transport with synthetic fixtures, recorded separately; any additional live cancellation/error checks remain **NOT RUN** unless separately authorized. Never label fixture coverage a live result.

An unavailable/denied model is a retained **FAIL** for that model's attempted canary, not a reason to silently switch models. Tests not attempted are **NOT RUN**. A response from one model cannot certify the other. Live behavior/timing and generated prose are not deterministic guarantees.

## 7. Dependency, notices and decision

Related requirement: SEC-009 / NFR-003; AC-019; T-021.

Review the selected run's inventories/SBOM separately from the actual installer payload. Run 62 has eight allowed warnings: five Windows-compiled unmaintained `unic-*`, two Linux-only warnings and Windows compile-time yanked `yoke-derive 0.8.3`. Audit exit 0 is not risk acceptance. The owner LICENSE is now in main; it does not clear third-party payload obligations. The older [DEPENDENCIES-0001.md](DEPENDENCIES-0001.md) records issues for its own revision, not approval of a new binary. Record current upstream-license/notice decisions, exact redistributed files and any owner's scoped exception. If incomplete, retain NOT RUN and the release limitation.

Record an explicit **unsigned-engineering evaluation decision** against the exact source/EXE hash, scope and residual risks. Do not mark signed/public-production promotion PASS or close issue #11 merely because signing was deliberately excluded. Actual Windows 11 and separately scoped acceptance evidence must exist; any issue-body adjustment to the engineering-only scope must preserve the remaining checks and the owner's dated choice.

## Result record — complete only after execution

Copy this template into dated private evidence; do not replace the initial NOT RUN labels with PASS until the exact step was actually performed.

| Field | Value |
| --- | --- |
| UTC start/end / tester | NOT RUN |
| Existing account / non-elevated app / UAC observation | NOT RUN |
| Windows 11 caption/version/build / display scale / ESET version | NOT RUN |
| Developer Mode baseline exception | Owner reports personally enabled on 2026-09-30; not changed by agent; independently source-bound observation/rerun evidence is recorded separately |
| Remaining protection state before/after; further changes | NOT RUN (expected: unchanged; no further changes authorized) |
| Historical non-elevated symlink fixture | FAIL, Windows error 1314; retained separately from any new rerun |
| Source commit / true clone or snapshot / remote / ruleset | NOT RUN |
| Workflow run/attempt / manifest / ZIP hash / EXE hash and bytes | NOT RUN |
| Install / normal launch / uninstall | NOT RUN |
| Native cancel / synthetic entry / denial / deletion / disposable read round trip | NOT RUN |
| Renderer/IPC/log/file secret-boundary inspection locations/results | NOT RUN |
| Full/compact viewport / keyboard / labels / inert rendering | NOT RUN |
| Root selection / honest independently gated capabilities | NOT RUN |
| Restart after synthetic message / no chat persistence | NOT RUN |
| Live authority/budget | Exactly two minimal requests authorized, one per exact model; execution NOT RUN |
| Each exact model's single completion/stream result / cancellation/error fixture result | NOT RUN |
| Additional live cancellation/error requests | NOT RUN; not authorized |
| Current license/payload/notices / warning review / scoped owner decisions | NOT RUN |
| Evidence filenames/hashes / retained failures / outstanding NOT RUN items | NOT RUN |
| Unsigned engineering decision / residual risks / production not claimed | NOT RUN |

For each executed row retain: **PASS or FAIL**, exact manual step/command, UTC, source, environment, artifact identity and evidence path. A blocked dependent row remains **NOT RUN** with its reason. Do not commit keys, private prompts, raw app data or unredacted reports.
