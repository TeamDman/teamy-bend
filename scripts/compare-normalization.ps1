# SPDX-License-Identifier: MPL-2.0
# Interpreter-only comparison: preserve raw bytes and compare parsed core structure.
param(
    [Parameter(Mandatory = $true)][string]$Reference,
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [ValidateRange(1, 3)][int]$Repetitions = 2
)
$ErrorActionPreference = 'Stop'
$port = Split-Path $PSScriptRoot -Parent
$Reference = (Resolve-Path $Reference).Path
$pin = 'e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
$head = git -C $Reference rev-parse HEAD
$status = git -C $Reference status --porcelain=v1
if ($LASTEXITCODE -ne 0 -or $head -ne $pin -or $status) { throw 'Reference must be clean at the pin.' }
$revision = git -C $port rev-parse HEAD
if ($LASTEXITCODE -ne 0) { throw 'Cannot identify port source.' }
$dirty = git -C $port status --porcelain=v1
if ($LASTEXITCODE -ne 0) { throw 'Cannot identify source status.' }
$outputPath = [IO.Path]::GetFullPath($OutputDirectory)
$separator = [IO.Path]::DirectorySeparatorChar
if ($outputPath -eq $Reference -or $outputPath.StartsWith("$Reference$separator", [StringComparison]::Ordinal)) {
    throw 'Evidence must stay outside the reference checkout.'
}
if (Test-Path -LiteralPath $outputPath) { throw 'Choose a fresh evidence directory.' }
$null = New-Item -ItemType Directory -Path $outputPath
$extension = if ($IsWindows) { '.exe' } else { '' }
$binary = Join-Path $outputPath "teamy-bend$extension"
$probe = Join-Path $outputPath "compare_normalization$extension"
foreach ($pair in @(@("target/debug/teamy-bend$extension", $binary), @("target/debug/examples/compare_normalization$extension", $probe))) {
    $source = Join-Path $port $pair[0]
    [IO.File]::Copy($source, $pair[1])
    if (-not $IsWindows) { [IO.File]::SetUnixFileMode($pair[1], [IO.File]::GetUnixFileMode($source)) }
}
$binaryHash = (Get-FileHash $binary -Algorithm SHA256).Hash.ToLowerInvariant()
$probeHash = (Get-FileHash $probe -Algorithm SHA256).Hash.ToLowerInvariant()
$node = if ($env:TEAMY_BEND_NODE) { $env:TEAMY_BEND_NODE } else { 'node' }
$rows = [System.Collections.Generic.List[object]]::new()
function Invoke-Bounded {
    param([string]$Label, [string]$Program, [string[]]$Arguments, [string]$Directory, [switch]$AllowFailure)
    $start = [System.Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $Program
    $start.WorkingDirectory = $Directory
    $start.UseShellExecute = $false
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.Environment['BEND_GPU'] = 'off'
    foreach ($argument in $Arguments) { $start.ArgumentList.Add($argument) }
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $start
    $stdout = [System.IO.MemoryStream]::new()
    $stderr = [System.IO.MemoryStream]::new()
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        [void]$process.Start()
        $outTask = $process.StandardOutput.BaseStream.CopyToAsync($stdout)
        $errTask = $process.StandardError.BaseStream.CopyToAsync($stderr)
        $timedOut = -not $process.WaitForExit(60000)
        if ($timedOut) { $process.Kill($true); $process.WaitForExit() }
        [void]$outTask.GetAwaiter().GetResult()
        [void]$errTask.GetAwaiter().GetResult()
        $outBytes = $stdout.ToArray()
        $errBytes = $stderr.ToArray()
        [System.IO.File]::WriteAllBytes((Join-Path $caseDirectory "$Label.stdout"), $outBytes)
        [System.IO.File]::WriteAllBytes((Join-Path $caseDirectory "$Label.stderr"), $errBytes)
        $result = [ordered]@{
            command = @($Program) + $Arguments; cwd = $Directory
            exit = $process.ExitCode; timed_out = $timedOut
            elapsed_ms = $timer.ElapsedMilliseconds
            stdout_base64 = [Convert]::ToBase64String($outBytes)
            stderr_base64 = [Convert]::ToBase64String($errBytes)
        }
        $row.steps[$Label] = $result
        if ($timedOut -or ($process.ExitCode -ne 0 -and -not $AllowFailure)) { throw "$Label failed; inspect captured evidence." }
        return $result
    } finally {
        $process.Dispose(); $stdout.Dispose(); $stderr.Dispose()
    }
}

