# WINDOWS11-ACCEPTANCE-0001 — unsigned engineering acceptance

Created: **2026-09-30 UTC**. Initial status: **NOT RUN** for all target-Windows-11 checks. This is an exact manual procedure and result template for issue #11, not a record of completed laptop tests. The owner selected an **unsigned engineering build** for this early stage. No self-signed certificate, certificate trust change or signed production release is needed or claimed. Hosted Windows Server 2025 CI does not satisfy this procedure.

## Authority and environment

- Use the owner's **existing administrator account**. Do not create or switch to a guest/new account.
- Open PowerShell and Aster normally, **without Run as administrator**. Accept installer UAC only if the installer needs it; record whether it occurred. The app itself must be exercised non-elevated.
- On 2026-09-30 the owner reported **personally enabling Windows Developer Mode** and requested continuation. This is an explicit owner-made baseline exception, not a setting changed by the agent, a test PASS, or authority to change anything else. Record the reported change and the observed mode state separately; configuration verification and rerun results are not assumed.
- Keep all remaining Windows Security, Smart App Control/SmartScreen, ESET and other existing protections enabled and unchanged from that documented baseline. Do not add exclusions, change further policies, install a trust certificate, create/switch accounts, elevate the app, or bypass a block. If a step is blocked, retain the message, mark **FAIL** for that attempted step (dependent checks **NOT RUN**) and stop that dependent test.
- Use only disposable, non-sensitive text fixtures. Do not publish key bytes, private prompts, project contents, credential-manager exports, raw Authorization headers or unredacted diagnostic files. Record classifications, counts, hashes and paths only where non-sensitive.
- Native credential tests may alter only Aster's fixed credential after confirming it is not an existing real credential. Do not overwrite/delete an existing real key for a synthetic test. The owner will enter the real key directly in Windows Credential Manager and has authorized exactly two minimal live canaries, one per model; this is not authorization for extra live cancellation/error requests, retries or fallback. Agree on a cleanup decision without revealing the key.
- Tests for issue #12 are independent. Do not enable gated file IPC or mutate real project files merely to complete release acceptance.

## 1. Bind the evidence to the exact candidate

The newer source candidate's offline owner-context results are [LOCAL-0002.md](LOCAL-0002.md); they do not pass this manual procedure or reuse the older installer below. Developer Mode's explicit owner change is included in that evidence.

For the currently verified baseline:

