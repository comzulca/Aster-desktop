# CREDENTIAL-GUI-FIXTURE-0001 — owner-run isolated native credential proof

Date: 2026-09-30 UTC; reconciled 2026-10-01 UTC. Status: the isolated diagnostic and automated T-033 boundary fixtures are retained and pass in the current local aggregate suite, but immutable-source binding and owner-installed GUI execution T-034 remain pending for the protocol-2 candidate. Trace: ADR-0005, FR-013, SEC-010, TM-013, AC-026, T-033/T-034. Run this procedure only after the exact new candidate passes CI/package review; no earlier installer certifies the changed source.

## Preconditions

Use the owner's existing Windows 11 administrator account but launch PowerShell and the installed application normally/non-elevated. Keep all Windows/ESET protections unchanged. Close ordinary Aster windows. Preserve the existing NVIDIA credential; do not use normal configure/change/delete-key controls, display its value, export Credential Manager, or enter a real key into a diagnostic. Do not send any NVIDIA request. Record exact source, successful CI run/attempt, installer hash and installed application hash, Windows build, token/elevation, UTC and display/accessibility settings.

Identify the installed **application executable**, not the NSIS setup installer. Launch one interactive diagnostic:

```powershell
$asterFixtureApp = Read-Host 'Full path to the installed Aster application EXE'
Get-FileHash -LiteralPath $asterFixtureApp -Algorithm SHA256
$asterFixtureProcess = Start-Process -FilePath $asterFixtureApp -ArgumentList '--aster-credential-gui-fixture' -Wait -PassThru
$asterFixtureProcess.ExitCode
```

Supply no extra flag, path, target, password, URL, environment override or provider selection. Valid, denied and malformed diagnostic paths terminate before normal application initialization. No process/window automation may substitute for human observation and consent.

## Exact native journey

1. The diagnostic generates and checks one unused address under `comzulca/aster-desktop/credential-gui-fixture/`. Read the complete target and fixture username displayed by the native prompt. Confirm this is a synthetic test, not the NVIDIA production address. Stop on any unexpected/pre-existing state; do not remove a foreign entry to continue.
2. In Control Panel -> Credential Manager -> Windows Credentials -> Add a generic credential, enter the exact generated address and the displayed fixture username. Use only non-secret password `synthetic-fixture-key-123`. Confirm manual GUI entry in the diagnostic; the host compares the decoded fixture internally and displays only a classification. An encoding or wrong-value error is a retained FAIL, not permission to repair/rewrite any existing credential.
3. Remove **only this generated synthetic entry** through Credential Manager. Continue only when the diagnostic confirms its absence. Never remove the NVIDIA credential or an unrelated entry.
4. Follow the native prompt cancellation stage; select Cancel/close and verify no synthetic entry was stored. At the subsequent entry stage, enter only the known fake password. The host rejects any other value and rechecks absence before an explicit fixture-only write, then verifies bounded read-back.
5. At the deletion-denial stage, inspect the full generated target and choose Cancel. Verify the synthetic entry still exists. At the separate deletion-confirmation stage, explicitly confirm deletion of that same synthetic entry; the host revalidates expected fixture metadata and its recorded generation/persistence, then verifies absence. With initial default focus left on Cancel, Enter must deny; deliberately selecting the affirmative button is a separate explicit action.
6. Record the fixed terminal classification and ExitCode: 0 = all synthetic stages completed; 2 = denied/cancelled; 3 = failed closed/unavailable/attention required. Individual intentionally tested cancellation/denial stages may be part of a successful complete journey; record observations, not merely the final numeric code.

If a prompt, stage or remaining credential is ambiguous, stop, retain FAIL/NOT RUN and the generated non-secret address. The diagnostic never blindly cleans up on error/exit. Owner cleanup may remove only the exact known synthetic entry after confirming its identity; wrong/foreign data must remain untouched for review. Generic Credential APIs provide no malicious same-user compare-and-delete guarantee.

## Evidence boundary

Capture only non-secret fixture prompts/status and necessary isolated result fields; redact unrelated credential names and avoid stored-password screenshots. Test keyboard/default-deny/labels/Narrator separately. This proves the owner's manual GUI representation and shared native primitives only. It does not prove the entire production Tauri remove handler, renderer IPC/log/file secret-boundary inspection, live NVIDIA compatibility or all release gates. API-only fixture PASS is not T-034 PASS. Existing NVIDIA custody is preserved; no value is read/exported/logged by the agent. Mark all unexecuted checks NOT RUN with their reasons.
