# Clone a reviewed private repository into the requested Windows user profile.
# Run interactively on the user's Windows laptop after the GitHub repo exists.
# Git Credential Manager handles browser authentication and credential storage.

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repositoryUrl = 'https://github.com/comzulca/Aster-desktop.git'
$profileDirectory = $env:USERPROFILE
if ([string]::IsNullOrWhiteSpace($profileDirectory)) {
    throw 'USERPROFILE is not set. Start a normal Windows user session.'
}

$documentsDirectory = Join-Path $profileDirectory 'Documents'
$companyDirectory = Join-Path $documentsDirectory 'comzulca'
$destination = Join-Path $companyDirectory 'aster-desktop'

Get-Command git -ErrorAction Stop | Out-Null
& git credential-manager --version | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw 'Git Credential Manager is required. Install Git for Windows with its credential manager enabled.'
}

foreach ($directory in @($profileDirectory, $documentsDirectory, $companyDirectory)) {
    if (Test-Path -LiteralPath $directory) {
        $item = Get-Item -LiteralPath $directory -Force
        if (-not $item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw "Unexpected file or reparse point in checkout path: $directory"
        }
    } elseif ($directory -eq $companyDirectory) {
        New-Item -ItemType Directory -Path $directory | Out-Null
    } else {
        throw "Expected Windows profile directory does not exist: $directory"
    }
}

if (Test-Path -LiteralPath $destination) {
    throw "Destination already exists; inspect it manually before proceeding: $destination"
}

Write-Host "Cloning the private repository into: $destination"
# Apply GCM only to this command. Do not request a PAT or place credentials in
# a remote URL, script, environment variable, log, or app source file.
& git -c credential.helper=manager -c credential.credentialStore=wincredman clone --origin origin --branch main --single-branch $repositoryUrl $destination
if ($LASTEXITCODE -ne 0) {
    throw 'Clone failed. If Git created a partial destination, inspect it manually; this script does not delete files.'
}

$actualOrigin = & git -C $destination remote get-url origin
if ($LASTEXITCODE -ne 0 -or $actualOrigin.Trim() -ne $repositoryUrl) {
    throw 'The checked-out origin does not match the expected company repository.'
}

Write-Host "Checkout created: $destination"
Write-Host 'This script did not install, execute, or provision the application.'