| Field | Expected baseline |
| --- | --- |
| Repository | Private `comzulca/Aster-desktop` |
| Source | `31a82bf50fc46ea4fbb91c50c59cc02a65da4c71` |
| CI | [Run 36190804857](https://github.com/comzulca/Aster-desktop/actions/runs/36190804857), all four jobs PASS |
| ZIP | Artifact `10888958678`, SHA-256 `c6e7d67a50d438f5b2979c6a33e5fbf69ae3a20ae01e968a79baec3893880d9f` |
| EXE | `Aster Desktop_0.1.0_x64-setup.exe`, 3,527,015 bytes |
| EXE SHA-256 | `8175407e55838319b3553c9d6e577b5858a67bc0413903aa7178d2522f5df19e` |
| Authenticode | `NotSigned` (expected for engineering evaluation) |

If issue #12 changes runtime behavior, **select the newly successful main run and replace every identity field in the result record with that run's source and artifact hashes**. Never use the baseline binary to certify new code. A filename/version alone is insufficient.

In normal PowerShell, record UTC and the OS identity:

```powershell
(Get-Date).ToUniversalTime().ToString('o')
Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, BuildNumber
```

Record the ESET version and Windows protection state as observed in their normal interfaces, without exporting private configuration. Include the owner's reported Developer Mode change as a separate baseline field; do not describe all Windows settings as unchanged. Observation of the mode must not become a new configuration change. Confirm that the OS is actually Windows 11 rather than the hosted Windows Server CI environment.

Historical native-fixture evidence includes a non-elevated symlink-creation **FAIL with Windows error 1314** before the reported Developer Mode change. Preserve it unchanged. The setting report alone does not establish a new PASS: record any rerun separately with exact source, Windows build, existing-account/token, command, UTC and evidence. A source-bound rerun result has not yet been appended to this document; manual UI/live acceptance remains independently gated.

In GitHub, verify the repository is private, the intended source revision is `main`, and all expected workflow jobs passed for **that revision**. Inspect the active `main` ruleset and the owner's repository role; record the ruleset name, enabled status and required checks without changing permissions. Existing historical repository-visibility evidence is not a new ruleset inspection.

The requested local folder currently contains a verified source snapshot, not `.git` metadata. Check it without overwriting it:

```powershell
$asterSource = Join-Path $env:USERPROFILE 'Documents\comzulca\aster-desktop'
Test-Path -LiteralPath (Join-Path $asterSource '.git')
Get-FileHash -LiteralPath (Join-Path $asterSource 'public\assets\Aster_UI2.jpg') -Algorithm SHA256
```

Expected JPG SHA-256: `b95163a411529b8608461dda4faeacab3f09c6cde090c3b1d72092bf0a7c4887`. A `False` `.git` result is consistent with a snapshot; it is not a failed source hash and must not be recorded as a successful clone. If a true clone is required at this same location, arrange a separately reviewed preservation/migration of the existing snapshot first. After an actual clone exists, record `git remote -v`, `git rev-parse HEAD`, and `git status --short`; do not fabricate those outputs for the snapshot.

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

Compare ZIP digest with GitHub's artifact digest; compare EXE digest/length and every manifest identity with the selected run. The baseline values above apply only to baseline source `31a82bf...`. Expected signature status is `NotSigned`; do not treat it as proof of public trust or production readiness. Any mismatch is **FAIL** and blocks install. Keep hashes and the non-secret manifest as evidence.

## 3. Install, launch and uninstall under the documented owner baseline

1. Record UTC before starting and confirm the installer hash just checked is the executable being launched.
2. Launch the installer normally. Record any Windows/ESET prompt and whether UAC was required. If protection blocks it, record the block and stop; do not disable protection or bypass it under this procedure.
3. If installation succeeds, launch Aster normally from the installed entry. Record its version and installed executable path/hash. Verify the expected empty session and honest credential/project status. Do not infer the source hash from the displayed `v0.1.0` alone; retain the installer manifest binding.
4. Perform sections 4–6 while that exact candidate is installed. If a real credential is added later, agree with the owner whether to retain or explicitly remove it; never delete it silently as part of uninstall.
5. Close Aster and uninstall using Windows **Settings > Apps > Installed apps**. Record successful removal of the installed app entry and launch entry, any remaining app files, any running Aster process, and whether UAC occurred. Do not remove unrelated files or assume uninstall deletes Credential Manager secrets.

Record install, normal launch and uninstall independently. A blocked install leaves runtime checks **NOT RUN**, not PASS. The resulting artifact remains an unsigned engineering candidate even if these checks pass.

## 4. Native credentials and secret-boundary checks

Related requirements: FR-011 / SEC-001, SEC-002; AC-013, AC-020; T-015, T-022.

1. Inspect Aster's initial status and the fixed target in Windows Credential Manager. If a credential already exists, stop the synthetic fixed-target test and ask the owner to choose a safe plan; never overwrite the real credential. The baseline host at `31a82bf...` expects UTF-8 credential blobs. Manual Windows Credential Manager entry may store UTF-16LE; therefore use the newly reviewed, tested compatible candidate for that path, not an unsupported older binary. Encoding compatibility must have its own source/tests/evidence.
2. With no existing Aster key, open Windows **Control Panel > Credential Manager > Windows Credentials > Add a generic credential** in the existing account. Set the Internet/network address to exactly `comzulca/aster-desktop/nvidia/integrate.api.nvidia.com`, username to `nvidia-api`, and password to deliberately fake non-secret `synthetic-fixture-key-123`. Keep this within the native Windows interface. Do not send a chat request using this fake value.
3. Start/restart the compatible candidate normally. Verify a truthful configured state and that only a Boolean crosses renderer IPC, never token bytes. Presence alone is not a byte-for-byte host-read proof. Retain independent host decoding/credential round-trip test evidence. Do not reveal the stored value or export the credential.
4. Select **Eliminar clave** in Aster, reject the native deletion confirmation once, and verify the credential remains configured. Repeat and accept the exact deletion; verify it is unconfigured and only the fixed Aster target was removed. Separately open **Ajustes > Configurar clave en Windows** and cancel the native Aster/NVIDIA entry dialog; verify that cancellation does not create a credential. If the app-native entry route is also tested, use the same synthetic non-secret value and explicitly delete it afterwards; do not replace a real credential.
5. The interactive steps prove manual/native entry, state and deletion, not by themselves the host's byte-for-byte read round trip. From a reviewed genuine checkout, with locked dependencies and fixture-only execution, run the disposable-target test below on Windows 11 and the new manual-entry decoding fixtures for the selected candidate. The round-trip test creates a uniquely named synthetic target and cleans up that target, not a production credential. Record exact source, command, UTC, result and sanitized test summary. If local compilation is blocked by Windows Security, do not change security; mark this target check **NOT RUN** and retain the separate hosted-Server PASS.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --locked synthetic_credential_round_trip_uses_a_disposable_target
```

6. Independently inspect bounded renderer IPC/status, source/configuration and the app-created file/log locations for the **synthetic** fixture value. Do not enable request/header/body logging, dump Credential Manager, or search/export an actual key. Record the inspected locations and whether the fixture was found without publishing private file contents. Source review and synthetic observations are separate from a real-key assurance claim. If inspection cannot be completed safely, mark it **NOT RUN**.

After synthetic cleanup and compatibility verification, the **owner alone** enters the real NVIDIA key directly in Windows Credential Manager at that same fixed target and username, using the Windows native interface. Never send it through chat, WebView inputs, a shell command, an environment file or CI. Record only success/failure and configured state. Another process running as this same Windows user may access generic credentials; this test does not prove isolation against that user.

## 5. Rendered UI, honest scope and ephemeral session

Related requirements: UX-001, UX-002 / SEC-008; AC-006, AC-007, AC-012, AC-015, AC-016; T-007, T-008, T-014, T-017, T-018.

1. At client/window viewports **1917 x 1137** and **1366 x 768**, inspect the original-reference composition, composer, connection status and sidebar. Record actual measured viewport/window dimensions and display scale; do not substitute the JPG size for a rendered test. Capture only non-sensitive empty/synthetic UI evidence.
2. Use keyboard only: Tab/Shift+Tab, visible focus, **Ctrl+N**, **Ctrl+K**, settings open/close, Escape, and model selection. Check named controls, readable state text and focus return after closing settings. Record any clipping, unreachable control or incorrect focus order as FAIL.
3. Select only a disposable non-sensitive root using the native folder dialog. At the baseline, attachment, proposal, disclosure and apply remain unavailable: root selection must not silently enable content access or transmit a file. A newer candidate must match its separately proved capability flags and issue #12 evidence; absence of such proof cannot be papered over by enabling controls.
4. Verify absent-key, configured-but-connection-unverified, busy, cancel and failure messages are truthful when their conditions can be exercised. A configured credential Boolean is not verified NVIDIA connectivity. Test injected markup as **inert text** with a synthetic/fake-host fixture; do not upload a private project or invent a live response to produce that state.
5. Exercise a synthetic session using the fixture/fake-host path if available; otherwise a non-sensitive message is contingent on section 6 authorization. Close and restart Aster. Expect an empty conversation and no recovered chat text. Record inspected app-created file locations and restart result without exporting private content. If no safe message path is available, mark the restart-after-message test NOT RUN rather than calling an empty first launch a persistence test.

## 6. Separately authorized NVIDIA canaries

Related requirements: FR-001–FR-003 / SEC-003, SEC-007; AC-003–AC-005, AC-021, AC-025; T-003–T-006, T-023, T-027.

**Initial status: NOT RUN.** The owner has separately authorized **exactly two minimal live canaries: one request per exact model below**. The unsigned-build choice alone did not provide that authority, and authorization is not execution evidence. The owner enters the key directly in the Windows Credential Manager native interface as described above. Use a short non-sensitive synthetic prompt, include no files, and keep each request within the app's fixed ceilings. Do not log credentials or raw private payloads. There is no authority for automatic/manual retries, a third request, live cancellation/error cases, fallback, or a different model.

For **each model independently**, select its exact registered ID:

- `nvidia/nemotron-3.5-lightning-30b-a3b`
- `meta/muse-glimmer-30b`

Make one minimal completion/streaming request for each model, **two requests total**. For each request record UTC, source and installer hash, exact requested model ID, observed request/response model identity where safely observable, whether partial content streamed, one terminal result, and whether any unexpected retry, redirect or provider/model fallback occurred. Do not log response/prompt content or headers. If model identity cannot be independently observed, record that limit; the selector label alone is not proof. Cover cancellation and malformed/error transport with synthetic fixtures, recorded separately; any additional live cancellation/error checks remain **NOT RUN** unless separately authorized. Never label fixture coverage a live result.

An unavailable/denied model is a retained **FAIL** for that model's attempted canary, not a reason to silently switch models. Tests not attempted are **NOT RUN**. A response from one model cannot certify the other. Live behavior/timing and generated prose are not deterministic guarantees.

## 7. Dependency, notices and decision

Related requirement: SEC-009 / NFR-003; AC-019; T-021.

Review the selected run's frontend and Rust license inventories/SBOM separately from the actual installer payload. The baseline audit has seven allowed warnings, including five Windows-compiled unmaintained `unic-*` packages and two Linux-only warnings; audit exit 0 does not eliminate them. The older [DEPENDENCIES-0001.md](DEPENDENCIES-0001.md) names local `UNKNOWN`, MPL and `r-efi` license issues for its own revision, not a clearance for the new binary. Record current upstream-license/notice decisions, exact redistributed files and any owner's scoped exception, without inventing legal approval. If these decisions are not made, mark **NOT RUN** and state the release limitation.

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
