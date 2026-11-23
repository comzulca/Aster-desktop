# Read only the reviewed, source-controlled pins. Never take expected hashes from an installer.
function Get-SpeechResourceInventory {
    $source = Join-Path $PSScriptRoot '../../src-tauri/speech-assets.json'
    $assets = Get-Content -LiteralPath $source -Raw -ErrorAction Stop | ConvertFrom-Json
    $inventory = [ordered]@{}
    foreach ($property in $assets.PSObject.Properties) {
        $leaf = $property.Name
        $asset = $property.Value
        if ($leaf -cnotmatch '^[a-zA-Z0-9][a-zA-Z0-9_.-]+\.(bin|dll|exe|txt)$' -or
            $asset.sha256 -cnotmatch '^[0-9a-f]{64}$' -or
            ($asset.bytes -isnot [int] -and $asset.bytes -isnot [long]) -or
            $asset.bytes -le 0 -or $asset.bytes -gt 134217728) { throw 'Invalid pinned speech resource.' }
        $inventory.Add(('speech/' + $leaf), @{ byte_count = [long]$asset.bytes; sha256 = $asset.sha256 })
    }
    if (-not $inventory.Contains('speech/whisper-cli.exe') -or
        -not $inventory.Contains('speech/ggml-base-q5_1.bin')) { throw 'Incomplete speech inventory.' }
    return $inventory
}
