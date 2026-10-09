#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidateSet('metadata','source','frontend','rust','audit','build')][string]$Stage,
    [Parameter(Mandatory)][string]$RuntimeRoot,
    [Parameter(Mandatory)][string]$RustBin,
    [Parameter(Mandatory)][string]$CargoCache,
    [string]$AuditExecutable,
    [string]$AdvisoryDatabase,
    [string]$BuildConfig
)
# Local-only engineering checks. No app, installer, real key or provider call.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
Set-Location -LiteralPath $root
$node = Join-Path $RuntimeRoot 'node/bin/node.exe'
$pnpm = Join-Path $RuntimeRoot 'bin/fallback/pnpm.cmd'
$python = Join-Path $RuntimeRoot 'python/python.exe'
$cargo = Join-Path $RustBin 'cargo.exe'
$gitBin = 'C:\Program Files\Git\cmd'
$env:PATH = (Join-Path $RuntimeRoot 'node/bin') + ';' + (Join-Path $RuntimeRoot 'bin/fallback') + ';' + $RustBin + ';' + $gitBin + ';' + $env:PATH
$env:CARGO_HOME = $CargoCache
$env:RUSTC = Join-Path $RustBin 'rustc.exe'
$env:CARGO_NET_OFFLINE = 'true'
$env:CARGO_INCREMENTAL = '0'
$env:ASTER_DISABLE_LIVE = '1'
$env:GIT_CONFIG_COUNT = '1'
$env:GIT_CONFIG_KEY_0 = 'safe.directory'
$env:GIT_CONFIG_VALUE_0 = $root.Replace('\','/')
$reports = Join-Path $root 'reports'
[void](New-Item -ItemType Directory -Path $reports -Force)
if ((Get-Item -LiteralPath $reports -Force).Attributes -band [System.IO.FileAttributes]::ReparsePoint) { throw 'Reports must not be a reparse point.' }
$runId = $Stage + '-' + (Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssfffZ')
$runDirectory = Join-Path $reports $runId
[void](New-Item -ItemType Directory -Path $runDirectory)
$head = (& git rev-parse HEAD).Trim()
$tree = (& git rev-parse 'HEAD^{tree}').Trim()
$started = (Get-Date).ToUniversalTime().ToString('o')
$records = [System.Collections.Generic.List[object]]::new()
function Invoke-Recorded([string]$Name, [string]$Executable, [string[]]$Arguments, [string]$JsonOutput = '') {
    $at = (Get-Date).ToUniversalTime().ToString('o')
    $log = Join-Path $runDirectory ($Name + '.txt')
    Write-Output ('Running ' + $Name)
    if ($JsonOutput) {
        & $Executable @Arguments 2> $log | Out-File -LiteralPath (Join-Path $root $JsonOutput) -Encoding utf8NoBOM
    } else {
        & $Executable @Arguments 2>&1 | Tee-Object -FilePath $log
    }
    $code = $LASTEXITCODE
    $records.Add([ordered]@{name=$Name; executable=$Executable; arguments=$Arguments; started_at_utc=$at; completed_at_utc=(Get-Date).ToUniversalTime().ToString('o'); exit_code=$code; log=$log; generated_output=$JsonOutput})
    if ($code -ne 0) { throw ('Check failed: ' + $Name + ', exit ' + $code) }
}
try {
    switch ($Stage) {
        'metadata' {
            Invoke-Recorded 'notices-self-test' $python @('-B','scripts/ci/third_party_notices.py','--self-test')
            Invoke-Recorded 'cargo-metadata' $cargo @('metadata','--offline','--locked','--manifest-path','src-tauri/Cargo.toml','--format-version','1') 'reports/packaging-metadata.json'
            Invoke-Recorded 'cargo-metadata-windows' $cargo @('metadata','--offline','--locked','--manifest-path','src-tauri/Cargo.toml','--format-version','1','--filter-platform','x86_64-pc-windows-msvc') 'reports/packaging-metadata-windows.json'
            Invoke-Recorded 'notices' $python @('-B','scripts/ci/third_party_notices.py','reports/packaging-metadata.json','reports/packaging-metadata-windows.json','.','reports/THIRD-PARTY-NOTICES.txt','reports/third-party-notice-inventory.json','x86_64-pc-windows-msvc')
            Invoke-Recorded 'rust-sbom' $python @('-B','scripts/ci/cargo_sbom.py','reports/packaging-metadata.json','reports/rust.cdx.json','reports/rust-licenses.json')
            Invoke-Recorded 'frontend-licenses' $pnpm @('licenses','list','--json') 'reports/frontend-licenses.json'
            Invoke-Recorded 'frontend-sbom' $pnpm @('sbom','--sbom-format','cyclonedx') 'reports/frontend.cdx.json'
        }
        'source' {
            Invoke-Recorded 'secrets-self-test' $python @('-B','scripts/ci/scan_secrets.py','--self-test')
            Invoke-Recorded 'secrets' $python @('-B','scripts/ci/scan_secrets.py','--include-untracked')
            Invoke-Recorded 'compatibility-self-test' $python @('-I','scripts/ci/check_tauri_versions.py','--self-test')
            Invoke-Recorded 'compatibility' $python @('-I','scripts/ci/check_tauri_versions.py','--root','.')
            Invoke-Recorded 'whitespace' 'git.exe' @('diff','--check')
            Invoke-Recorded 'source-manifest' $python @('-B','scripts/ci/publish_preflight.py','.') ('reports/' + $runId + '/source-manifest.json')
        }
        'frontend' {
            Invoke-Recorded 'typecheck' $pnpm @('typecheck')
            Invoke-Recorded 'vitest' $pnpm @('test')
            Invoke-Recorded 'vite-build' $pnpm @('build')
        }
        'rust' {
            Invoke-Recorded 'format' $cargo @('fmt','--manifest-path','src-tauri/Cargo.toml','--','--check')
            Invoke-Recorded 'check' $cargo @('check','--offline','--locked','--manifest-path','src-tauri/Cargo.toml','--all-targets')
            Invoke-Recorded 'clippy' $cargo @('clippy','--offline','--locked','--manifest-path','src-tauri/Cargo.toml','--all-targets','--','-D','warnings')
            Invoke-Recorded 'tests' $cargo @('test','--offline','--locked','--manifest-path','src-tauri/Cargo.toml')
        }
        'audit' {
            if (-not $AuditExecutable -or -not $AdvisoryDatabase) { throw 'Supply an independently fetched RustSec snapshot and auditor.' }
            Invoke-Recorded 'pnpm-audit' $pnpm @('audit','--audit-level','moderate')
            Invoke-Recorded 'rustsec-audit' $AuditExecutable @('audit','--db',$AdvisoryDatabase,'--no-fetch','--file','src-tauri/Cargo.lock')
        }
        'build' {
            # Locked dependencies are already installed. The invocation does not
            # sign or execute the resulting NSIS installer or GUI application.
            $buildArguments = @('exec','tauri','build','--ci','--no-sign')
            if ($BuildConfig) {
                $configPath = [System.IO.Path]::GetFullPath((Join-Path $root $BuildConfig))
                if (-not $configPath.StartsWith($reports + '\',[System.StringComparison]::OrdinalIgnoreCase)) { throw 'Local build overlay must be under reports.' }
                $configItem = Get-Item -LiteralPath $configPath -Force
                if ($configItem.PSIsContainer -or $configItem.Length -gt 1024 -or ($configItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint)) { throw 'Invalid local build overlay file.' }
                $overlay = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
                if (@($overlay.PSObject.Properties).Count -ne 1 -or @($overlay.build.PSObject.Properties).Count -ne 1 -or $overlay.build.beforeBuildCommand -cne 'pnpm build') { throw 'Only the cached pnpm build hook may be overridden.' }
                $buildArguments += @('--config',$configPath)
            }
            $buildArguments += @('--','--locked','--offline')
            Invoke-Recorded 'tauri-nsis' $pnpm $buildArguments
        }
    }
} finally {
    $receipt = [ordered]@{classification='local-engineering-checks-not-installed';stage=$Stage;head_at_start=$head;tree_at_start=$tree;source_binding='HEAD plus any explicit working-tree changes; final qualification requires the recorded clean source commit.';principal=[System.Security.Principal.WindowsIdentity]::GetCurrent().Name;os=[System.Environment]::OSVersion.VersionString;started_at_utc=$started;completed_at_utc=(Get-Date).ToUniversalTime().ToString('o');app_or_installer_executed=$false;live_meta_called=$false;checks=@($records)}
    $receipt | ConvertTo-Json -Depth 8 | Out-File -LiteralPath (Join-Path $runDirectory 'receipt.json') -Encoding utf8NoBOM
    Write-Output ('Receipt: ' + $runDirectory)
}
