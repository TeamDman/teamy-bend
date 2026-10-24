# SPDX-License-Identifier: MPL-2.0
# Interpreter-only comparison: preserve raw bytes and compare parsed core structure.
param(
    [Parameter(Mandatory = $true)][string]$Reference,
    [Parameter(Mandatory = $true)][string]$OutputDirectory,
    [ValidateRange(1, 3)][int]$Repetitions = 2,
    [ValidateSet('operator', 'recursion', 'companions', 'base-functions', 'nat-fuel', 'rewrite', 'float-suffix', 'u32-suffix', 'brace-suffix', 'nat-literal', 'default-nat', 'spacing-functions', 'paren-functions', 'layout', 'all')][string]$Suite = 'all',
    [switch]$RequireExactText
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

$names = @()
if ($Suite -in @('operator', 'all')) {
    $names += @('computed_operator_match_002', 'computed_operator_match_002_dbl', 'computed_operator_match_002_sel')
}
if ($Suite -in @('recursion', 'all')) {
    $names += @('computed_recursion_002', 'computed_recursion_003', 'computed_recursion_004')
}
if ($Suite -in @('companions', 'all')) {
    $names += @('computed_recursion_002_sel', 'computed_recursion_003_flip', 'computed_recursion_004_sel')
}
$cases = @($names | ForEach-Object { @{ name = $_; fixture = "flatten/$_.bend"; original = Join-Path $Reference "tests/flatten/$_.bend"; origin = 'upstream'; function_printer = $false } })
if ($Suite -in @('base-functions', 'all')) {
    foreach ($name in @('computed_recursion_001_main_dec', 'computed_recursion_001_not', 'computed_recursion_001_par')) {
        $cases += @{ name = $name; fixture = "flatten/$name.bend"; original = Join-Path $Reference "tests/flatten/$name.bend"; origin = 'upstream'; function_printer = $true }
    }
}
if ($Suite -in @('nat-fuel', 'all')) {
    foreach ($name in @('computed_recursion_000', 'computed_recursion_000_big', 'computed_recursion_001')) {
        $cases += @{ name = $name; fixture = "flatten/$name.bend"; original = Join-Path $Reference "tests/flatten/$name.bend"; origin = 'upstream'; function_printer = $true }
    }
}
if ($Suite -in @('rewrite', 'all')) {
    foreach ($fixture in @('eval/stuck_rewrite_print.bend', 'proof/context_rewrite_hypothesis.bend', 'proof/context_rewrite_value.bend')) {
        $cases += @{ name = [IO.Path]::GetFileNameWithoutExtension($fixture); fixture = $fixture; original = Join-Path $Reference "tests/$fixture"; origin = 'upstream'; function_printer = ($fixture -eq 'eval/stuck_rewrite_print.bend') }
    }
    foreach ($name in @('unnamed', 'named', 'shadow', 'nested')) {
        $cases += @{ name = "rewrite-$name"; fixture = "authored/rewrite-$name.bend"; original = Join-Path $port "tests/fixtures/cpu-rewrite/$name.bend"; origin = 'authored'; function_printer = $false }
    }
}
if ($Suite -in @('float-suffix', 'all')) {
    foreach ($name in @('chain_suffix_f32_f', 'chain_suffix_f32_g', 'chain_suffix_f32_h')) {
        $cases += @{ name = $name; fixture = "parse/$name.bend"; original = Join-Path $Reference "tests/parse/$name.bend"; origin = 'upstream'; function_printer = $true; executable_function = $true }
    }
}
if ($Suite -in @('u32-suffix', 'all')) {
    foreach ($name in @('chain_suffix_u32_f', 'chain_suffix_u32_g', 'chain_suffix_u32_h')) {
        $cases += @{ name = $name; fixture = "parse/$name.bend"; original = Join-Path $Reference "tests/parse/$name.bend"; origin = 'upstream'; function_printer = $true }
    }
}
if ($Suite -in @('brace-suffix', 'all')) {
    foreach ($name in @('chain_brace_suffix_f', 'chain_brace_suffix_g', 'chain_brace_suffix_h')) {
        $cases += @{ name = $name; fixture = "parse/$name.bend"; original = Join-Path $Reference "tests/parse/$name.bend"; origin = 'upstream'; function_printer = $true }
    }
}
if ($Suite -in @('nat-literal', 'all')) {
    $cases += @{ name = 'nat_literal_roundtrip'; fixture = 'eval/nat_literal_roundtrip.bend'; original = Join-Path $Reference 'tests/eval/nat_literal_roundtrip.bend'; origin = 'upstream'; function_printer = $true }
}
if ($Suite -in @('default-nat', 'all')) {
    foreach ($name in @('chain_default_nat_f', 'chain_default_nat_g')) {
        $cases += @{ name = $name; fixture = "parse/$name.bend"; original = Join-Path $Reference "tests/parse/$name.bend"; origin = 'upstream'; function_printer = $true }
    }
}
if ($Suite -in @('spacing-functions', 'all')) {
    foreach ($name in @('chain_spacing_rules_f', 'chain_spacing_rules_g', 'chain_spacing_rules_h')) {
        $cases += @{ name = $name; fixture = "parse/$name.bend"; original = Join-Path $Reference "tests/parse/$name.bend"; origin = 'upstream'; function_printer = $true }
    }
}
if ($Suite -in @('paren-functions', 'layout', 'all')) {
    foreach ($name in @('chain_paren_group_f', 'chain_paren_group_g', 'chain_paren_group_h', 'chain_paren_group_k', 'chain_paren_group_m')) {
        $cases += @{ name = $name; fixture = "parse/$name.bend"; original = Join-Path $Reference "tests/parse/$name.bend"; origin = 'upstream'; function_printer = $true }
    }
}
if ($Suite -in @('layout', 'all')) {
    foreach ($name in @('match_indent_column', 'ctor_row_layout', 'case_body_newline', 'tab_character')) {
        $cases += @{ name = $name; fixture = "parse/$name.bend"; original = Join-Path $Reference "tests/parse/$name.bend"; origin = 'upstream'; no_base_printer = $true }
    }
    foreach ($name in @('if_branch_newline', 'elif_alignment', 'do_line_depth')) {
        $cases += @{ name = "$($name)_pure"; fixture = "authored/$($name)_pure.bend"; original = Join-Path $port "tests/fixtures/cpu-layout/$($name)_pure.bend"; origin = 'authored'; function_printer = $false }
    }
}
foreach ($case in $cases) {
    $name = $case.name
    $caseDirectory = (New-Item -ItemType Directory -Path (Join-Path $outputPath $name)).FullName
    $original = $case.original
    $executableFunction = [bool]$case.executable_function
    $source = Join-Path $caseDirectory 'main.bend'
    Copy-Item -LiteralPath $original -Destination $source
    $golden = (@([regex]::Matches([IO.File]::ReadAllText($source), '(?m)^#\|([^\r\n]*)') | ForEach-Object { $_.Groups[1].Value }) -join "`n") + "`n"
    $row = [ordered]@{
        fixture = $case.fixture; origin = $case.origin; source_sha256 = (Get-FileHash $source -Algorithm SHA256).Hash.ToLowerInvariant()
        reference = $head; port_revision = $revision; port_dirty = [bool]$dirty
        port_binary_sha256 = $binaryHash; probe_binary_sha256 = $probeHash
        timeout_seconds = 60; repetitions = $Repetitions; require_exact_text = [bool]$RequireExactText; steps = [ordered]@{}
        structural_match = $false; exact_text_match = $false; deterministic = $null
        structure_source = if ($executableFunction) { 'parsed_executable_output_with_core_regression' } else { 'checked_kernel_term' }
        port_eval_applicability = if ($executableFunction) { 'strict_proof_exclusion' } else { 'required' }
        compiled_lanes = if ($case.function_printer) { 'unverified_function_printer_refusal' } elseif ($case.origin -eq 'authored') { 'authored_interpreter_control' } else { 'not_selected_by_upstream_gate_no_Base' }
    }
    $rows.Add($row)
    try {
        for ($repeat = 1; $repeat -le $Repetitions; $repeat++) {
            # Establish compiled-printer applicability before comparing values.
            if ($case.function_printer) {
                foreach ($target in @('c', 'js')) {
                    $outFile = Join-Path $caseDirectory "unprintable-$repeat.$target"
                    $refusal = Invoke-Bounded "upstream-printer-$target-$repeat" $node @('main.ts', $source, '-o', $outFile) (Join-Path $Reference 'bend2') -AllowFailure
                    $diagnostic = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($refusal.stderr_base64))
                    if ($refusal.exit -ne 1 -or $refusal.stdout_base64 -cne '' -or $diagnostic -notmatch 'cannot be printed \(a function, a Type, an erased or dependent field\)' -or (Test-Path -LiteralPath $outFile)) {
                        throw 'Upstream function-printer refusal did not match the reviewed contract.'
                    }
                }
                $row.compiled_lanes = 'not_required_upstream_function_printer_refusal'
            }
            if ($case.no_base_printer) {
                foreach ($target in @('c', 'js')) {
                    $outFile = Join-Path $caseDirectory "no-base-$repeat.$target"
                    $refusal = Invoke-Bounded "upstream-printer-$target-$repeat" $node @('main.ts', $source, '-o', $outFile) (Join-Path $Reference 'bend2') -AllowFailure
                    $diagnostic = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($refusal.stderr_base64))
                    if ($refusal.exit -ne 1 -or $refusal.stdout_base64 -cne '' -or $diagnostic -cne "Error: a build needs import Base`n" -or (Test-Path -LiteralPath $outFile)) {
                        throw 'Upstream no-Base build refusal did not match the reviewed contract.'
                    }
                }
                $row.compiled_lanes = 'not_required_upstream_no_Base_refusal'
            }
            $upstream = Invoke-Bounded "upstream-$repeat" $node @('main.ts', $source) (Join-Path $Reference 'bend2')
            $run = Invoke-Bounded "port-run-$repeat" $binary @('run', $source) $caseDirectory
            $eval = Invoke-Bounded "port-eval-$repeat" $binary @('--output-format', 'json', 'eval', $source) $caseDirectory -AllowFailure:$executableFunction
            $oracle = Join-Path $caseDirectory "upstream-$repeat.stdout"
            $comparisonArgs = @($source, $oracle)
            if ($executableFunction) { $comparisonArgs += '--executable-function' }
            $comparison = Invoke-Bounded "structure-$repeat" $probe $comparisonArgs $caseDirectory
            $successful = @($upstream, $run, $comparison)
            if ($executableFunction) {
                $diagnostic = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($eval.stderr_base64))
                if ($eval.exit -ne 1 -or $eval.stdout_base64 -cne '' -or -not $diagnostic.Contains('undefined name F32.add')) {
                    throw 'Strict proof-mode exclusion changed; it is not an executable parity result.'
                }
            } else { $successful += $eval }
            foreach ($result in $successful) {
                if ($result.stderr_base64 -cne '') { throw 'Unexpected stderr.' }
            }
            $text = { param($result) [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($result.stdout_base64)) }
            if ((& $text $upstream) -cne $golden) { throw 'Pinned interpreter differs from original expectation.' }
            $report = (& $text $comparison) | ConvertFrom-Json
            if (-not $executableFunction) {
                $value = (& $text $eval) | ConvertFrom-Json
                if ($value.entry -cne 'main' -or $value.value -cne $report.port_value) { throw 'Strict eval CLI contract differs.' }
            }
            if (-not $report.structural_match -or $report.core_term_observed -eq $executableFunction -or
                (& $text $run) -cne "$($report.port_value)`n" -or (& $text $upstream) -cne "$($report.upstream_value)`n") {
                throw 'Core structure or CLI contract differs.'
            }
            $row.structural_match = $true
            $row.exact_text_match = $report.exact_text_match
            if ($RequireExactText -and -not $row.exact_text_match) {
                throw 'Core structure matches but raw term text differs.'
            }
            if ($repeat -gt 1) {
                foreach ($prefix in @('upstream', 'port-run', 'port-eval', 'structure')) {
                    foreach ($field in @('exit', 'stdout_base64', 'stderr_base64')) {
                        if ($row.steps["$prefix-$repeat"][$field] -cne $row.steps["$prefix-1"][$field]) {
                            $row.deterministic = $false
                            throw 'Repeated observation changed.'
                        }
                    }
                }
                if ($case.function_printer) {
                    foreach ($target in @('c', 'js')) {
                        foreach ($field in @('exit', 'stdout_base64', 'stderr_base64')) {
                            if ($row.steps["upstream-printer-$target-$repeat"][$field] -cne $row.steps["upstream-printer-$target-1"][$field]) { throw 'Function-printer refusal changed between repetitions.' }
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
