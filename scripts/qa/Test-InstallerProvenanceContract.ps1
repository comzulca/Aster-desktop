#requires -Version 5.1
# Pure synthetic fixtures; never execute the passive verifier or native helpers.
[CmdletBinding()]
param()
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$verifierPath = Join-Path $PSScriptRoot 'Test-EngineeringInstallerArtifact.ps1'
$tokens = $null
$parseErrors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($verifierPath, [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count -ne 0) { throw 'Passive verifier syntax is invalid.' }
$definitions = @($ast.FindAll({ param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'Assert-InstallerProvenance' }, $true))
if ($definitions.Count -ne 1) { throw 'Expected one exact provenance helper.' }
# Only this reviewed pure function definition is evaluated, not the script body.
. ([scriptblock]::Create($definitions[0].Extent.Text))
$revision = 'a' * 40
$run = '123456'
$attempt = '1'
$caseCount = 0
function New-SyntheticManifest([bool]$Diagnostic) {
    return [pscustomobject]@{
        classification=$(if ($Diagnostic) { 'unsigned-engineering-candidate' } else { 'unsigned-engineering-build' })
        workflow_ref=$(if ($Diagnostic) { 'refs/pull/41/merge' } else { 'refs/heads/main' })
        workflow_event=$(if ($Diagnostic) { 'pull_request' } else { 'push' })
        source_revision=$revision
        workflow_run_id=$run
        workflow_run_attempt=$attempt
    }
}
function Assert-Denied($Manifest, [string]$Source, [string]$Run, [string]$Attempt, [string]$Pr) {
    $denied = $false
    try { $null = Assert-InstallerProvenance $Manifest $Source $Run $Attempt $Pr } catch { $denied = $true }
    if (-not $denied) { throw 'Synthetic forged provenance was admitted.' }
}
foreach ($event in @('push', 'workflow_dispatch')) {
    $manifest = New-SyntheticManifest $false
    $manifest.workflow_event = $event
    if ((Assert-InstallerProvenance $manifest $revision $run $attempt '') -cne 'Main') { throw 'Valid main provenance did not retain main-only mode.' }
    $caseCount++
}
if ((Assert-InstallerProvenance (New-SyntheticManifest $true) $revision $run $attempt '41') -cne 'DiagnosticPullRequest') { throw 'Exact PR diagnostic provenance was denied.' }
$caseCount++
Assert-Denied (New-SyntheticManifest $true) $revision $run $attempt ''
Assert-Denied (New-SyntheticManifest $false) $revision $run $attempt '41'
$caseCount += 2
foreach ($diagnostic in @($false, $true)) {
    $pr = $(if ($diagnostic) { '41' } else { '' })
    foreach ($change in @('classification', 'workflow_ref', 'workflow_event', 'source_revision', 'workflow_run_id', 'workflow_run_attempt')) {
        $manifest = New-SyntheticManifest $diagnostic
        $manifest.$change = 'wrong'
        Assert-Denied $manifest $revision $run $attempt $pr
        $caseCount++
        foreach ($shape in @('null', 'true', 'false', '1', '1.5', '[]', '["{expected}"]', '["{expected}","wrong"]', '{"value":"{expected}"}', '[["{expected}"]]')) {
            $manifest = New-SyntheticManifest $diagnostic
            $expected = $manifest.$change
            # Assign the property from a parsed envelope so empty arrays/null are
            # not silently enumerated or string-coerced by the fixture itself.
            $envelope = ('{"value":' + $shape.Replace('{expected}', $expected) + '}') | ConvertFrom-Json
            $manifest.$change = $envelope.value
            Assert-Denied $manifest $revision $run $attempt $pr
            $caseCount++
        }
        $manifest = New-SyntheticManifest $diagnostic
        $manifest.PSObject.Properties.Remove($change)
        Assert-Denied $manifest $revision $run $attempt $pr
        $caseCount++
    }
    foreach ($source in @(('a' * 39), ('A' * 40), ('b' * 40))) {
        Assert-Denied (New-SyntheticManifest $diagnostic) $source $run $attempt $pr
        $caseCount++
    }
    foreach ($badRun in @('0', '0123456', '123457')) {
        Assert-Denied (New-SyntheticManifest $diagnostic) $revision $badRun $attempt $pr
        $caseCount++
    }
    foreach ($badAttempt in @('0', '01', '2')) {
        Assert-Denied (New-SyntheticManifest $diagnostic) $revision $run $badAttempt $pr
        $caseCount++
    }
}
foreach ($badPr in @('0', '041', '42', '-41', '41/merge', ('1' * 21))) {
    Assert-Denied (New-SyntheticManifest $true) $revision $run $attempt $badPr
    $caseCount++
}
foreach ($badRef in @('refs/pull/41/head', 'refs/pull/42/merge', 'refs/heads/main', 'refs/pull/41/merge/extra')) {
    $manifest = New-SyntheticManifest $true
    $manifest.workflow_ref = $badRef
    Assert-Denied $manifest $revision $run $attempt '41'
    $caseCount++
}
[pscustomobject][ordered]@{
    classification='pure-synthetic-provenance-fixtures-not-artifact-verification'
    status='PASS'
    cases=$caseCount
    native_executed=$false
    whole_verifier_executed=$false
    final_main_mode_preserved=$true
    verifier_sha256=(Get-FileHash -LiteralPath $verifierPath -Algorithm SHA256).Hash.ToLowerInvariant()
    checked_at_utc=(Get-Date).ToUniversalTime().ToString('o')
} | ConvertTo-Json
