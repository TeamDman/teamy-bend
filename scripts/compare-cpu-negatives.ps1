# SPDX-License-Identifier: MPL-2.0
# Bounded Linux qualification of six pinned runtime-negative fixtures.
# Exit 1 means required parity cells remain open, even for safe/known refusals.
param(
    [Parameter(Mandatory = $true)][string]$Reference,
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [string]$Compiler = $env:TEAMY_BEND_CC,
    [ValidateRange(1, 3)][int]$Repetitions = 2
)
$ErrorActionPreference = 'Stop'
if (-not $IsLinux) { throw 'This comparison qualifies the Linux POSIX CPU profile.' }
if (-not $Compiler) { $Compiler = 'clang' }
$port = Split-Path $PSScriptRoot -Parent
$Reference = (Resolve-Path $Reference).Path
$pin = 'e6676b080f25b1bc1bf5b5b7d7a17e22f8022599'
if ((git -C $Reference rev-parse HEAD) -ne $pin -or $LASTEXITCODE -ne 0) { throw 'Unexpected reference revision.' }
$status = git -C $Reference status --porcelain=v1
if ($LASTEXITCODE -ne 0 -or $status) { throw 'Reference checkout must be clean.' }
$outputPath = [IO.Path]::GetFullPath($OutputDirectory)
if ($outputPath -eq $Reference -or $outputPath.StartsWith("$Reference/", [StringComparison]::Ordinal)) { throw 'Evidence must stay outside the reference checkout.' }
if (Test-Path -LiteralPath $outputPath) { throw 'Choose a new evidence directory.' }
$OutputDirectory = (New-Item -ItemType Directory -Path $outputPath).FullName
$binary = Join-Path $OutputDirectory 'teamy-bend'
$built = Join-Path $port 'target/debug/teamy-bend'
[IO.File]::Copy($built, $binary)
[IO.File]::SetUnixFileMode($binary, [IO.File]::GetUnixFileMode($built))
$binaryHash = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
$revision = git -C $port rev-parse HEAD
$dirty = [bool](git -C $port status --porcelain=v1)
$node = if ($env:TEAMY_BEND_NODE) { $env:TEAMY_BEND_NODE } else { 'node' }
$rows = [System.Collections.Generic.List[object]]::new()
$openCells = 0

