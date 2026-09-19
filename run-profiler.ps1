param(
	[Parameter(ValueFromRemainingArguments = $true)]
	[string[]]$QueryArgs
)

if (-not $QueryArgs -or $QueryArgs.Count -eq 0) {
	$QueryArgs = @("check", (Join-Path $PSScriptRoot "examples/laws.bend"))
}

$profiler = Get-Command teamy-profiler -ErrorAction SilentlyContinue
if (-not $profiler) {
	throw "teamy-profiler not found in PATH"
}

& $profiler.Source run cargo `
	--project $PSScriptRoot `
	--bin teamy-bend `
	--profile release `
	--feature extended_observability `
	--feature tracy `
	-- @QueryArgs
exit $LASTEXITCODE
