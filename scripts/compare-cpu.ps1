# SPDX-License-Identifier: MPL-2.0
# Bounded Linux CPU execution comparison; never modifies the reference checkout.
param(
    [Parameter(Mandatory = $true)][string]$Reference,
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [string]$Compiler = $env:TEAMY_BEND_CC,
    [ValidateSet('O0', 'O1', 'O2', 'O3')][string]$OracleOptimization = 'O3',
    [ValidateSet('baseline', 'effects', 'control', 'collections', 'scheduler', 'words', 'representation', 'parse-chains', 'parse-operators', 'parse-floats', 'all')][string]$Suite = 'baseline',
    [ValidateRange(1, 3)][int]$Repetitions = 1
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
$builtBinary = Join-Path $port 'target/debug/teamy-bend'
$binary = Join-Path $OutputDirectory 'teamy-bend'
[IO.File]::Copy($builtBinary, $binary)
[IO.File]::SetUnixFileMode($binary, [IO.File]::GetUnixFileMode($builtBinary))
$binaryHash = (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash.ToLowerInvariant()
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
        [System.IO.File]::WriteAllBytes((Join-Path $fixtureDirectory "$Label.stdout"), $outBytes)
        [System.IO.File]::WriteAllBytes((Join-Path $fixtureDirectory "$Label.stderr"), $errBytes)
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

function Invoke-Execution {
    param([string]$Label, [string]$Program, [string[]]$Arguments, [switch]$AllowFailure)
    # Upstream's file fixtures use fixed absolute names. Never overwrite a
    # pre-existing file; execute serially and retain each newly created payload.
    $effectFile = $fixture.effect_file
    if ($effectFile -and (Get-Item -LiteralPath $effectFile -Force -ErrorAction SilentlyContinue)) {
        throw "Fixture path already exists; leave it intact: $effectFile"
    }
    $directory = (New-Item -ItemType Directory -Path (Join-Path $fixtureDirectory $Label)).FullName
    try {
        $result = Invoke-Bounded $Label $Program $Arguments $directory -AllowFailure:$AllowFailure
        if ($effectFile) {
            $bytes = [IO.File]::ReadAllBytes($effectFile)
            [IO.File]::WriteAllBytes((Join-Path $directory 'effect-file.bin'), $bytes)
            $result['artifact_base64'] = [Convert]::ToBase64String($bytes)
            $expectedFile = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($fixture.effect_contents))
            if ($result.artifact_base64 -cne $expectedFile) { throw "$Label file contents differ." }
        }
        return $result
    } finally {
        if ($effectFile -and (Test-Path -LiteralPath $effectFile -PathType Leaf)) {
            Remove-Item -LiteralPath $effectFile
        }
    }
}

$baseline = @(
    @{ name = 'drop-chain'; source = "$port/tests/fixtures/cpu-baseline/drop-chain.bend"; expected = "16`n" },
    @{ name = 'nested-layout'; source = "$port/tests/fixtures/cpu-baseline/nested-layout.bend"; expected = "17`n" },
    @{ name = 'word_arithmetic'; source = "$Reference/tests/run/word_arithmetic.bend"; expected = "63`n" },
    @{ name = 'array_boxed_get'; source = "$Reference/tests/run/array_boxed_get.bend"; expected = "777113`n" },
    @{ name = 'closures_hof'; source = "$Reference/tests/run/closures_hof.bend"; expected = "64180`n" }
)
$effects = @(
    @{ name = 'spawn_outlives_main'; expected = "main returns`nlate`n"; node_ffi_prefix = "main returns`n" },
    @{ name = 'channel_fork_join'; expected = "1122`n" },
    @{ name = 'chan_rendezvous'; expected = "m0`ns0`ns1`nm1`nm2`ns2`n" },
    @{ name = 'chan_pipe'; expected = "4`n3`n2`n1`n0`nend`n" },
    @{ name = 'file_roundtrip'; expected = "No such file or directory`nbend-file`n"; node_ffi_prefix = '';
       effect_file = '/tmp/bend_io_file.tmp'; effect_contents = 'bend-file' },
    @{ name = 'file_open_mode'; expected = "opened`nopened`nInvalid argument`nInvalid argument`n"; node_ffi_prefix = "opened`nopened`n";
       effect_file = '/tmp/bend_io_mode.txt'; effect_contents = '' }
)
foreach ($fixture in $effects) { $fixture.source = "$Reference/tests/io/$($fixture.name).bend" }
$control = @(
    @{ name = 'chan_close_send'; expected = "TrueFalse`n" },
    @{ name = 'fiber_single_join'; expected = ''; expected_exit = 1; expected_stderr = "IO.join: the channel was closed`n" },
    @{ name = 'halt_utf8_law'; expected = ''; expected_exit = 7; expected_stderr = "è❁`n" },
    @{ name = 'text_utf8'; expected = "eéé`n"; expected_exit = 7; expected_stderr = "é`né`n" },
    @{ name = 'deadlock'; expected = "waiting`n"; expected_exit = 1; expected_stderr = "bend: deadlock: every computation waits on a channel`n" }
)
foreach ($fixture in $control) { $fixture.source = "$Reference/tests/io/$($fixture.name).bend" }
$collections = @(
    @{ name = 'map_ops'; expected = "7`n" },
    @{ name = 'map_prefix_keys'; expected = "a=2,ab=1,b=3,.`na=20,ab=1,b=3,c=30,.`na=2,ab=1,b=3,c=30,.`nLTGTLT`n" },
    @{ name = 'map_set_array_kit'; expected = "3`n[1, 2, 3]`n2`n[x, y]`n[0, 5, 0, 7]`n[1, 6, 1, 8]`n" }
)
foreach ($fixture in $collections) { $fixture.source = "$Reference/tests/base/$($fixture.name).bend" }
$scheduler = @(
    @{ name = 'sleep_order'; source = "$Reference/tests/io/sleep_order.bend"; expected = "a`nb`nc`nd`n"; node_ffi_prefix = '' },
    @{ name = 'sleep_timer'; source = "$Reference/tests/io/sleep_timer.bend"; expected = "woke`n"; node_ffi_prefix = '' },
    @{ name = 'ready_before_timers'; source = "$port/tests/fixtures/cpu-scheduler/ready-before-timers.bend"; expected = "main`nready`na`nb`n"; node_ffi_prefix = "main`nready`n" }
)
# Assert ordering, not elapsed duration. Different-duration timers can all be
# overdue after host descheduling, when registration order takes precedence;
# such timing-dependent goldens (for example spawn_sleep) do not belong here.
$words = @(
    @{ name = 'word_wrap_chains'; source = "$Reference/tests/run/word_wrap_chains.bend"; expected = "4`n" },
    @{ name = 'word_div_mod'; source = "$Reference/tests/run/word_div_mod.bend"; expected = "9`n" },
    @{ name = 'word_compare'; source = "$Reference/tests/run/word_compare.bend"; expected = "6`n" },
    @{ name = 'word_bitwise_literals'; source = "$Reference/tests/run/word_bitwise_literals.bend"; expected = "11`n" },
    @{ name = 'bit_walking'; source = "$Reference/tests/run/bit_walking.bend"; expected = "5`n" },
    @{ name = 'u32_divmod_zero'; source = "$Reference/tests/base/u32_divmod_zero.bend"; expected = "6n`n" },
    @{ name = 'u32_divmod'; source = "$Reference/tests/base/u32_divmod.bend"; expected = "14n`n" }
)
$representation = @(
    @{ name = 'u32_mod_operator'; source = "$Reference/tests/run/u32_mod_operator.bend"; expected = "2 17 1.5`n" },
    @{ name = 'word_copy_native'; source = "$Reference/tests/reg/word_copy_native.bend"; expected = "22`n" },
    @{ name = 'u32_structural_view'; source = "$Reference/tests/reg/u32_structural_view.bend"; expected = "14`n" }
)
$parseChains = @(
    @{ name = 'adjacent_application'; source = "$Reference/tests/parse/adjacent_application.bend"; expected = "True{}`n"; direct_mode = 'pure' },
    @{ name = 'chain_paren_group'; source = "$Reference/tests/parse/chain_paren_group.bend"; expected = "7`n"; direct_mode = 'io' },
    @{ name = 'chain_spacing_rules'; source = "$Reference/tests/parse/chain_spacing_rules.bend"; expected = "2`n"; direct_mode = 'io' }
)
$parseOperators = @(
    @{ name = 'chain_brace_suffix'; source = "$Reference/tests/parse/chain_brace_suffix.bend"; expected = "5`n"; direct_mode = 'io' },
    @{ name = 'chain_default_nat'; source = "$Reference/tests/parse/chain_default_nat.bend"; expected = "7`n"; direct_mode = 'io' },
    @{ name = 'chain_suffix_u32'; source = "$Reference/tests/parse/chain_suffix_u32.bend"; expected = "16`n"; direct_mode = 'io' }
)
$parseFloats = @(
    @{ name = 'chain_suffix_f32'; source = "$Reference/tests/parse/chain_suffix_f32.bend"; expected = "7.5`n"; direct_mode = 'io' },
    @{ name = 'float_exponent'; source = "$Reference/tests/parse/float_exponent.bend"; expected = "0.0025 1000 150`n"; direct_mode = 'io' },
    @{ name = 'float_pattern'; source = "$Reference/tests/parse/float_pattern.bend"; expected = "1`n"; direct_mode = 'pure' },
    @{ name = 'float-bits-and-patterns'; source = "$port/tests/fixtures/cpu-parse-floats/float-bits-and-patterns.bend"; expected = "992204554`n1065353216`n1065353218`n1065353216`n10`n2147483648`n20`n1071644672`n7`n99`n1`n4294967295`n" }
)
$fixtures = switch ($Suite) {
    'baseline' { $baseline }
    'effects' { $effects }
    'control' { $control }
    'collections' { $collections }
    'scheduler' { $scheduler }
    'words' { $words }
    'representation' { $representation }
    'parse-chains' { $parseChains }
    'parse-operators' { $parseOperators }
    'parse-floats' { $parseFloats }
    'all' { $baseline + $effects + $control + $collections + $scheduler + $words + $representation + $parseChains + $parseOperators + $parseFloats }
}
foreach ($fixture in $fixtures) {
    if ($fixture.effect_file -and (Get-Item -LiteralPath $fixture.effect_file -Force -ErrorAction SilentlyContinue)) {
        throw "Fixture path already exists; leave it intact: $($fixture.effect_file)"
    }
}
foreach ($fixture in $fixtures) {
    $fixtureDirectory = (New-Item -ItemType Directory -Path (Join-Path $OutputDirectory $fixture.name)).FullName
    $source = Join-Path $fixtureDirectory 'main.bend'
    Copy-Item $fixture.source $source
    $row = [ordered]@{
        fixture = $fixture.name; source_sha256 = (Get-FileHash $source -Algorithm SHA256).Hash.ToLowerInvariant()
        reference = $referenceHead; port_revision = $portHead; port_dirty = [bool]$portStatus
        port_binary_sha256 = $binaryHash; c_arguments = @('--threads', '1', '--gpu', 'off'); upstream_c_optimization = $OracleOptimization
        upstream_c_link_flags = @('-lpthread', '-lm'); upstream_direct_cli = 'unverified'
        timeout_seconds = 60; repetitions = $Repetitions; steps = [ordered]@{}; matched = $false
        completed_runtimes_match = $false; deterministic = $null; environmental_limits = @()
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
            $links = @('-lpthread', '-lm')
            if ($implementation -eq 'upstream') { $flags += "-$OracleOptimization" }
            if ($implementation -eq 'port') {
                $flags += @('-Wall', '-Wextra', '-Werror', '-pedantic')
                $links = @('-pthread', '-lm', '-ldl')
            }
            $null = Invoke-Bounded "$implementation-compile" $Compiler ($flags + @($c) + $links + @('-o', $exe)) $port
        }
        $expected = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($fixture.expected))
        $expectedExit = if ($fixture.ContainsKey('expected_exit')) { $fixture.expected_exit } else { 0 }
        $expectedStderr = if ($fixture.ContainsKey('expected_stderr')) { $fixture.expected_stderr } else { '' }
        $expectedError = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($expectedStderr))
        $row['expected'] = @{ exit = $expectedExit; stdout_base64 = $expected; stderr_base64 = $expectedError }
        for ($repeat = 1; $repeat -le $Repetitions; $repeat++) {
            foreach ($runtime in @('upstream-c', 'port-c', 'upstream-js', 'port-js', 'native')) {
                $label = "$runtime-$repeat"
                $allowFfiFailure = $runtime -eq 'upstream-js' -and $fixture.ContainsKey('node_ffi_prefix')
                $allowFailure = $allowFfiFailure -or $expectedExit -ne 0
                if ($runtime -eq 'native') {
                    $result = Invoke-Execution $label $binary @('run', $source) -AllowFailure:$allowFailure
                } elseif ($runtime.EndsWith('-js')) {
                    $js = Join-Path $fixtureDirectory ($runtime.Replace('-js', '.js'))
                    $result = Invoke-Execution $label $node @($js) -AllowFailure:$allowFailure
                } else {
                    $exe = Join-Path $fixtureDirectory ($runtime.Replace('-c', ''))
                    $result = Invoke-Execution $label $exe @('--threads', '1', '--gpu', 'off') -AllowFailure:$allowFailure
                }
                if ($repeat -gt 1) {
                    $first = $row.steps["$runtime-1"]
                    foreach ($field in @('exit', 'stdout_base64', 'stderr_base64', 'artifact_base64')) {
                        if ($result[$field] -cne $first[$field]) {
                            $row.deterministic = $false
                            throw "$($fixture.name) $runtime changed $field between repetitions."
                        }
                    }
                }
                if ($allowFfiFailure -and $result.exit -eq 1) {
                    # Classify only this exact missing-host-FFI failure, including
                    # its partial stdout. Every other failure remains fatal.
                    $errorText = "Error: Cannot find module 'bun:ffi'`nRequire stack:`n- $js`n"
                    $errorBytes = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($errorText))
                    $prefix = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($fixture.node_ffi_prefix))
                    if ($result.stderr_base64 -ceq $errorBytes -and $result.stdout_base64 -ceq $prefix) {
                        $row.environmental_limits += $label
                        continue
                    }
                }
                if ($result.exit -ne $expectedExit -or $result.stdout_base64 -cne $expected -or $result.stderr_base64 -cne $expectedError) {
                    throw "$($fixture.name) $label differs from expected output."
                }
            }
            if ($fixture.ContainsKey('direct_mode')) {
                $label = "upstream-direct-$repeat"
                $direct = Invoke-Bounded $label $node @('main.ts', $source) "$Reference/bend2" -AllowFailure
                if ($repeat -gt 1) {
                    foreach ($field in @('exit', 'stdout_base64', 'stderr_base64')) {
                        if ($direct[$field] -cne $row.steps['upstream-direct-1'][$field]) { throw 'Direct upstream entry changed between repetitions.' }
                    }
                }
                if ($direct.exit -eq $expectedExit -and $direct.stdout_base64 -ceq $expected -and $direct.stderr_base64 -ceq $expectedError) {
                    $row.upstream_direct_cli = 'pass'
                } elseif ($fixture.direct_mode -eq 'io' -and $direct.exit -eq 1 -and $direct.stdout_base64 -ceq '' -and
                    $direct.stderr_base64 -ceq [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("TypeError: require is not a function`n"))) {
                    $row.upstream_direct_cli = 'blocked_import_meta_require'
                } else {
                    throw 'Unclassified direct upstream entry result.'
                }
            }
        }
        $row.completed_runtimes_match = $true
        if ($Repetitions -gt 1) { $row.deterministic = $true }
        $row.matched = $row.environmental_limits.Count -eq 0 -and $row.upstream_direct_cli -ne 'blocked_import_meta_require'
        if ($row.matched) {
            Write-Host "$($fixture.name): all five runtimes match, $Repetitions repetition(s)"
        } elseif ($row.upstream_direct_cli -eq 'blocked_import_meta_require') {
            Write-Host "$($fixture.name): all five generated/native runtimes match; direct upstream IO entry lacks import.meta.require ($Repetitions repetition(s))"
        } else {
            Write-Host "$($fixture.name): four runtimes match; upstream JS requires bun:ffi ($Repetitions repetition(s))"
        }
    } finally {
        ConvertTo-Json -InputObject $rows.ToArray() -Depth 12 | Set-Content -Encoding utf8 (Join-Path $OutputDirectory 'receipt.json')
    }
}
