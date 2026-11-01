#requires -Version 5.1
[CmdletBinding(DefaultParameterSetName = 'Main')]
param(
    [Parameter(Mandatory = $true)][string]$ZipPath,
    [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ExpectedZipSha256,
    [Parameter(Mandatory = $true)][ValidatePattern('^[0-9a-f]{40}$')][string]$ExpectedSourceRevision,
    [Parameter(Mandatory = $true)][ValidatePattern('^[0-9]+$')][string]$ExpectedRunId,
    [Parameter(Mandatory = $true)][ValidatePattern('^[1-9][0-9]*$')][string]$ExpectedRunAttempt,
    [Parameter(Mandatory = $true)][string]$Destination,
    [Parameter(Mandatory = $true, ParameterSetName = 'DiagnosticPullRequest')]
    [ValidatePattern('^[1-9][0-9]*$')][string]$DiagnosticPullRequestNumber
)

# Verifies and materializes, but never executes/installs the downloaded artifact.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
function Assert-InstallerProvenance($Manifest, [string]$SourceRevision, [string]$RunId, [string]$RunAttempt, [string]$DiagnosticPrNumber) {
    if ($Manifest -isnot [pscustomobject]) { throw 'Installer provenance must be a JSON object.' }
    foreach ($field in @('source_revision', 'workflow_run_id', 'workflow_run_attempt', 'classification', 'workflow_ref', 'workflow_event')) {
        $property = $Manifest.PSObject.Properties[$field]
        if ($null -eq $property -or $property.Value -isnot [string]) { throw 'Installer provenance fields must be present scalar strings.' }
    }
    if ($SourceRevision -cnotmatch '^[0-9a-f]{40}$' -or $RunId -cnotmatch '^[1-9][0-9]*$' -or $RunAttempt -cnotmatch '^[1-9][0-9]*$' -or
        $Manifest.source_revision -cne $SourceRevision -or $Manifest.workflow_run_id -cne $RunId -or $Manifest.workflow_run_attempt -cne $RunAttempt) {
        throw 'Installer source/run/attempt provenance is not the independently expected identity.'
    }
    if ([string]::IsNullOrEmpty($DiagnosticPrNumber)) {
        if ($Manifest.classification -cne 'unsigned-engineering-build' -or $Manifest.workflow_ref -cne 'refs/heads/main' -or
            $Manifest.workflow_event -cnotin @('push', 'workflow_dispatch')) { throw 'Final engineering staging requires actual-main provenance.' }
        return 'Main'
    }
    if ($DiagnosticPrNumber -cnotmatch '^[1-9][0-9]*$' -or $DiagnosticPrNumber.Length -gt 20 -or
        $Manifest.classification -cne 'unsigned-engineering-candidate' -or $Manifest.workflow_ref -cne "refs/pull/$DiagnosticPrNumber/merge" -or
        $Manifest.workflow_event -cne 'pull_request') { throw 'Diagnostic staging requires the exact separately selected PR-candidate provenance.' }
    return 'DiagnosticPullRequest'
}
function Get-Sha256Hex([byte[]]$Bytes) {
    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try { return [System.BitConverter]::ToString($algorithm.ComputeHash($Bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $algorithm.Dispose() }
}
function Get-AsterBuilderResourceInventory {
    # Exact original inert ZIP-member identities; never accept manifest-defined replacements.
    return [ordered]@{
        'aster-builder/SKILL.md' = @{ byte_count=7329; sha256='4a2eb40201f49ac4dee89cfd08852b4f764c44e770d90bee2d4dd18847773c71' }
        'aster-builder/references/architecture.md' = @{ byte_count=2240; sha256='f1835a20299004b83318d9ebb068a909deee02a8518af4dcecaad77b22dc7f10' }
        'aster-builder/references/spec-threats.md' = @{ byte_count=1677; sha256='0a5c611e7508d9d13251f97a682afa4fc8a382c8966b511f5b0d537ef3ab3cfd' }
        'aster-builder/references/provider-contracts.md' = @{ byte_count=1655; sha256='675b3443e9005a559d4d5d719121aac3cbd826929c1508646db8f98577b6f294' }
        'aster-builder/references/storage-tools.md' = @{ byte_count=1481; sha256='46b01b2fb5dd3a8adb00cafad6d71e92e188dfd30c6e40bcf6fdb8ea24db9ac9' }
        'aster-builder/references/ui.md' = @{ byte_count=1613; sha256='c22cfa4a62ddf334f3b3b9a24f46b5d755c4cddcb4204d758c0e548e9548720a' }
        'aster-builder/references/verification.md' = @{ byte_count=2256; sha256='ef0e3c086f350c2b9ef629f087c4a80f66de308f7efd97fb5ca61ec1f4a534bf' }
        'aster-builder/references/aster-case-study.md' = @{ byte_count=1160; sha256='31e868a022e71c84fa2bd0418a2ff621c597fc576bdd056fc90be303afadaf3a' }
        'aster-builder/references/ledger-schema.md' = @{ byte_count=4172; sha256='d322712e3d4614827248aceed46ff82ac126501e60f805ba94a68a86191dd591' }
        'aster-builder/assets/icon.svg' = @{ byte_count=1738; sha256='8936b24f674a1250d999e2c2b5901374fb4240426f32bd1ae340c352a90d617d' }
    }
}
function Assert-RequiredResourceMembers([object[]]$Members, $ExpectedSizes) {
    $names = @($ExpectedSizes.Keys)
    $seen = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::Ordinal)
    foreach ($member in $Members) {
        $path = ([string]$member.Path).Replace('\', '/')
        $parts = @($path.Split('/'))
        $leaf = $parts[$parts.Count - 1]
        $isBuilder = @($parts | Where-Object { [string]::Equals($_.TrimEnd(' ', '.'), 'aster-builder', [System.StringComparison]::OrdinalIgnoreCase) }).Count -gt 0
        $isResource = $isBuilder -or $leaf.TrimEnd(' ', '.') -in @('THIRD-PARTY-NOTICES.txt', 'LICENSE.txt')
        if (-not $isResource) { continue }
        if ($names -cnotcontains $path -or -not $seen.Add($path)) { throw 'Installer resource namespace contains an unknown, aliased or duplicate member.' }
        if ([long]$member.Size -ne [long]$ExpectedSizes[$path]) { throw 'Installer resource member length differs from the fixed inventory.' }
    }
    if ($seen.Count -ne $names.Count) { throw 'Installer resource inventory is incomplete.' }
}
function Assert-RequiredResourceEvidence($Manifest, [string]$PayloadList) {
    $builderInventory = Get-AsterBuilderResourceInventory
    $names = @('THIRD-PARTY-NOTICES.txt', 'LICENSE.txt') + @($builderInventory.Keys)
    $hashProperties = @($Manifest.bundle_resource_sha256.PSObject.Properties)
    $sizeProperties = @($Manifest.bundle_resource_byte_count.PSObject.Properties)
    if ($hashProperties.Count -ne $names.Count -or $sizeProperties.Count -ne $names.Count -or
        @($hashProperties | Where-Object { $names -cnotcontains $_.Name }).Count -ne 0 -or
        @($sizeProperties | Where-Object { $names -cnotcontains $_.Name }).Count -ne 0) { throw 'Manifest resource hash/length sets are not exact.' }
    $separator = "`n--- 7z listing follows ---`n"
    $headerEnd = $PayloadList.IndexOf($separator, [System.StringComparison]::Ordinal)
    if ($headerEnd -le 0) { throw 'Installer payload evidence lacks its exact listing separator.' }
    $headerLines = @($PayloadList.Substring(0, $headerEnd).Split("`n"))
    if ($headerLines.Count -ne 30 -or
        @($headerLines | Where-Object { $_ -ceq 'classification=verified-extracted-nsis-payload' }).Count -ne 1 -or
        @($headerLines | Where-Object { $_ -ceq 'required_resource_count=12' }).Count -ne 1 -or
        @($headerLines | Where-Object { $_ -ceq 'aster_builder_resource_count=10' }).Count -ne 1 -or
        @($headerLines | Where-Object { $_ -cmatch '^preflight_entry_count=[1-9][0-9]*$' }).Count -ne 1 -or
        @($headerLines | Where-Object { $_ -cmatch '^preflight_total_bytes=[0-9]+$' }).Count -ne 1 -or
        @($headerLines | Where-Object { $_ -cmatch '^extracted_file_bytes=[0-9]+$' }).Count -ne 1) { throw 'Installer payload evidence classification/count is invalid.' }
    $sizes = [ordered]@{}
    foreach ($name in $names) {
        $hash = $Manifest.bundle_resource_sha256.PSObject.Properties[$name].Value
        $size = $Manifest.bundle_resource_byte_count.PSObject.Properties[$name].Value
        if ($hash -isnot [string] -or $hash -cnotmatch '^[0-9a-f]{64}$' -or
            ($size -isnot [int] -and $size -isnot [long]) -or $size -le 0 -or $size -gt 268435456 -or
            @($headerLines | Where-Object { $_ -ceq "$name.sha256=$hash" }).Count -ne 1 -or
            @($headerLines | Where-Object { $_ -ceq "$name.byte_count=$size" }).Count -ne 1) { throw 'Installer payload evidence is not bound to the exact manifest resource identity.' }
        if ($builderInventory.Contains($name) -and
            ($hash -cne $builderInventory[$name].sha256 -or [long]$size -ne [long]$builderInventory[$name].byte_count)) { throw 'Manifest replaced an original inert Aster Builder resource identity.' }
        $sizes[$name] = [long]$size
    }
    return $sizes
}
function Assert-SafeSevenZipListing($SevenZip, [string]$ArchivePath) {
    $lines = @(& $SevenZip.Source l -slt -ba $ArchivePath)
    if ($LASTEXITCODE -ne 0 -or $lines.Count -eq 0) { throw 'Installer payload preflight listing failed.' }
    $entryCount = 0
    [long]$totalBytes = 0
    $currentPath = $null
    $currentSize = $null
    $members = New-Object 'System.Collections.Generic.List[object]'
    foreach ($line in @($lines) + '') {
        if ([string]::IsNullOrWhiteSpace([string]$line)) {
            if ($null -ne $currentPath) {
                [long]$parsedSize = 0
                if ($null -eq $currentSize -or -not [long]::TryParse([string]$currentSize, [ref]$parsedSize) -or $parsedSize -lt 0 -or $parsedSize -gt 268435456) { throw 'Installer payload member size is invalid or exceeds 256 MiB.' }
                $normalized = ([string]$currentPath).Replace('/', '\')
                if ($normalized.Length -eq 0 -or $normalized.Length -gt 4096 -or
                    [System.IO.Path]::IsPathRooted($normalized) -or $normalized.StartsWith('\') -or
                    $normalized -match ':' -or $normalized -match '(^|\\)\.{1,2}(\\|$)') { throw 'Installer payload member path is unsafe.' }
                if ($totalBytes -gt 536870912 - $parsedSize) { throw 'Installer payload exceeds the 512 MiB expanded-size bound.' }
                $totalBytes += $parsedSize
                $entryCount++
                if ($entryCount -gt 8192) { throw 'Installer payload has too many members.' }
                $members.Add([pscustomobject]@{ Path=$normalized.Replace('\', '/'); Size=$parsedSize })
                $currentPath = $null
                $currentSize = $null
            }
            continue
        }
        if ([string]$line -clike 'Path = *') { $currentPath = ([string]$line).Substring(7); continue }
        if ([string]$line -clike 'Size = *') { $currentSize = ([string]$line).Substring(7); continue }
        if ([string]$line -clike 'Anti = +*' -or [string]$line -clike 'Symbolic Link = *' -or [string]$line -clike 'Hard Link = *') { throw 'Installer payload contains a link or anti-item.' }
    }
    if ($entryCount -eq 0) { throw 'Installer payload listing contains no members.' }
    return [pscustomobject]@{ Lines = $lines; EntryCount = $entryCount; TotalBytes = $totalBytes; Members = @($members.ToArray()) }
}
$zipFull = [System.IO.Path]::GetFullPath($ZipPath)
$destinationFull = [System.IO.Path]::GetFullPath($Destination).TrimEnd('\', '/')
if (-not [System.IO.Path]::IsPathRooted($Destination) -or [string]::Equals($destinationFull, [System.IO.Path]::GetPathRoot($destinationFull).TrimEnd('\', '/'), [System.StringComparison]::OrdinalIgnoreCase)) { throw 'Destination must be a named absolute child directory.' }
if (Test-Path -LiteralPath $destinationFull) { throw 'Destination must not already exist.' }
$parent = Get-Item -LiteralPath (Split-Path -Parent $destinationFull) -Force
if (-not $parent.PSIsContainer -or ($parent.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Destination parent is not a normal directory.' }
$zipItem = Get-Item -LiteralPath $zipFull -Force
if ($zipItem.PSIsContainer -or ($zipItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) { throw 'Artifact ZIP must be a normal file.' }
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zipStream = [System.IO.File]::Open($zipFull, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read)
$archive = $null
$installerBytes = $null
$manifestBytes = $null
$evidenceBytes = @{}
try {
    # Hash and parse through the same non-write/non-delete-shared handle. A path
    # replacement cannot change the archive bytes underneath their reported hash.
    $actualZipSha256 = (Get-FileHash -InputStream $zipStream -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualZipSha256 -cne $ExpectedZipSha256) { throw 'GitHub artifact ZIP digest mismatch.' }
    $zipStream.Position = 0
    $archive = [System.IO.Compression.ZipArchive]::new($zipStream, [System.IO.Compression.ZipArchiveMode]::Read, $false)
    $expectedNames = @('manifest.json', 'Aster Desktop_0.2.4_x64-setup.exe', 'THIRD-PARTY-NOTICES.txt', 'third-party-notice-inventory.json', 'installer-payload-list.txt')
    if ($archive.Entries.Count -ne 5) { throw 'Artifact must have exactly five entries.' }
    $seen = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
    foreach ($entry in $archive.Entries) {
        if ($expectedNames -cnotcontains $entry.FullName -or -not $seen.Add($entry.FullName)) { throw 'Artifact contains an unexpected or duplicate entry.' }
        $attributeBits = [System.BitConverter]::ToUInt32([System.BitConverter]::GetBytes([int]$entry.ExternalAttributes), 0)
        if ((($attributeBits -shr 16) -band 0xF000) -eq 0xA000 -or ($attributeBits -band 0x400) -ne 0) { throw 'Artifact contains a link or reparse entry.' }
        $limit = switch ($entry.FullName) { 'manifest.json' { 16384 }; 'Aster Desktop_0.2.4_x64-setup.exe' { 67108864 }; 'THIRD-PARTY-NOTICES.txt' { 33554432 }; default { 4194304 } }
        if ($entry.Length -le 0 -or $entry.Length -gt $limit) { throw 'Artifact entry exceeds its size limit or is empty.' }
        $stream = $entry.Open()
        $memory = [System.IO.MemoryStream]::new()
        try {
            [byte[]]$buffer = New-Object byte[] 8192
            while ($true) {
                $readCount = $stream.Read($buffer, 0, $buffer.Length)
                if ($readCount -eq 0) { break }
                if ($memory.Length + $readCount -gt $entry.Length -or $memory.Length + $readCount -gt $limit) { throw 'Decompressed artifact entry exceeds its declared length or size limit.' }
                $memory.Write($buffer, 0, $readCount)
            }
            $bytes=$memory.ToArray()
        } finally { $stream.Dispose(); $memory.Dispose() }
        if ($bytes.LongLength -ne $entry.Length) { throw 'Artifact entry length mismatch.' }
        if ($entry.FullName -ceq 'manifest.json') { $manifestBytes=$bytes } elseif ($entry.FullName -ceq 'Aster Desktop_0.2.4_x64-setup.exe') { $installerBytes=$bytes } else { $evidenceBytes[$entry.FullName]=$bytes }
    }
} finally { if ($null -ne $archive) { $archive.Dispose() } else { $zipStream.Dispose() } }
$strictUtf8 = [System.Text.UTF8Encoding]::new($false, $true)
$manifest = $strictUtf8.GetString($manifestBytes).TrimStart([char]0xFEFF) | ConvertFrom-Json
$provenanceMode = Assert-InstallerProvenance $manifest $ExpectedSourceRevision $ExpectedRunId $ExpectedRunAttempt $DiagnosticPullRequestNumber
if ($manifest.filename -cne 'Aster Desktop_0.2.4_x64-setup.exe' -or
    [long]$manifest.byte_count -ne $installerBytes.LongLength -or
    $manifest.authenticode_status -cne 'NotSigned' -or $manifest.runner -cne 'Windows' -or
    $manifest.sha256 -cnotmatch '^[0-9a-f]{64}$') { throw 'Installer manifest byte/signature binding does not match the expected CI artifact.' }
$actualInstallerSha256 = Get-Sha256Hex $installerBytes
if ($actualInstallerSha256 -cne $manifest.sha256) { throw 'Installer SHA-256 does not match manifest.' }
$expectedEvidenceNames = @('THIRD-PARTY-NOTICES.txt', 'third-party-notice-inventory.json', 'installer-payload-list.txt')
if (@($manifest.evidence_sha256.PSObject.Properties).Count -ne $expectedEvidenceNames.Count) { throw 'Manifest evidence hash set is not exact.' }
foreach ($evidenceName in @('THIRD-PARTY-NOTICES.txt', 'third-party-notice-inventory.json', 'installer-payload-list.txt')) {
    $evidenceHash = Get-Sha256Hex $evidenceBytes[$evidenceName]
    if ($evidenceHash -cne $manifest.evidence_sha256.PSObject.Properties[$evidenceName].Value) { throw 'Supplemental evidence digest mismatch.' }
}

# Independently bind the inventory structure to the exact notice bytes. This
# validates reproducible evidence only; it is not a legal sufficiency opinion.
$inventory = $strictUtf8.GetString($evidenceBytes['third-party-notice-inventory.json']).TrimStart([char]0xFEFF) | ConvertFrom-Json
$packages = @($inventory.packages)
if ([int]$inventory.schemaVersion -ne 2 -or
    $inventory.classification -cne 'installed-locked-source-notice-inventory' -or
    $inventory.cargoFilterPlatform -cne 'x86_64-pc-windows-msvc' -or
    $packages.Count -le 0 -or [int]$inventory.packageCount -ne $packages.Count -or
    $inventory.legalConclusion -cnotmatch '^UNASSESSED') { throw 'Notice inventory header/scope is invalid.' }
$packageKeys = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::Ordinal)
$requiredCount = 0
$directMissingCount = 0
$peerCoveredCount = 0
$overlayCoveredCount = 0
$canonicalOverlayCoveredCount = 0
$missingCount = 0
$derivedRequiredMissing = New-Object 'System.Collections.Generic.List[string]'
$derivedNonRequiredMissing = New-Object 'System.Collections.Generic.List[string]'
$derivedRequiredUnknown = New-Object 'System.Collections.Generic.List[string]'
foreach ($package in $packages) {
    if ($package.ecosystem -cnotin @('cargo', 'npm') -or
        [string]::IsNullOrWhiteSpace([string]$package.name) -or
        [string]::IsNullOrWhiteSpace([string]$package.version) -or
        [string]::IsNullOrWhiteSpace([string]$package.source) -or
        $package.requiredForPackagedApp -isnot [bool]) { throw 'Notice inventory contains an invalid package record.' }
    $key = '{0}|{1}|{2}|{3}' -f $package.ecosystem, $package.name, $package.version, $package.source
    if (-not $packageKeys.Add($key)) { throw 'Notice inventory contains a duplicate package identity.' }
    $texts = @($package.texts)
    $hasPackageRootText = $false
    foreach ($noticeText in $texts) {
        if ([string]::IsNullOrWhiteSpace([string]$noticeText.name) -or
            [string]$noticeText.sha256 -cnotmatch '^[0-9a-f]{64}$' -or
            [long]$noticeText.byteCount -le 0 -or
            ([string]$noticeText.origin -cne 'package-root' -and
             [string]$noticeText.origin -cnotmatch '^installed-repository-peer:' -and
             [string]$noticeText.origin -cnotmatch '^reviewed-overlay:(upstream-repository-original|canonical-license-steward):[^\\/]+$')) {
            throw 'Notice inventory contains an invalid text record.'
        }
        if ([string]$noticeText.origin -ceq 'package-root') { $hasPackageRootText = $true }
    }
    if (-not $hasPackageRootText) { $directMissingCount++ }
    if (-not $hasPackageRootText -and @($texts | Where-Object { [string]$_.origin -cmatch '^installed-repository-peer:' }).Count -gt 0) { $peerCoveredCount++ }
    if (-not $hasPackageRootText -and @($texts | Where-Object { [string]$_.origin -cmatch '^reviewed-overlay:' }).Count -gt 0) { $overlayCoveredCount++ }
    if (-not $hasPackageRootText -and @($texts | Where-Object { [string]$_.origin -cmatch '^reviewed-overlay:canonical-license-steward:' }).Count -gt 0) { $canonicalOverlayCoveredCount++ }
    if ($texts.Count -eq 0) { $missingCount++ }
    $identity = '{0}:{1}@{2}' -f $package.ecosystem, $package.name, $package.version
    if ($package.requiredForPackagedApp) {
        $requiredCount++
        if ($texts.Count -eq 0) { [void]$derivedRequiredMissing.Add($identity) }
        if (([string]$package.declaredLicense -replace '\s+', '').ToUpperInvariant() -ceq 'UNKNOWN') { [void]$derivedRequiredUnknown.Add($identity) }
    } elseif ($texts.Count -eq 0) {
        [void]$derivedNonRequiredMissing.Add($identity)
    }
}
$reportedRequiredMissing = @($inventory.unresolvedRequiredPackages)
$reportedNonRequiredMissing = @($inventory.unresolvedNonRequiredPackages)
$reportedRequiredUnknown = @($inventory.unknownLicenseRequiredPackages)
if ([int]$inventory.requiredPackageCount -ne $requiredCount -or
    [int]$inventory.directMissingTextCount -ne $directMissingCount -or
    [int]$inventory.repositoryPeerCoveredCount -ne $peerCoveredCount -or
    [int]$inventory.reviewedOverlayCoveredCount -ne $overlayCoveredCount -or
    [int]$inventory.canonicalLicenseStewardCoveredCount -ne $canonicalOverlayCoveredCount -or
    [int]$inventory.missingTextCount -ne $missingCount -or
    [int]$inventory.requiredMissingTextCount -ne $derivedRequiredMissing.Count -or
    [int]$inventory.nonRequiredMissingTextCount -ne $derivedNonRequiredMissing.Count -or
    [int]$inventory.requiredUnknownLicenseCount -ne $derivedRequiredUnknown.Count -or
    ($reportedRequiredMissing -join "`n") -cne ($derivedRequiredMissing -join "`n") -or
    ($reportedNonRequiredMissing -join "`n") -cne ($derivedNonRequiredMissing -join "`n") -or
    ($reportedRequiredUnknown -join "`n") -cne ($derivedRequiredUnknown -join "`n")) {
    throw 'Notice inventory summary is inconsistent with its package records.'
}
$expectedOverlayFingerprints = @{
    'cargo|selectors|0.38.0' = 'MPL-2.0|github.com/servo/stylo|572ecba2d1600e7c3d490586692a209faf703baa|selectors|Mozilla-MPL-2.0.txt|16726|3f3d9e0024b1921b067d6f7f88deb4a60cbe7a78e76c64e3f1d7fc3b779b9d04|canonical-license-steward|https://www.mozilla.org/media/MPL/2.0/index.txt|https://github.com/servo/stylo/tree/572ecba2d1600e7c3d490586692a209faf703baa/selectors|False'
    'cargo|webview2-com|0.39.1' = 'MIT|github.com/wravery/webview2-rs|edc2caf886175ccaebe86078c9cfe1ae2a187328|crates/webview2-com|webview2-rs-MIT.txt|1067|0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73|upstream-repository-original|https://raw.githubusercontent.com/wravery/webview2-rs/edc2caf886175ccaebe86078c9cfe1ae2a187328/LICENSE|https://github.com/wravery/webview2-rs/tree/edc2caf886175ccaebe86078c9cfe1ae2a187328/crates/webview2-com|True'
    'cargo|webview2-com-macros|0.8.1' = 'MIT|github.com/wravery/webview2-rs|dffa41a8a46d3f5565eefbff2de57d38d399f158|crates/callback-macros|webview2-rs-MIT.txt|1067|0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73|upstream-repository-original|https://raw.githubusercontent.com/wravery/webview2-rs/dffa41a8a46d3f5565eefbff2de57d38d399f158/LICENSE|https://github.com/wravery/webview2-rs/tree/dffa41a8a46d3f5565eefbff2de57d38d399f158/crates/callback-macros|True'
    'cargo|webview2-com-sys|0.39.1' = 'MIT|github.com/wravery/webview2-rs|edc2caf886175ccaebe86078c9cfe1ae2a187328|crates/bindings|webview2-rs-MIT.txt|1067|0dcf41516e608bbcb6cdc5229feb7b86fe4a643b85e7df251133c93408fdac73|upstream-repository-original|https://raw.githubusercontent.com/wravery/webview2-rs/edc2caf886175ccaebe86078c9cfe1ae2a187328/LICENSE|https://github.com/wravery/webview2-rs/tree/edc2caf886175ccaebe86078c9cfe1ae2a187328/crates/bindings|True'
}
$reviewedOverlays = @($inventory.reviewedOverlays)
if ([string]$inventory.overlayManifestSha256 -cnotmatch '^[0-9a-f]{64}$' -or
    [long]$inventory.overlayManifestByteCount -le 0 -or
    $reviewedOverlays.Count -ne $expectedOverlayFingerprints.Count -or
    $overlayCoveredCount -ne $expectedOverlayFingerprints.Count) { throw 'Reviewed overlay manifest binding/count is invalid.' }
$seenOverlays = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::Ordinal)
foreach ($overlay in $reviewedOverlays) {
    if (@($overlay.PSObject.Properties).Count -ne 14) { throw 'Reviewed overlay entry shape is not exact.' }
    $overlayKey = '{0}|{1}|{2}' -f $overlay.ecosystem, $overlay.name, $overlay.version
    if (-not $seenOverlays.Add($overlayKey) -or -not $expectedOverlayFingerprints.ContainsKey($overlayKey)) { throw 'Reviewed overlay identity is duplicate or unknown.' }
    $fingerprint = @(
        [string]$overlay.declaredLicense, [string]$overlay.repository, [string]$overlay.vcsSha1,
        [string]$overlay.pathInVcs, [string]$overlay.noticeFile, [string]$overlay.noticeByteCount,
        [string]$overlay.noticeSha256, [string]$overlay.originKind, [string]$overlay.sourceUrl,
        [string]$overlay.packageSourceAccessUrl, [string]$overlay.upstreamRepositoryOriginal
    ) -join '|'
    if ($fingerprint -cne $expectedOverlayFingerprints[$overlayKey]) { throw 'Reviewed overlay provenance/hash changed.' }
    $boundPackages = @($packages | Where-Object {
        [string]$_.ecosystem -ceq [string]$overlay.ecosystem -and
        [string]$_.name -ceq [string]$overlay.name -and
        [string]$_.version -ceq [string]$overlay.version
    })
    if ($boundPackages.Count -ne 1 -or
        [string]$boundPackages[0].declaredLicense -cne [string]$overlay.declaredLicense -or
        [string]$boundPackages[0].repository -cne [string]$overlay.repository) { throw 'Reviewed overlay is not bound to one exact package record.' }
    $expectedOrigin = 'reviewed-overlay:{0}:{1}' -f $overlay.originKind, $overlay.noticeFile
    $boundTexts = @($boundPackages[0].texts | Where-Object {
        [string]$_.name -ceq [string]$overlay.noticeFile -and
        [string]$_.sha256 -ceq [string]$overlay.noticeSha256 -and
        [long]$_.byteCount -eq [long]$overlay.noticeByteCount -and
        [string]$_.origin -ceq $expectedOrigin
    })
    if ($boundTexts.Count -ne 1) { throw 'Reviewed overlay text is not bound to its package record.' }
}
if ($derivedRequiredMissing.Count -ne 0 -or $derivedRequiredUnknown.Count -ne 0) { throw 'Required packaged-app dependency notice/license review remains unresolved.' }
$noticeHash = Get-Sha256Hex $evidenceBytes['THIRD-PARTY-NOTICES.txt']
if ([long]$inventory.noticeByteCount -ne $evidenceBytes['THIRD-PARTY-NOTICES.txt'].LongLength -or
    [string]$inventory.noticeSha256 -cne $noticeHash) { throw 'Notice inventory is not bound to the notice file bytes.' }

$payloadList = $strictUtf8.GetString($evidenceBytes['installer-payload-list.txt']).TrimStart([char]0xFEFF).Replace("`r`n", "`n").Replace("`r", "`n")
$resourceSizes = Assert-RequiredResourceEvidence $manifest $payloadList
$resourceNames = @($resourceSizes.Keys)
if ([string]$manifest.bundle_resource_sha256.PSObject.Properties['THIRD-PARTY-NOTICES.txt'].Value -cne $noticeHash -or
    [long]$resourceSizes['THIRD-PARTY-NOTICES.txt'] -ne $evidenceBytes['THIRD-PARTY-NOTICES.txt'].LongLength) {
    throw 'Bundled notice resource is not bound to supplemental notice evidence.'
}
[void](New-Item -ItemType Directory -Path $destinationFull -ErrorAction Stop)
$payloads = @(@{ name='manifest.json'; bytes=$manifestBytes }, @{ name=$manifest.filename; bytes=$installerBytes })
foreach ($evidenceName in @('THIRD-PARTY-NOTICES.txt', 'third-party-notice-inventory.json', 'installer-payload-list.txt')) { $payloads += @{ name=$evidenceName; bytes=$evidenceBytes[$evidenceName] } }
foreach ($payload in $payloads) {
    $outputPath = Join-Path $destinationFull $payload.name
    $output = [System.IO.File]::Open($outputPath, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write, [System.IO.FileShare]::None)
    try { $output.Write($payload.bytes, 0, $payload.bytes.Length); $output.Flush($true) } finally { $output.Dispose() }
}
$installerPath = Join-Path $destinationFull $manifest.filename
$signature = Get-AuthenticodeSignature -LiteralPath $installerPath
if ([string]$signature.Status -cne 'NotSigned') { throw 'Independent local Authenticode status is not NotSigned.' }
if ((Get-FileHash -LiteralPath $installerPath -Algorithm SHA256).Hash.ToLowerInvariant() -cne $actualInstallerSha256) { throw 'Materialized installer digest mismatch.' }
$sevenZip = Get-Command 7z.exe -ErrorAction Stop
$payloadPreflight = Assert-SafeSevenZipListing $sevenZip $installerPath
Assert-RequiredResourceMembers $payloadPreflight.Members $resourceSizes
if ($payloadList -cnotmatch ('(?m)^preflight_entry_count=' + $payloadPreflight.EntryCount + '$') -or
    $payloadList -cnotmatch ('(?m)^preflight_total_bytes=' + $payloadPreflight.TotalBytes + '$')) { throw 'Independent payload preflight differs from CI evidence.' }
$payloadDirectory = Join-Path $destinationFull 'verified-installer-payload'
[void](New-Item -ItemType Directory -Path $payloadDirectory -ErrorAction Stop)
& $sevenZip.Source x $installerPath "-o$payloadDirectory" -y | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Independent installer payload extraction failed.' }
$payloadItems = @(Get-ChildItem -LiteralPath $payloadDirectory -Recurse -Force)
if (@($payloadItems | Where-Object { ($_.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0 }).Count -ne 0) { throw 'Extracted installer payload contains a link/reparse point.' }
# NSIS uninstall.exe size metadata describes encoded patches; the extractor
# reconstructs a larger executable. Check actual member lengths independently.
if (@($payloadItems | Where-Object { -not $_.PSIsContainer -and $_.Length -gt 268435456 }).Count -ne 0) { throw 'Extracted installer payload member exceeds 256 MiB.' }
$expandedBytes = @($payloadItems | Where-Object { -not $_.PSIsContainer } | Measure-Object -Property Length -Sum).Sum
if ([long]$expandedBytes -gt 536870912) { throw 'Extracted installer payload exceeds the 512 MiB expanded-size bound.' }
if ($payloadList -cnotmatch ('(?m)^extracted_file_bytes=' + [long]$expandedBytes + '$')) { throw 'Independent extracted byte count differs from CI evidence.' }
$extractedMembers = @($payloadItems | Where-Object { -not $_.PSIsContainer } | ForEach-Object {
    [pscustomobject]@{ Path=$_.FullName.Substring($payloadDirectory.Length + 1).Replace('\', '/'); Size=$_.Length }
})
Assert-RequiredResourceMembers $extractedMembers $resourceSizes
foreach ($resourceName in $resourceNames) {
    $resourcePath = Join-Path $payloadDirectory $resourceName
    $extractedHash = (Get-FileHash -LiteralPath $resourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($extractedHash -cne [string]$manifest.bundle_resource_sha256.PSObject.Properties[$resourceName].Value) {
        throw "Extracted installer resource digest mismatch: $resourceName"
    }
}
[pscustomobject][ordered]@{
    status=$(if ($provenanceMode -ceq 'Main') { 'VERIFIED_NOT_INSTALLED' } else { 'VERIFIED_DIAGNOSTIC_NOT_INSTALLED' })
    provenance_mode=$provenanceMode
    diagnostic_pull_request=$(if ($provenanceMode -ceq 'DiagnosticPullRequest') { $DiagnosticPullRequestNumber } else { $null })
    verified_at_utc=(Get-Date).ToUniversalTime().ToString('o')
    classification=$manifest.classification
    source_revision=$ExpectedSourceRevision
    workflow_run_id=$ExpectedRunId
    workflow_run_attempt=$ExpectedRunAttempt
    zip_path=$zipFull
    zip_sha256=$actualZipSha256
    installer_path=$installerPath
    installer_bytes=$installerBytes.LongLength
    installer_sha256=$actualInstallerSha256
    authenticode_status=[string]$signature.Status
    extracted_payload_path=$payloadDirectory
    extracted_payload_entry_count=$payloadPreflight.EntryCount
    extracted_payload_bytes=[long]$expandedBytes
    required_resource_count=$resourceNames.Count
    aster_builder_resource_count=(Get-AsterBuilderResourceInventory).Count
    notice_inventory_status='STRUCTURALLY_VERIFIED_NO_REQUIRED_GAPS'
    manifest=$manifest
} | ConvertTo-Json -Depth 6
