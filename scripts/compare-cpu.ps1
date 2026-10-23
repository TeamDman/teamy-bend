# SPDX-License-Identifier: MPL-2.0
# Bounded Linux CPU execution comparison; never modifies the reference checkout.
param(
    [Parameter(Mandatory = $true)][string]$Reference,
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [string]$Compiler = $env:TEAMY_BEND_CC
)
$ErrorActionPreference = 'Stop'
if (-not $IsLinux) { throw 'This receipt qualifies the Linux POSIX C runtime.' }
if (-not $Compiler) { $Compiler = 'clang' }
$port = Split-Path $PSScriptRoot -Parent
$Reference = (Resolve-Path $Reference).Path
$expectedReference = 'e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
$referenceHead = git -C $Reference rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $referenceHead -ne $expectedReference) { throw 'Unexpected reference revision.' }
$referenceStatus = git -C $Reference status --porcelain=v1
if ($LASTEXITCODE -ne 0 -or $referenceStatus) { throw 'Reference checkout must be clean.' }
$portHead = git -C $port rev-parse HEAD
if ($LASTEXITCODE -ne 0) { throw 'Cannot identify the port revision.' }
$portStatus = git -C $port status --porcelain=v1
if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect the port checkout.' }
$outputPath = [IO.Path]::GetFullPath($OutputDirectory)
if ($outputPath -eq $Reference -or $outputPath.StartsWith("$Reference/", [StringComparison]::Ordinal)) {
    throw 'Comparison artifacts must stay outside the reference checkout.'
}
if (Test-Path $OutputDirectory) { throw 'Choose a new output directory to preserve prior evidence.' }
$OutputDirectory = (New-Item -ItemType Directory -Path $OutputDirectory).FullName
$binary = Join-Path $port 'target/debug/teamy-bend'
$node = if ($env:TEAMY_BEND_NODE) { $env:TEAMY_BEND_NODE } else { 'node' }
$rows = [System.Collections.Generic.List[object]]::new()

function Invoke-Bounded {
    param([string]$Label, [string]$Program, [string[]]$Arguments, [string]$Directory)
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
    try {
        [void]$process.Start()
        $outTask = $process.StandardOutput.BaseStream.CopyToAsync($stdout)
        $errTask = $process.StandardError.BaseStream.CopyToAsync($stderr)
        $timedOut = -not $process.WaitForExit(60000)
        if ($timedOut) { $process.Kill($true); $process.WaitForExit() }
        $outTask.GetAwaiter().GetResult()
        $errTask.GetAwaiter().GetResult()
        $outBytes = $stdout.ToArray()
        $errBytes = $stderr.ToArray()
        [System.IO.File]::WriteAllBytes((Join-Path $fixtureDirectory "$Label.stdout"), $outBytes)
        [System.IO.File]::WriteAllBytes((Join-Path $fixtureDirectory "$Label.stderr"), $errBytes)
        $result = [ordered]@{
            command = @($Program) + $Arguments; cwd = $Directory
            exit = $process.ExitCode; timed_out = $timedOut
            stdout_base64 = [Convert]::ToBase64String($outBytes)
            stderr_base64 = [Convert]::ToBase64String($errBytes)
        }
        $row.steps[$Label] = $result
        if ($timedOut -or $process.ExitCode -ne 0) { throw "$Label failed; inspect captured evidence." }
        return $result
    } finally {
        $process.Dispose(); $stdout.Dispose(); $stderr.Dispose()
    }
}

$fixtures = @(
    @{ name = 'drop-chain'; source = "$port/tests/fixtures/cpu-baseline/drop-chain.bend"; expected = "16`n" },
    @{ name = 'nested-layout'; source = "$port/tests/fixtures/cpu-baseline/nested-layout.bend"; expected = "17`n" },
    @{ name = 'word_arithmetic'; source = "$Reference/tests/run/word_arithmetic.bend"; expected = "63`n" },
    @{ name = 'array_boxed_get'; source = "$Reference/tests/run/array_boxed_get.bend"; expected = "777113`n" },
    @{ name = 'closures_hof'; source = "$Reference/tests/run/closures_hof.bend"; expected = "64180`n" }
)
foreach ($fixture in $fixtures) {
    $fixtureDirectory = (New-Item -ItemType Directory -Path (Join-Path $OutputDirectory $fixture.name)).FullName
    $source = Join-Path $fixtureDirectory 'main.bend'
    Copy-Item $fixture.source $source
    $row = [ordered]@{
        fixture = $fixture.name; source_sha256 = (Get-FileHash $source -Algorithm SHA256).Hash.ToLowerInvariant()
        reference = $referenceHead; port_revision = $portHead; port_dirty = [bool]$portStatus
        timeout_seconds = 60; steps = [ordered]@{}; matched = $false
    }
    $rows.Add($row)
    try {
        foreach ($target in @('c', 'js')) {
            $upstream = Join-Path $fixtureDirectory "upstream.$target"
            $generated = Join-Path $fixtureDirectory "port.$target"
            $null = Invoke-Bounded "upstream-generate-$target" $node @('main.ts', $source, '-o', $upstream) "$Reference/bend2"
            $portTarget = if ($target -eq 'js') { 'javascript' } else { 'c' }
            $null = Invoke-Bounded "port-generate-$target" $binary @('compile', '--executable', $source, '--target', $portTarget, '--output', $generated) $port
        }
        foreach ($implementation in @('upstream', 'port')) {
            $c = Join-Path $fixtureDirectory "$implementation.c"
            $exe = Join-Path $fixtureDirectory $implementation
            $flags = @('-std=c11')
            if ($implementation -eq 'port') { $flags += @('-Wall', '-Wextra', '-Werror', '-pedantic') }
            $null = Invoke-Bounded "$implementation-compile" $Compiler ($flags + @($c, '-pthread', '-lm', '-ldl', '-o', $exe)) $port
            $null = Invoke-Bounded "$implementation-c" $exe @() $fixtureDirectory
            $null = Invoke-Bounded "$implementation-js" $node @((Join-Path $fixtureDirectory "$implementation.js")) $fixtureDirectory
        }
        $null = Invoke-Bounded 'native' $binary @('run', $source) $port
        $expected = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($fixture.expected))
        foreach ($label in @('upstream-c', 'port-c', 'upstream-js', 'port-js', 'native')) {
            $result = $row.steps[$label]
            if ($result.stdout_base64 -cne $expected -or $result.stderr_base64 -cne '') {
                throw "$($fixture.name) $label differs from expected output."
            }
        }
        $row.matched = $true
        Write-Host "$($fixture.name): all five runtimes match stdout, stderr and exit status"
    } finally {
        ConvertTo-Json -InputObject $rows.ToArray() -Depth 12 | Set-Content -Encoding utf8 (Join-Path $OutputDirectory 'receipt.json')
    }
}
