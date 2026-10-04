# Passive, bounded local packaging audit. Never execute an app or installer.
# This report cannot qualify main-CI, final delivery or installed acceptance.
#requires -Version 5.1
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$RepositoryRoot,
    [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{40}$')][string]$SourceRevision,
    [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{40}$')][string]$SourceTree
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$started = (Get-Date).ToUniversalTime().ToString('o')
$root = [System.IO.Path]::GetFullPath($RepositoryRoot).TrimEnd('\','/')
$expectedRoot = 'C:\Users\Rene Nehru Alvarez\Documents\aster-desktop-premium\Aster-desktop-v0.2.2'
if ($root -cne $expectedRoot) { throw 'This isolated audit accepts only the explicitly assigned workspace.' }
$git = 'C:\Program Files\Git\cmd\git.exe'
$actualRevision = (& $git -c "safe.directory=$root" -C $root rev-parse HEAD).Trim()
$actualTree = (& $git -c "safe.directory=$root" -C $root rev-parse 'HEAD^{tree}').Trim()
$sourceChanges = @(& $git -c "safe.directory=$root" -C $root status --porcelain)
if ($LASTEXITCODE -ne 0 -or $actualRevision -cne $SourceRevision -or $actualTree -cne $SourceTree -or $sourceChanges.Count -ne 0) { throw 'Passive payload audit requires the exact clean local source commit.' }
$reports = Join-Path $root 'reports'
if ((Get-Item -LiteralPath $reports -Force).Attributes -band [System.IO.FileAttributes]::ReparsePoint) { throw 'Reports directory is a reparse point.' }
$installer = Join-Path $root 'src-tauri\target\release\bundle\nsis\Aster Desktop_0.2.2_x64-setup.exe'
$builtApp = Join-Path $root 'src-tauri\target\release\aster-desktop.exe'
$sevenZip = 'C:\Users\Rene Nehru Alvarez\Documents\aster-desktop-premium\engineering-tools\7zip-26.03-x64\portable\7z.exe'
$sevenZipHash = '6ee3c0ed0b27663c1b948ae85a7c0bb073aed1498983182f3f0df1f6a8c30b2f'
if ((Get-FileHash -LiteralPath $sevenZip -Algorithm SHA256).Hash.ToLowerInvariant() -cne $sevenZipHash) { throw 'The reviewed portable extractor bytes changed.' }
$names = @(
    '$PLUGINSDIR/modern-wizard.bmp',
    '$PLUGINSDIR/nsDialogs.dll',
    '$PLUGINSDIR/nsis_tauri_utils.dll',
    '$PLUGINSDIR/NSISdl.dll',
    '$PLUGINSDIR/StartMenu.dll',
    '$PLUGINSDIR/System.dll',
    'aster-desktop.exe', 'LICENSE.txt', 'THIRD-PARTY-NOTICES.txt', 'uninstall.exe'
)
$output = Join-Path $reports ('local-payload-v022-{0}.json' -f $SourceRevision.Substring(0,7))
if (Test-Path -LiteralPath $output) { throw 'Do not overwrite an existing source-bound audit report.' }
$setupItem = Get-Item -LiteralPath $installer -Force
if ($setupItem.PSIsContainer -or ($setupItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -or $setupItem.Length -le 0 -or $setupItem.Length -gt 67108864) { throw 'Installer must be an ordinary nonempty file at most64MiB.' }
# Keep one restrictive read handle throughout listing/extraction. No write or
# delete share can replace the bytes whose package digest we report.
$setupHandle = [System.IO.File]::Open($installer,[System.IO.FileMode]::Open,[System.IO.FileAccess]::Read,[System.IO.FileShare]::Read)
try {
    $installerHash = (Get-FileHash -InputStream $setupHandle -Algorithm SHA256).Hash.ToLowerInvariant()
    $listing = @(& $sevenZip l -slt -ba $installer)
    if ($LASTEXITCODE -ne 0) { throw '7-Zip advertised-payload listing failed.' }
    $seen = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
    $currentPath = $null
    $currentSize = $null
    [long]$advertisedBytes = 0
    foreach ($line in @($listing) + '') {
        if ([string]::IsNullOrWhiteSpace([string]$line)) {
            if ($null -ne $currentPath) {
                [long]$size = 0
                if ($null -eq $currentSize -or -not [long]::TryParse([string]$currentSize,[ref]$size) -or $size -lt 0 -or $size -gt 268435456) { throw 'Invalid/missing advertised member size or member exceeds256MiB.' }
                $normal = ([string]$currentPath).Replace('\','/')
                if ($normal.Length -eq 0 -or $normal.Length -gt 4096 -or [System.IO.Path]::IsPathRooted($normal) -or $normal.StartsWith('/') -or $normal -match ':' -or $normal -match '(^|/)\.{1,2}(/|$)' -or $normal -match '[\x00-\x1f]') { throw 'Unsafe advertised member path.' }
                if ($names -cnotcontains $normal -or -not $seen.Add($normal)) { throw 'Unexpected or duplicate advertised member.' }
                if ($seen.Count -gt 8192 -or $advertisedBytes -gt 536870912-$size) { throw 'Advertised count/aggregate size exceeds reviewed bounds.' }
                $advertisedBytes += $size
                $currentPath = $null
                $currentSize = $null
            }
            continue
        }
        if ([string]$line -clike 'Path = *') {
            if ($null -ne $currentPath) { throw 'Duplicate path field in listing entry.' }
            $currentPath = ([string]$line).Substring(7)
            continue
        }
        if ([string]$line -clike 'Size = *') {
            if ($null -ne $currentSize) { throw 'Duplicate size field in listing entry.' }
            $currentSize = ([string]$line).Substring(7)
            continue
        }
        if ([string]$line -clike 'Anti = +*' -or [string]$line -clike 'Symbolic Link = *' -or [string]$line -clike 'Hard Link = *' -or [string]$line -clike 'Folder = +*') { throw 'Advertised archive has a link, anti-item or unexpected directory member.' }
    }
    if ($seen.Count -ne $names.Count) { throw 'The advertised archive must have exactly the10reviewed members.' }
    # Extraction occurs only AFTER all advertised preflight bounds pass.
    $extract = Join-Path $reports ('local-payload-{0}-{1}' -f $SourceRevision.Substring(0,7),[Guid]::NewGuid().ToString('N'))
    if (Test-Path -LiteralPath $extract) { throw 'The fresh GUID extraction directory already exists.' }
    [void](New-Item -ItemType Directory -Path $extract -ErrorAction Stop)
    & $sevenZip x $installer "-o$extract" -y | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Passive installer-payload extraction failed.' }
    $items = @(Get-ChildItem -LiteralPath $extract -Recurse -Force)
    if (@($items | Where-Object { $_.Attributes -band [System.IO.FileAttributes]::ReparsePoint }).Count -ne 0) { throw 'Extracted payload has a reparse point.' }
    $files = @($items | Where-Object { -not $_.PSIsContainer })
    $directories = @($items | Where-Object { $_.PSIsContainer })
    if ($files.Count -ne 10 -or $directories.Count -ne 1 -or $directories[0].Name -cne '$PLUGINSDIR') { throw 'Unexpected extracted member/directory set.' }
    [long]$expandedBytes = 0
    $actualSeen = New-Object 'System.Collections.Generic.HashSet[string]' ([System.StringComparer]::OrdinalIgnoreCase)
    $members = @()
    foreach ($file in $files) {
        if (-not $file.FullName.StartsWith($extract+'\',[System.StringComparison]::OrdinalIgnoreCase)) { throw 'Extracted member escaped the explicitly allocated directory.' }
        $relative = $file.FullName.Substring($extract.Length+1).Replace('\','/')
        if ($names -cnotcontains $relative -or -not $actualSeen.Add($relative) -or $file.Length -gt 268435456 -or $expandedBytes -gt 536870912-$file.Length) { throw 'Actual member set/member/aggregate bound failed.' }
        $expandedBytes += $file.Length
        $members += [ordered]@{path=$relative; byte_count=$file.Length; sha256=(Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}
    }
    foreach ($resource in @(@{packaged='LICENSE.txt';source='LICENSE'},@{packaged='THIRD-PARTY-NOTICES.txt';source='reports\THIRD-PARTY-NOTICES.txt'})) {
        $sourceHash = (Get-FileHash -LiteralPath (Join-Path $root $resource.source) -Algorithm SHA256).Hash.ToLowerInvariant()
        $packaged = @($members | Where-Object {$_.path -ceq $resource.packaged})
        if ($packaged.Count -ne 1 -or $packaged[0].sha256 -cne $sourceHash) { throw 'A packaged licence/notice differs from its exact current source resource.' }
    }
    $app = Join-Path $extract 'aster-desktop.exe'
    $appInfo = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($app)
    if ($appInfo.ProductName -cne 'Aster Desktop' -or $appInfo.ProductVersion -cne '0.2.2') { throw 'Wrong packaged app product name/version.' }
    $appSignature = [string](Get-AuthenticodeSignature -LiteralPath $app).Status
    $setupSignature = [string](Get-AuthenticodeSignature -LiteralPath $installer).Status
    if ($appSignature -cne 'NotSigned' -or $setupSignature -cne 'NotSigned') { throw 'Unexpected installer or app Authenticode classification.' }
    [byte[]]$payloadBytes = [System.IO.File]::ReadAllBytes($app)
    if ($payloadBytes.Length -lt 256 -or $payloadBytes[0] -ne 0x4d -or $payloadBytes[1] -ne 0x5a) { throw 'Missing DOS/PE data header.' }
    $peOffset = [System.BitConverter]::ToInt32($payloadBytes,0x3c)
    if ($peOffset -lt 64 -or $peOffset -gt $payloadBytes.Length-96 -or [System.BitConverter]::ToUInt32($payloadBytes,$peOffset) -ne 0x00004550) { throw 'Invalid bounded PE header.' }
    $optional = $peOffset+24
    if ([System.BitConverter]::ToUInt16($payloadBytes,$peOffset+4) -ne 0x8664 -or [System.BitConverter]::ToUInt16($payloadBytes,$optional) -ne 0x20b -or [System.BitConverter]::ToUInt16($payloadBytes,$optional+68) -ne 2) { throw 'Application must be x64/PE32+/Windows GUI.' }
    $builtItem = Get-Item -LiteralPath $builtApp -Force
    if ($builtItem.PSIsContainer -or ($builtItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -or $builtItem.Length -ne $payloadBytes.Length -or $builtItem.Length -gt 268435456) { throw 'Unpatched local app is not a matching bounded ordinary file.' }
    [byte[]]$unpatchedBytes = [System.IO.File]::ReadAllBytes($builtApp)
    $plainMarker = '__TAURI_BUNDLE_TYPE_VAR_UNK'
    $nsisMarker = '__TAURI_BUNDLE_TYPE_VAR_NSS'
    $binaryText = [System.Text.Encoding]::ASCII.GetString($unpatchedBytes)
    $markerOffset = $binaryText.IndexOf($plainMarker,[System.StringComparison]::Ordinal)
    if ($markerOffset -lt 0 -or $binaryText.IndexOf($plainMarker,$markerOffset+1,[System.StringComparison]::Ordinal) -ge 0) { throw 'The exact unpatched Tauri marker must occur once only.' }
    [byte[]]$expectedNsis = $unpatchedBytes.Clone()
    [byte[]]$nsisToken = [System.Text.Encoding]::ASCII.GetBytes($nsisMarker)
    [System.Buffer]::BlockCopy($nsisToken,0,$expectedNsis,$markerOffset,$nsisToken.Length)
    if (-not [System.Collections.StructuralComparisons]::StructuralEqualityComparer.Equals($expectedNsis,$payloadBytes)) { throw 'Packaged app differs beyond the exact unique reviewed UNK-to-NSS bundle token.' }
    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try {
        $expectedNsisHash = [System.BitConverter]::ToString($algorithm.ComputeHash($expectedNsis)).Replace('-','').ToLowerInvariant()
        $appHash = [System.BitConverter]::ToString($algorithm.ComputeHash($payloadBytes)).Replace('-','').ToLowerInvariant()
        $unpatchedHash = [System.BitConverter]::ToString($algorithm.ComputeHash($unpatchedBytes)).Replace('-','').ToLowerInvariant()
    } finally { $algorithm.Dispose() }
    # System.Drawing reads icon resources as data; no app entry point executes.
    Add-Type -AssemblyName System.Drawing
    $packagedIcon = [System.Drawing.Icon]::ExtractAssociatedIcon($app)
    $sourceIcon = [System.Drawing.Icon]::new((Join-Path $root 'src-tauri\icons\icon.ico'),32,32)
    $packagedBitmap = $null
    $sourceBitmap = $null
    try {
        $packagedBitmap = $packagedIcon.ToBitmap()
        $sourceBitmap = $sourceIcon.ToBitmap()
        if ($packagedBitmap.Width -ne 32 -or $packagedBitmap.Height -ne 32 -or $sourceBitmap.Width -ne 32 -or $sourceBitmap.Height -ne 32) { throw 'Unexpected passive icon resource size.' }
        for ($y=0;$y -lt 32;$y++) {
            for ($x=0;$x -lt 32;$x++) {
                if ($packagedBitmap.GetPixel($x,$y).ToArgb() -ne $sourceBitmap.GetPixel($x,$y).ToArgb()) { throw 'Packaged32pxlauncher icon differs from the current committed icon.' }
            }
        }
    } finally {
        if ($null -ne $packagedBitmap) {$packagedBitmap.Dispose()}
        if ($null -ne $sourceBitmap) {$sourceBitmap.Dispose()}
        if ($null -ne $packagedIcon) {$packagedIcon.Dispose()}
        if ($null -ne $sourceIcon) {$sourceIcon.Dispose()}
    }
    $setupHandle.Position = 0
    if ((Get-FileHash -InputStream $setupHandle -Algorithm SHA256).Hash.ToLowerInvariant() -cne $installerHash) { throw 'The installer bytes changed during passive inspection.' }
    $report = [ordered]@{
        classification='locally-built-source-pinned-unsigned-non-final-package'
        result='PASS-local-bounded-payload-only'
        source_revision=$SourceRevision; source_tree=$SourceTree
        source_binding='Exact local source/tree independently matched to clean Git HEAD before inspection; no main-CI or installed qualification.'
        started_at_utc=$started; completed_at_utc=(Get-Date).ToUniversalTime().ToString('o')
        principal=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name
        installed=$false; delivered=$false; main_ci_verified=$false; app_or_installer_executed=$false
        extractor_sha256=$sevenZipHash
        installer_filename=$setupItem.Name; installer_bytes=$setupItem.Length; installer_sha256=$installerHash; installer_authenticode=$setupSignature
        app_authenticode=$appSignature; app_product=$appInfo.ProductName; app_product_version=$appInfo.ProductVersion
        app_bytes=$payloadBytes.Length; app_payload_sha256=$appHash; app_unpatched_sha256=$unpatchedHash; app_expected_nsis_sha256=$expectedNsisHash
        app_architecture='x64'; app_subsystem='Windows GUI'; tauri_marker_offset=$markerOffset
        tauri_marker_change='Exact unique UNK-to-NSS bundle-type token only; complete byte-array equality and SHA verify all other bytes.'
        preflight_entry_count=$seen.Count; preflight_bytes=$advertisedBytes
        expanded_file_count=$files.Count; expanded_bytes=$expandedBytes; extraction_root=$extract
        launcher_icon_pixels_match=$true; launcher_icon_size=32
        complete_payload_members=@($members | Sort-Object { $_.path })
    }
    $json = $report | ConvertTo-Json -Depth 5
    [System.IO.File]::WriteAllText($output,$json+[Environment]::NewLine,[System.Text.UTF8Encoding]::new($false))
    Write-Output $json
} finally { $setupHandle.Dispose() }
