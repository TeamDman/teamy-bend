#Requires -Version 7.0
# SPDX-License-Identifier: MPL-2.0
param(
    [string]$SanitizerPath = '',
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$SanitizerArguments = @()
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Resolve-Sanitizer {
    $candidates = [Collections.Generic.List[string]]::new()
    if ($SanitizerPath) {
        if (Test-Path -LiteralPath $SanitizerPath -PathType Leaf) {
            $candidates.Add((Get-Item -LiteralPath $SanitizerPath).FullName)
        } else {
            $command = Get-Command -Name $SanitizerPath -CommandType Application -ErrorAction SilentlyContinue |
                Select-Object -First 1
            if ($command) { $candidates.Add($command.Source) }
        }
    } else {
        if ($env:CUDA_PATH) {
            $candidates.Add((Join-Path $env:CUDA_PATH 'compute-sanitizer/compute-sanitizer.exe'))
            $candidates.Add((Join-Path $env:CUDA_PATH 'bin/compute-sanitizer'))
        }
        foreach ($name in @('compute-sanitizer.exe', 'compute-sanitizer')) {
            foreach ($command in @(Get-Command -Name $name -CommandType Application -ErrorAction SilentlyContinue)) {
                $candidates.Add($command.Source)
            }
        }
    }
    foreach ($candidate in $candidates) {
        if (!(Test-Path -LiteralPath $candidate -PathType Leaf)) { continue }
        if ($IsWindows -and [IO.Path]::GetFileName($candidate) -ieq 'compute-sanitizer.bat') {
            # CUDA's bin directory can expose only a batch shim. Resolve its
            # adjacent native tool without invoking a shell or reparsing args.
            $candidate = Join-Path ([IO.Path]::GetDirectoryName($candidate)) '../compute-sanitizer/compute-sanitizer.exe'
            if (!(Test-Path -LiteralPath $candidate -PathType Leaf)) { continue }
        }
        if (!$IsWindows -or [IO.Path]::GetExtension($candidate) -ieq '.exe') {
            return (Get-Item -LiteralPath $candidate).FullName
        }
    }
    throw 'Compute Sanitizer executable not found. Set CUDA_PATH, add it to PATH, or pass -SanitizerPath.'
}

$start = [Diagnostics.ProcessStartInfo]::new((Resolve-Sanitizer))
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$start.WorkingDirectory = (Get-Location).ProviderPath
if ($IsWindows) {
    # NVIDIA documents this local frontend/target transport. Apply it only to
    # this tool and its children; do not change the caller or machine settings.
    $start.Environment['NV_COMPUTE_SANITIZER_LOCAL_CONNECTION_OVERRIDE'] = 'named-pipes'
}
foreach ($argument in $SanitizerArguments) { $start.ArgumentList.Add($argument) }

$process = [Diagnostics.Process]::new()
$process.StartInfo = $start
$started = $false
try {
    $started = $process.Start()
    if (!$started) { throw 'Compute Sanitizer did not start' }
    # Drain both pipes concurrently as bytes, preserving native output without
    # PowerShell text conversion or blocking one pipe behind the other.
    $standardOutput = [Console]::OpenStandardOutput()
    $standardError = [Console]::OpenStandardError()
    $outputCopy = $process.StandardOutput.BaseStream.CopyToAsync($standardOutput)
    $errorCopy = $process.StandardError.BaseStream.CopyToAsync($standardError)
    $process.WaitForExit()
    [Threading.Tasks.Task]::WhenAll([Threading.Tasks.Task[]]@($outputCopy, $errorCopy)).GetAwaiter().GetResult()
    $standardOutput.Flush()
    $standardError.Flush()
    $status = $process.ExitCode
} finally {
    if ($started -and !$process.HasExited) {
        $process.Kill($true)
        [void]$process.WaitForExit(5000)
    }
    $process.Dispose()
}
exit $status