foreach ($name in @('computed_operator_match_002', 'computed_operator_match_002_dbl', 'computed_operator_match_002_sel')) {
    $caseDirectory = (New-Item -ItemType Directory -Path (Join-Path $outputPath $name)).FullName
    $original = Join-Path $Reference "tests/flatten/$name.bend"
    $source = Join-Path $caseDirectory 'main.bend'
    Copy-Item -LiteralPath $original -Destination $source
    $golden = (@([regex]::Matches([IO.File]::ReadAllText($source), '(?m)^#\|([^\r\n]*)') | ForEach-Object { $_.Groups[1].Value }) -join "`n") + "`n"
    $row = [ordered]@{
        fixture = "flatten/$name.bend"; source_sha256 = (Get-FileHash $source -Algorithm SHA256).Hash.ToLowerInvariant()
        reference = $head; port_revision = $revision; port_dirty = [bool]$dirty
        port_binary_sha256 = $binaryHash; probe_binary_sha256 = $probeHash
        timeout_seconds = 60; repetitions = $Repetitions; steps = [ordered]@{}
        structural_match = $false; exact_text_match = $false; deterministic = $null
        compiled_lanes = 'not_selected_by_upstream_gate_no_Base'
    }
    $rows.Add($row)
    try {
        for ($repeat = 1; $repeat -le $Repetitions; $repeat++) {
            $upstream = Invoke-Bounded "upstream-$repeat" $node @('main.ts', $source) (Join-Path $Reference 'bend2')
            $run = Invoke-Bounded "port-run-$repeat" $binary @('run', $source) $caseDirectory
            $eval = Invoke-Bounded "port-eval-$repeat" $binary @('--output-format', 'json', 'eval', $source) $caseDirectory
            $oracle = Join-Path $caseDirectory "upstream-$repeat.stdout"
            $comparison = Invoke-Bounded "structure-$repeat" $probe @($source, $oracle) $caseDirectory
            foreach ($result in @($upstream, $run, $eval, $comparison)) {
                if ($result.stderr_base64 -cne '') { throw 'Unexpected stderr.' }
            }
            $text = { param($result) [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($result.stdout_base64)) }
            if ((& $text $upstream) -cne $golden) { throw 'Pinned interpreter differs from original expectation.' }
            $report = (& $text $comparison) | ConvertFrom-Json
            $value = (& $text $eval) | ConvertFrom-Json
            if (-not $report.structural_match -or $value.entry -cne 'main' -or $value.value -cne $report.port_value -or
                (& $text $run) -cne "$($report.port_value)`n" -or (& $text $upstream) -cne "$($report.upstream_value)`n") {
                throw 'Core structure or CLI contract differs.'
            }
            $row.structural_match = $true
            $row.exact_text_match = $report.exact_text_match
            if ($repeat -gt 1) {
                foreach ($prefix in @('upstream', 'port-run', 'port-eval', 'structure')) {
                    foreach ($field in @('exit', 'stdout_base64', 'stderr_base64')) {
                        if ($row.steps["$prefix-$repeat"][$field] -cne $row.steps["$prefix-1"][$field]) {
                            $row.deterministic = $false
                            throw 'Repeated observation changed.'
                        }
                    }
                }
            }
        }
        if ($Repetitions -gt 1) { $row.deterministic = $true }
        Write-Host "$name : structure matches; exact text = $($row.exact_text_match)"
    } finally {
        ConvertTo-Json -InputObject $rows.ToArray() -Depth 15 | Set-Content -Encoding utf8 (Join-Path $outputPath 'receipt.json')
    }
}