function Invoke-Bounded {
    param([string]$Label, [string]$Program, [string[]]$Arguments, [string]$Directory)
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $Program; $start.WorkingDirectory = $Directory
    $start.UseShellExecute = $false
    $start.RedirectStandardInput = $true; $start.RedirectStandardOutput = $true; $start.RedirectStandardError = $true
    $start.Environment['BEND_GPU'] = 'off'
    [void]$start.Environment.Remove('BEND_TEST_NO_SUCH_VAR_09AF')
    foreach ($argument in $Arguments) { $start.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::new(); $process.StartInfo = $start
    $stdout = [IO.MemoryStream]::new(); $stderr = [IO.MemoryStream]::new()
    $timer = [Diagnostics.Stopwatch]::StartNew()
    try {
        [void]$process.Start(); $process.StandardInput.Close()
        $outTask = $process.StandardOutput.BaseStream.CopyToAsync($stdout)
        $errTask = $process.StandardError.BaseStream.CopyToAsync($stderr)
        $timedOut = -not $process.WaitForExit(60000)
        if ($timedOut) { $process.Kill($true); $process.WaitForExit() }
        [void]$outTask.GetAwaiter().GetResult(); [void]$errTask.GetAwaiter().GetResult()
        $outBytes = $stdout.ToArray(); $errBytes = $stderr.ToArray()
        [IO.File]::WriteAllBytes((Join-Path $directoryRoot "$Label.stdout"), $outBytes)
        [IO.File]::WriteAllBytes((Join-Path $directoryRoot "$Label.stderr"), $errBytes)
        $result = [ordered]@{
            command = @($Program) + $Arguments; cwd = $Directory; exit = $process.ExitCode
            timed_out = $timedOut; elapsed_ms = $timer.ElapsedMilliseconds; child_exited = $process.HasExited
            stdout_base64 = [Convert]::ToBase64String($outBytes); stderr_base64 = [Convert]::ToBase64String($errBytes)
        }
        $row.steps[$Label] = $result
        return $result
    } finally { $process.Dispose(); $stdout.Dispose(); $stderr.Dispose() }
}
function Get-Outcome {
    param($Result, [string]$Runtime)
    if ($Result.timed_out) { return 'timeout' }
    $out = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($Result.stdout_base64))
    $err = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($Result.stderr_base64))
    if ($Result.exit -eq $fixture.exit -and $out -ceq $fixture.stdout -and $err -ceq $fixture.stderr) { return 'pass' }
    if ($Runtime -eq 'upstream-js' -and $fixture.name -eq 'io/stderr_failure') {
        $js = Join-Path $directoryRoot 'upstream.js'
        if ($Result.exit -eq 1 -and $out -ceq "to-stdout`n" -and $err -ceq "to-stderr`nError: Cannot find module 'bun:ffi'`nRequire stack:`n- $js`n") { return 'blocked_bun_ffi' }
    }
    if ($Result.exit -eq 1 -and $out -ceq $fixture.stdout) {
        if ($Runtime -eq 'native' -and $fixture.name -eq 'io/marshal_char_scalar' -and $err.Contains('native execution does not support the foreign implementation of echo')) { return 'unsupported_foreign' }
        if ($err.Contains('budget exhausted') -or $err.Contains('continuation depth exhausted')) { return 'safety_limit' }
        if ($fixture.message -and $err.Contains($fixture.message)) { return 'diagnostic_mismatch' }
        if ($fixture.name -eq 'io/request_out_of_band' -and $err.Contains('foreign effect request')) { return 'diagnostic_mismatch' }
        if ($fixture.name -eq 'io/request_out_of_band' -and $err.Contains('foreign request inspected')) { return 'diagnostic_mismatch' }
        if ($fixture.name -eq 'run/nat_overflow' -and $err.Contains('Nat multiplication overflow')) { return 'diagnostic_mismatch' }
    }
    return 'unexpected_result'
}
$fixtures = @(
    @{ name = 'io/array_depth_bound'; message = 'an array past the deepest block class 31'; stdout = ''; stderr = "bend: an array past the deepest block class 31`n"; exit = 1 },
    @{ name = 'io/marshal_char_scalar'; message = '55296 is not a Unicode scalar value'; stdout = ''; stderr = "bend: 55296 is not a Unicode scalar value`n"; exit = 1; companion = 'marshal_char_scalar.js' },
    @{ name = 'io/request_out_of_band'; message = 'runtime fail-stop'; stdout = "3`n"; stderr = "bend: runtime fail-stop`n"; exit = 1 },
    @{ name = 'io/stack_fault_trap'; stdout = ''; stderr = "bend: memory fault (machine stack overflow?)`n"; exit = 1 },
    @{ name = 'io/stderr_failure'; stdout = "to-stdout`n"; stderr = "to-stderr`nNo such file or directory`n"; exit = 2 },
    @{ name = 'run/nat_overflow'; message = 'a Nat past the largest immediate 2^48-1'; stdout = ''; stderr = "bend: a Nat past the largest immediate 2^48-1`n"; exit = 1 }
)
foreach ($fixture in $fixtures) {
    $directoryRoot = (New-Item -ItemType Directory -Path (Join-Path $OutputDirectory ($fixture.name.Split('/')[-1]))).FullName
    $original = Join-Path $Reference "tests/$($fixture.name).bend"
    $source = Join-Path $directoryRoot 'main.bend'; Copy-Item -LiteralPath $original -Destination $source
    $row = [ordered]@{
        fixture = "$($fixture.name).bend"; source_sha256 = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()
        reference = $pin; port_revision = $revision; port_dirty = $dirty; port_binary_sha256 = $binaryHash
        timeout_seconds = 60; repetitions = $Repetitions; c_arguments = @('--threads', '1', '--gpu', 'off')
        expected = @{ exit = $fixture.exit; stdout = $fixture.stdout; stderr = $fixture.stderr }
        steps = [ordered]@{}; lanes = [ordered]@{}; deterministic = $true
    }
    $rows.Add($row)
    try {
        if ($fixture.companion) {
            $companion = Join-Path (Split-Path $original -Parent) $fixture.companion
            Copy-Item -LiteralPath $companion -Destination $directoryRoot
            $row['companion_sha256'] = (Get-FileHash -LiteralPath $companion -Algorithm SHA256).Hash.ToLowerInvariant()
            $row.lanes['upstream-c'] = 'not_required_js_only_companion'; $row.lanes['port-c'] = 'not_required_js_only_companion'
        }
        foreach ($runtime in @('upstream-c', 'port-c', 'upstream-js', 'port-js', 'native')) {
            if ($row.lanes.Contains($runtime)) { continue }
            $program = $binary; $arguments = @('run', $source)
            if ($runtime -ne 'native') {
                $implementation, $target = $runtime.Split('-')
                $output = Join-Path $directoryRoot "$implementation.$target"
                $generator = $binary; $cwd = $port
                $portTarget = if ($target -eq 'c') { 'c' } else { 'javascript' }
                $generationArgs = @('compile', '--executable', $source, '--target', $portTarget, '--output', $output)
                if ($implementation -eq 'upstream') { $generator = $node; $cwd = "$Reference/bend2"; $generationArgs = @('main.ts', $source, '-o', $output) }
                $result = Invoke-Bounded "$implementation-generate-$target" $generator $generationArgs $cwd
                if ($result.exit -ne 0 -or $result.timed_out) { $row.lanes[$runtime] = 'generation_failed'; $openCells++; continue }
                $program = $node; $arguments = @($output)
                if ($target -eq 'c') {
                    $program = Join-Path $directoryRoot $implementation
                    $flags = @('-std=c11')
                    if ($implementation -eq 'port') { $flags += @('-Wall', '-Wextra', '-Werror', '-pedantic') }
                    $result = Invoke-Bounded "$implementation-compile" $Compiler ($flags + @($output, '-pthread', '-lm', '-ldl', '-o', $program)) $port
                    if ($result.exit -ne 0 -or $result.timed_out) { $row.lanes[$runtime] = 'compilation_failed'; $openCells++; continue }
                    $arguments = @('--threads', '1', '--gpu', 'off')
                }
            }
            for ($repeat = 1; $repeat -le $Repetitions; $repeat++) {
                $label = "$runtime-$repeat"
                $cwd = (New-Item -ItemType Directory -Path (Join-Path $directoryRoot $label)).FullName
                $result = Invoke-Bounded $label $program $arguments $cwd
                $result['remaining_files'] = @(Get-ChildItem -LiteralPath $cwd -Force | ForEach-Object Name)
                $result['outcome'] = Get-Outcome $result $runtime
                $result['phase'] = switch ($result.outcome) {
                    'unsupported_foreign' { 'native_foreign_support_check' }
                    'unexpected_result' { 'unclassified' }
                    default { 'execution' }
                }
                if ($repeat -gt 1) {
                    $first = $row.steps["$runtime-1"]
                    foreach ($field in @('exit', 'stdout_base64', 'stderr_base64')) {
                        if ($first[$field] -cne $result[$field]) { $row.deterministic = $false; $result.outcome = 'nondeterministic' }
                    }
                }
                if ($result.remaining_files.Count -ne 0) { $result.outcome = 'unexpected_artifact' }
                $row.lanes[$runtime] = $result.outcome
                if ($result.outcome -ne 'pass') { $openCells++ }
            }
        }
        Write-Host "$($row.fixture): $($row.lanes | ConvertTo-Json -Compress)"
    } finally {
        ConvertTo-Json -InputObject $rows.ToArray() -Depth 12 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'receipt.json') -Encoding utf8NoBOM
    }
}
if ($openCells -ne 0) { Write-Host "$openCells required observations remain unqualified; inspect receipt.json."; exit 1 }
