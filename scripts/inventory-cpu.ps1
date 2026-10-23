# SPDX-License-Identifier: MPL-2.0
# Read-only fixture inventory. Candidate lanes mirror the pinned upstream gate;
# they are not proof of applicability, successful execution or CPU safety.
param(
    [Parameter(Mandatory = $true)][string]$Reference,
    [Parameter(Mandatory = $true)][string]$OutputDirectory
)
$ErrorActionPreference = 'Stop'
$Reference = (Resolve-Path $Reference).Path
$pin = 'e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
if ((git -C $Reference rev-parse HEAD) -ne $pin -or $LASTEXITCODE -ne 0) {
    throw 'Unexpected reference revision.'
}
$status = git -C $Reference status --porcelain=v1
if ($LASTEXITCODE -ne 0 -or $status) { throw 'Reference checkout must be clean.' }
$outputPath = [IO.Path]::GetFullPath($OutputDirectory)
if ($outputPath -eq $Reference -or $outputPath.StartsWith("$Reference$([IO.Path]::DirectorySeparatorChar)", [StringComparison]::Ordinal)) {
    throw 'Inventory must stay outside the reference checkout.'
}
if (Test-Path -LiteralPath $outputPath) { throw 'Choose a new output directory.' }
$null = New-Item -ItemType Directory -Path $outputPath
$tests = Join-Path $Reference 'tests'
$files = @(Get-ChildItem -LiteralPath $tests -Recurse -File -Filter '*.bend')
$paths = [string[]]@($files | ForEach-Object { [IO.Path]::GetRelativePath($tests, $_.FullName).Replace('\', '/') })
[Array]::Sort($paths, [StringComparer]::Ordinal)
$rows = foreach ($relative in $paths) {
    $path = Join-Path $tests $relative
    if ((Get-Item -LiteralPath $path).LinkType) { throw "Inventory requires an ordinary source file: $relative" }
    $source = [IO.File]::ReadAllText($path)
    $expectedLines = @([regex]::Matches($source, '(?m)^#\|([^\r\n]*)') | ForEach-Object { $_.Groups[1].Value })
    if ($expectedLines.Count -eq 0) { throw "Missing fixture expectation: $relative" }
    $expected = $expectedLines -join "`n"
    $expectationHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($expected))).ToLowerInvariant()
    $base = [regex]::IsMatch($source, '(?m)^import Base\r?$')
    $main = [regex]::IsMatch($source, '(?m)^(def|law) main(\(|:)')
    $errorExpected = $expected.Trim().StartsWith('Error:', [StringComparison]::Ordinal)
    $companions = @([regex]::Matches($source, '(?m)^\s*import "\./[a-z0-9_]+\.(c|js)"\r?$') | ForEach-Object { $_.Groups[1].Value })
    $exitMarkers = @([regex]::Matches($expected, '(?m)^exit (\d+)$') | ForEach-Object { $_.Groups[1].Value })
    $hints = @()
    if ([regex]::IsMatch($source, '(?m)^\s*import "')) { $hints += 'foreign-import' }
    if ($source -match '\b(File|Socket|Tcp|Udp|Http|IO\.get_env)\b') { $hints += 'host-effect' }
    if ($source -match '\b(Window|Audio|App|Gfx)\b') { $hints += 'device-review' }
    if ($source -match '!\(') { $hints += 'offload-review' }
    [pscustomobject][ordered]@{
        fixture = $relative
        source_sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        expectation_sha256 = $expectationHash
        has_base = $base; has_main = $main; expects_check_error = $errorExpected
        exit_marker = $exitMarkers -join ';'
        candidate_js = $main -and -not $errorExpected -and $base -and ($companions.Count -eq 0 -or $companions -contains 'js')
        candidate_c = $main -and -not $errorExpected -and $base -and ($companions.Count -eq 0 -or $companions -contains 'c')
        review_hints = $hints -join ';'
    }
}
$rows | Export-Csv -LiteralPath (Join-Path $outputPath 'cpu-parity-inventory.csv') -NoTypeInformation -UseQuotes AsNeeded -Encoding utf8NoBOM
$summary = [ordered]@{
    schema = 1; upstream = $pin
    fixture_count = $rows.Count
    expected_checker_errors = @($rows | Where-Object expects_check_error).Count
    with_main = @($rows | Where-Object has_main).Count
    candidate_js = @($rows | Where-Object candidate_js).Count
    candidate_c = @($rows | Where-Object candidate_c).Count
    upstream_gate_sha256 = (Get-FileHash -LiteralPath (Join-Path $Reference 'gates/test.ts') -Algorithm SHA256).Hash.ToLowerInvariant()
    inventory_sha256 = (Get-FileHash -LiteralPath (Join-Path $outputPath 'cpu-parity-inventory.csv') -Algorithm SHA256).Hash.ToLowerInvariant()
    qualification = 'Inventory only. Every unlinked/unreviewed lane remains missing evidence; hints never waive a lane.'
}
$summary | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $outputPath 'cpu-parity-inventory.json') -Encoding utf8NoBOM
Write-Host "Inventoried $($rows.Count) pinned fixtures; no Bend program executed."
