#Requires -Version 5.1
[CmdletBinding()]
param()

# Executes only three reviewed pure resource-validator function definitions from
# the passive verifier. Never runs its top-level archive/native/materialisation
# code, 7z, Authenticode, an application, installer, network or credential route.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$sourceRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$verifierPath = Join-Path $sourceRoot 'scripts/qa/Test-EngineeringInstallerArtifact.ps1'
$workflowPath = Join-Path $sourceRoot '.github/workflows/ci.yml'
$tokens = $null
$parseErrors = $null
$verifierAst = [System.Management.Automation.Language.Parser]::ParseFile($verifierPath, [ref]$tokens, [ref]$parseErrors)
if (@($parseErrors).Count -ne 0) { throw 'Passive verifier has PowerShell parse errors.' }
function Get-ReviewedFunction($Ast, [string]$Name) {
    $definitions = @($Ast.FindAll({ param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq $Name }, $true))
    if ($definitions.Count -ne 1) { throw 'Expected one exact reviewed pure resource function.' }
    return $definitions[0].Extent.Text
}
foreach ($name in @('Get-AsterBuilderResourceInventory', 'Assert-RequiredResourceMembers', 'Assert-RequiredResourceEvidence')) {
    . ([scriptblock]::Create((Get-ReviewedFunction $verifierAst $name)))
}
$workflowLines = [System.IO.File]::ReadAllLines($workflowPath)
$packageStart = [Array]::FindIndex($workflowLines, [Predicate[string]]{ param($line) $line -ceq '          $installer = $candidates[0]' })
$packageEnd = [Array]::FindIndex($workflowLines, [Predicate[string]]{ param($line) $line -ceq '          Get-Content -LiteralPath reports/engineering-installer/manifest.json' })
if ($packageStart -lt 0 -or $packageEnd -le $packageStart) { throw 'Expected one bounded installer packaging block.' }
$packageCode = ($workflowLines[$packageStart..$packageEnd] | ForEach-Object { $_.Substring(10) }) -join "`n"
$workflowTokens = $null
$workflowErrors = $null
$workflowAst = [System.Management.Automation.Language.Parser]::ParseInput($packageCode, [ref]$workflowTokens, [ref]$workflowErrors)
if (@($workflowErrors).Count -ne 0) { throw 'CI packaging block has PowerShell parse errors.' }
$ciGetter = Get-ReviewedFunction $workflowAst 'Get-AsterBuilderResourceInventory'
$ciInventory = & ([scriptblock]::Create($ciGetter + "`nGet-AsterBuilderResourceInventory"))
$fixedInventory = Get-AsterBuilderResourceInventory
if (($ciInventory | ConvertTo-Json -Depth 4 -Compress) -cne ($fixedInventory | ConvertTo-Json -Depth 4 -Compress)) { throw 'CI and independent passive inventory differ.' }
$ciMembers = (Get-ReviewedFunction $workflowAst 'Assert-RequiredResourceMembers') -replace '\s+', ' '
$passiveMembers = (Get-ReviewedFunction $verifierAst 'Assert-RequiredResourceMembers') -replace '\s+', ' '
if ($ciMembers -cne $passiveMembers) { throw 'CI and passive member validator definitions differ.' }
$expectedMappings = [ordered]@{ '../reports/THIRD-PARTY-NOTICES.txt'='THIRD-PARTY-NOTICES.txt'; '../LICENSE'='LICENSE.txt' }
foreach ($name in $fixedInventory.Keys) {
    $expectedMappings['resources/' + $name] = $name
    $file = Get-Item -LiteralPath (Join-Path $sourceRoot ('src-tauri/resources/' + $name)) -Force
    if ($file.PSIsContainer -or ($file.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0 -or
        $file.Length -ne $fixedInventory[$name].byte_count -or
        (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant() -cne $fixedInventory[$name].sha256) { throw 'Original inert source bytes differ from the fixed inventory.' }
}
$configMappings = @((Get-Content -LiteralPath (Join-Path $sourceRoot 'src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json).bundle.resources.PSObject.Properties)
if ($configMappings.Count -ne 12) { throw 'Expected exactly twelve config mappings.' }
foreach ($mapping in $configMappings) {
    if (@($expectedMappings.Keys) -cnotcontains $mapping.Name -or [string]$mapping.Value -cne [string]$expectedMappings[$mapping.Name]) { throw 'Config mapping differs from the reviewed exact source/destination pair.' }
}
function New-SyntheticResourceFixture {
    $hashes = [ordered]@{ 'THIRD-PARTY-NOTICES.txt'=('a' * 64); 'LICENSE.txt'=('b' * 64) }
    $sizes = [ordered]@{ 'THIRD-PARTY-NOTICES.txt'=11; 'LICENSE.txt'=13 }
    foreach ($name in $fixedInventory.Keys) { $hashes[$name]=$fixedInventory[$name].sha256; $sizes[$name]=[int]$fixedInventory[$name].byte_count }
    $manifest = [pscustomobject]@{ bundle_resource_sha256=[pscustomobject]$hashes; bundle_resource_byte_count=[pscustomobject]$sizes }
    $lines = @('classification=verified-extracted-nsis-payload', 'required_resource_count=12', 'aster_builder_resource_count=10', 'preflight_entry_count=13', 'preflight_total_bytes=12345', 'extracted_file_bytes=12345')
    foreach ($name in $hashes.Keys) { $lines += "$name.sha256=$($hashes[$name])"; $lines += "$name.byte_count=$($sizes[$name])" }
    $lines += '--- 7z listing follows ---'
    $lines += 'Path = synthetic-non-resource.txt'
    $members = @($sizes.Keys | ForEach-Object { [pscustomobject]@{ Path=$_; Size=$sizes[$_] } })
    $members += [pscustomobject]@{ Path='aster-desktop.exe'; Size=101 }
    return [pscustomobject]@{ Manifest=$manifest; Header=($lines -join "`n") + "`n"; Members=$members; Sizes=$sizes }
}
$passedCases = New-Object 'System.Collections.Generic.List[string]'
function Assert-Denial([string]$Name, [scriptblock]$Action) {
    $denied = $false
    try { & $Action | Out-Null } catch { $denied=$true }
    if (-not $denied) { throw "Synthetic negative fixture was accepted: $Name" }
    $passedCases.Add($Name)
}
$fixture = New-SyntheticResourceFixture
$validatedSizes = Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header
Assert-RequiredResourceMembers $fixture.Members $validatedSizes
$passedCases.Add('exact-twelve-resource-evidence-and-members')
$fixture.Manifest = $fixture.Manifest | ConvertTo-Json -Depth 3 | ConvertFrom-Json
$roundtripSizes = Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header
Assert-RequiredResourceMembers $fixture.Members $roundtripSizes
$passedCases.Add('json-roundtrip-resource-length-types')
$fixture.Members | ForEach-Object { $_.Path=$_.Path.Replace('/', '\') }
Assert-RequiredResourceMembers $fixture.Members $validatedSizes
$passedCases.Add('canonical-backslash-listing-members')
foreach ($missing in $fixedInventory.Keys) {
    $fixture = New-SyntheticResourceFixture
    Assert-Denial ('missing:' + $missing) { Assert-RequiredResourceMembers @($fixture.Members | Where-Object { $_.Path -cne $missing }) $fixture.Sizes }
}
$fixture = New-SyntheticResourceFixture
Assert-Denial 'duplicate-member' { Assert-RequiredResourceMembers ($fixture.Members + $fixture.Members[2]) $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
$fixture.Members[2].Path='ASTER-BUILDER/SKILL.md'
Assert-Denial 'case-aliased-builder-prefix' { Assert-RequiredResourceMembers $fixture.Members $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
$fixture.Members[2].Path='aster-builder/skill.md'
Assert-Denial 'case-aliased-builder-leaf' { Assert-RequiredResourceMembers $fixture.Members $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
Assert-Denial 'trailing-dot-builder-prefix-alias' { Assert-RequiredResourceMembers ($fixture.Members + [pscustomobject]@{ Path='aster-builder./scripts/hook.ps1'; Size=1 }) $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
Assert-Denial 'trailing-space-builder-prefix-alias' { Assert-RequiredResourceMembers ($fixture.Members + [pscustomobject]@{ Path='aster-builder /scripts/hook.ps1'; Size=1 }) $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
Assert-Denial 'trailing-dot-existing-license-alias' { Assert-RequiredResourceMembers ($fixture.Members + [pscustomobject]@{ Path='LICENSE.txt.'; Size=13 }) $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
Assert-Denial 'unknown-executable-member' { Assert-RequiredResourceMembers ($fixture.Members + [pscustomobject]@{ Path='aster-builder/scripts/hook.ps1'; Size=1 }) $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
Assert-Denial 'nested-builder-prefix' { Assert-RequiredResourceMembers ($fixture.Members + [pscustomobject]@{ Path='other/aster-builder/SKILL.md'; Size=7329 }) $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
$fixture.Members[2].Size++
Assert-Denial 'changed-original-member-length' { Assert-RequiredResourceMembers $fixture.Members $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
$fixture.Members[0].Path='third-party-notices.txt'
Assert-Denial 'case-aliased-existing-notice' { Assert-RequiredResourceMembers $fixture.Members $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
Assert-Denial 'nested-existing-license' { Assert-RequiredResourceMembers ($fixture.Members + [pscustomobject]@{ Path='other/LICENSE.txt'; Size=13 }) $fixture.Sizes }
$fixture = New-SyntheticResourceFixture
$fixture.Manifest.bundle_resource_sha256.'aster-builder/SKILL.md'='c' * 64
$fixture.Header=$fixture.Header.Replace($fixedInventory['aster-builder/SKILL.md'].sha256, ('c' * 64))
Assert-Denial 'manifest-and-header-replace-original-hash' { Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header }
$fixture = New-SyntheticResourceFixture
$fixture.Manifest.bundle_resource_byte_count.'aster-builder/SKILL.md'=7330
$fixture.Header=$fixture.Header.Replace('aster-builder/SKILL.md.byte_count=7329', 'aster-builder/SKILL.md.byte_count=7330')
Assert-Denial 'manifest-and-header-replace-original-length' { Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header }
$fixture = New-SyntheticResourceFixture
$fixture.Manifest.bundle_resource_sha256 | Add-Member -NotePropertyName 'aster-builder/extra.md' -NotePropertyValue ('d' * 64)
Assert-Denial 'extra-manifest-hash-property' { Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header }
$fixture = New-SyntheticResourceFixture
$fixture.Manifest.bundle_resource_sha256.PSObject.Properties.Remove('aster-builder/SKILL.md')
$fixture.Manifest.bundle_resource_sha256 | Add-Member -NotePropertyName 'aster-builder/skill.md' -NotePropertyValue $fixedInventory['aster-builder/SKILL.md'].sha256
Assert-Denial 'case-aliased-manifest-property' { Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header }
$fixture = New-SyntheticResourceFixture
$fixture.Manifest.bundle_resource_sha256.PSObject.Properties.Remove('aster-builder/SKILL.md')
Assert-Denial 'missing-manifest-hash-property' { Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header }
$fixture = New-SyntheticResourceFixture
$fixture.Manifest.bundle_resource_byte_count | Add-Member -NotePropertyName 'aster-builder/extra.md' -NotePropertyValue 1
Assert-Denial 'extra-manifest-length-property' { Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header }
$fixture = New-SyntheticResourceFixture
$fixture.Manifest.bundle_resource_byte_count.'LICENSE.txt'=-1
Assert-Denial 'negative-manifest-length' { Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header }
$fixture = New-SyntheticResourceFixture
$fixture.Manifest.bundle_resource_byte_count.'LICENSE.txt'='13'
Assert-Denial 'string-manifest-length' { Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header }
$fixture = New-SyntheticResourceFixture
Assert-Denial 'duplicated-header-line' { Assert-RequiredResourceEvidence $fixture.Manifest ("required_resource_count=12`n" + $fixture.Header) }
$fixture = New-SyntheticResourceFixture
Assert-Denial 'incorrect-header-count' { Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header.Replace('required_resource_count=12', 'required_resource_count=2') }
$fixture = New-SyntheticResourceFixture
Assert-Denial 'header-forges-fixed-manifest-hash' { Assert-RequiredResourceEvidence $fixture.Manifest $fixture.Header.Replace($fixedInventory['aster-builder/SKILL.md'].sha256, ('f' * 64)) }
[pscustomobject][ordered]@{
    classification='static-synthetic-resource-contract-fixtures'
    status='PASS'
    observed_at_utc=(Get-Date).ToUniversalTime().ToString('o')
    powershell_version=$PSVersionTable.PSVersion.ToString()
    verifier_sha256=(Get-FileHash -LiteralPath $verifierPath -Algorithm SHA256).Hash.ToLowerInvariant()
    workflow_sha256=(Get-FileHash -LiteralPath $workflowPath -Algorithm SHA256).Hash.ToLowerInvariant()
    fixture_sha256=(Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
    original_inert_source_identity_count=$fixedInventory.Count
    exact_config_mapping_count=$configMappings.Count
    successful_cases=$passedCases.Count
    cases=@($passedCases.ToArray())
    artifact_verification='NOT RUN'
    native_resource_probes='NOT RUN'
    qualificationComplete=$false
} | ConvertTo-Json -Depth 4
