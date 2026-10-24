# Reflexivity diagnostics with normalized values and Bend origins

Tested source `f317911160b3f752b05f9ff869a41e000e691c7c` closes the recorded false-brace,
wrong-reverse and lost-duplicate diagnostic gaps against upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. The original authored negative
fixtures are unchanged. All still fail checking before execution. No acceptance,
evaluation, demand, resource-budget or cancellation policy changed.

## Cause and bounded correction

The pinned [reflexivity rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3571)
throws a structured error with the endpoints, context, definition and proof
span. Its [error printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1437)
normalizes the endpoints and context types and prints a one-line source window
on either side. The port previously flattened that information into a symbolic
message, then the CLI rendered host context instead of the Bend location.

On an already-rejected reflexivity check, the port now prepares optional
normalized presentation metadata using a clone of the current engine. It keeps
local aliases and uses only the remaining fuel and nesting allowance; it never
resets a budget. Endpoints and context share that remaining allowance. Failure
to prepare the diagnostic retains the original rejection and host diagnostic.
It cannot accept a false proof or replace it with a success report.

An optional loader-owned source map retains each original reflexivity node and
source snapshot. Existing substitution and flattening preserve those leaf
identities. Retaining the allocation prevents identity reuse; no pointer is
dereferenced and no global registry, source-text guessing or public AST field
was added. Imported files, template instances, multiple proof nodes, CRLF and
an unterminated final line have exact source-location regressions.

The check/eval/run CLI boundary emits the structured language diagnostic to
stderr, flushes it and returns exit status 1. Existing tracing retains the
internal symbolic error at debug level. Ordinary host errors, absent origins,
unavailable normalized presentation, write/flush errors and cancellation stay
errors. Text/JSON/CSV output settings never wrap these failure streams.

## Exact observations and API boundary

| Original authored control | Normalized expected / observed | Bend location |
| --- | --- | --- |
| wrong-reverse | `[3, 2, 1]` / `[3, 1, 2]` | rev, line 63 |
| lost-duplicate | `[1, 2, 3, 3]` / `[1, 2, 3]` | srt, line 75 |
| false-brace | `3` / `4` | refl, line 9 |

Each original runs twice through pinned interpretation and port run/check/eval:
**24 exact rejection observations**, exit 1, empty stdout and byte-identical
stderr. Six structured upstream probes independently confirm checking phase,
reflexivity, definition, endpoints and source line; their exit 0 reports an
observed rejection, not proof acceptance. Six adjacent cases add **48 exact
rejection observations**. All these repetitions are deterministic and finish
within the 60-second deadline.

The unchanged no-main list original has six successful checking observations;
its port check JSON report is validated separately from the upstream/run plain
text. Ten named list/brace/bitwise definitions add **60 matching normal-form,
strict-report and actual-core observations**. They do not establish generated
execution of proof values or returned functions. The combined qualification
receipt has **144 observations**, exit 0.

`check_book` and `check_executable` still return `Err(KernelError)` with their
original Display/message and equality behavior. KernelError remains Send+Sync;
the optional metadata contains no checker engine or Rc. Its pre-existing private
field construction restriction remains. Public Book/DefDecl/Term layouts are
unchanged.

For Rust CLI consumers, these presentable source failures now return
`Ok(CliOutput)` with `exit_code() == 1` from CheckArgs/EvalArgs/RunArgs.invoke.
The repository's top-level consumer already reads that code before emitting
output. External consumers must do likewise; Ok alone is not proof acceptance.
Other checker/parse failures and compile/batch/serve diagnostic surfaces are not
qualified by this change. The earlier run_main runtime-exit API caveat remains
unchanged.

## Validation at f317911

| Check | Result |
| --- | --- |
| Standard Linux gate | **846 passed, 0 failed, 49 ignored**; exit 0 |
| Focused integration tests | **44 passed** |
| Internal diagnostic tests | **7 passed** |
| Internal depth/work guards | **5 passed** |
| Strict all-target/all-feature Clippy | Passed after two initial lints were corrected without suppressions |
| Complete checking audit | 1,302 fixtures; zero status transitions from 0f83aea; all 439 checker negatives rejected; 366 other accepted and 497 other rejected |
| 42-case normalization matrix, twice | Exact structure and text pass; exit 0 |
| Five-fixture CPU baseline, twice | 50 exact executions; exit 0 |
| Bitwise/brace execution renewal | Available generated/native lanes match twice; original direct-IO host block remains |
| Six runtime-negative fixtures, twice | 44 exact, eight resource refusals, two native foreign gaps, two Bun host blocks, four C non-applicable observations; exit 1 |
| Ten approved-demand controls, twice | 92 exact, six approved wildcard exceptions, two unresolved competing-error observations; exit 1 |

The full gate includes eight new tests: four integration tests and four internal
ones. The audit checks statuses only; no other diagnostic row becomes exact
because it rejects. No guard, assertion, ignore, budget, compiler warning policy
or demand contract was weakened. The existing 49 ignores are 47 CUDA checks and
two optional profilers. GPU, desktop and new Windows/macOS validation were unrun.
Broader positive/comptime/effects and detailed error-order/F32.read receipts keep
their historical source identities.

Eleven of 81 runtime rows, all 38 interpretation rows, ten demand controls,
three checking controls and the list checking row now reference f317911. The
other 70 runtime rows and other checking/diagnostic receipts remain historical.
The inventory union remains **119 originals**; no authored control increases it.
The 1,183 unlinked runtime/interpretation entries retain their applicability
split: 439 checker negatives, twelve checking-only, 153 interpreter and 579
compiled candidates. The [acceptance criterion](cpu-parity-acceptance.md) remains
unchanged; full CPU parity is incomplete.

## Later imported-literal follow-up

Source `67cdcdf` [fixes the imported Nat literal acceptance defect](cpu-imported-nat-cons.md)
and renews the 144 diagnostic observations without changing the original controls.
Pinned literal sugar uses fixed global Zero/Succ names; the old port incorrectly
resolved them as local constructors. The original exploratory program is now
rejected before reflexivity for the same invalid literal. Constructor/parse
stderr remains unqualified, not silently waived. Production invoke caller audit
finds no dropped exit code; external downstream callers remain unverified. The
historical receipts and observations below retain their original source identity.

## Retained non-passing exploratory evidence

A malformed template control using angle brackets fails parsing in both
implementations and has unqualified raw diagnostics. The corrected tilde-binder
control reaches reflexivity and matches; the malformed attempt remains saved.

A separate imported local Nat control uses `0n` and `1n` against helper.Nat.
Pinned upstream rejects its unqualified literal constructors while the port
reaches the false reflexivity check using helper.Zero/helper.Succ. Both exit 1,
but the causes and diagnostics differ. This is an unresolved imported-literal
qualification gap, not an exact match or a new policy exception. The adjacent
origin regression uses explicit constructors so it isolates source mapping.

A large computed endpoint probe reaches upstream's host stack overflow and the
port's nesting refusal before establishing a useful reflexivity presentation
comparison. It is not fallback-success evidence. One exploratory runner initially
asserted raw determinism, but the upstream stack-trace frames varied. That failed
attempt and its bytes are retained; a fresh receipt records variability rather
than requiring a stable host crash trace. The internal regression separately
proves that presentation exhaustion retains the original reflexivity failure.

Existing resource, foreign/Bun, F32.read, competing-error, other diagnostic,
loader/API/platform and wider-corpus requirements stay open. Next independent
small family: parse/cons_operator and parse/cons_plus_pattern, establishing actual
printer applicability before execution. The imported-literal gap is another
recorded follow-up; this checkpoint does not silently choose a new contract.

## Reproduction and headroom

Use the existing activation with CARGO_NET_OFFLINE=true and BEND_GPU=off. All
comparison destinations must be fresh. Exact argv, file hashes, raw streams,
exit codes, deadlines, determinism and child-exit records are in the private
cpu-reflexivity-diagnostics evidence directory; no machine-specific captures are
published here.

```sh
cargo +nightly-2026-05-24 fmt --all -- --check
cargo test --locked --features extended_observability --test cli_checker_diagnostics --test cli_proofs --test cpu_bitwise_braces --test kernel_core --test kernel_collections --quiet
cargo test --locked --features extended_observability --lib diagnostic --quiet
cargo test --locked --features extended_observability --lib depth_tests --quiet
cargo clippy --locked --all-targets --all-features -- -D warnings
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
python3 <private-evidence>/qualify.py
python3 <private-evidence>/audit.py
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <programs> -Suite parse-bitwise-braces -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <baseline> -Suite baseline -Repetitions 2
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <normalization> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <negatives> -Suite negatives -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <demand> -Suite demand -Repetitions 2
```

C comparisons retain the actual pinned O3 CLI profile, -lpthread -lm,
--threads 1 --gpu off, and strict port warnings. No new library or toolchain was
installed and no reference source was edited. Cargo.lock remains frozen.

Repeated build growth is attributable to root integration-test executables and
inactive feature-specific library/example variants. The 12-GiB prebuild reserve
was enforced before the full gate, which ran before new snapshots. It completed
in 194.542 seconds; minimum headroom was
3.01 GiB workspace and 4.45 GiB temporary
space, above the 2/1-GiB stop floors. No disk exhaustion occurred. After final
validation, workspace headroom was 10.17 GiB.
Only inactive rebuildable target products were reclaimed; all source and
historical evidence remain intact.

An early focused-build shell lacked fail-fast behavior and continued after a
preflight ended approximately 36 MiB below reserve. That mistake and passing
build log are retained. Subsequent shells stop on preflight failure: a later
54-MiB shortfall correctly prevented the build until an inactive unnamed example
variant was reclaimed. This is recorded as runner history, not hidden by the
final pass. Original negative fixture bytes are preserved, including the brace
control's pre-existing final blank line.

| Receipt/input | SHA256 |
| --- | --- |
| qualification/receipt.json | `92588196a22b7c63b4076b2a390db8b8f8bde869b55f03a794fea5188ac807fc` |
| audit/receipt.json | `0cfc612381fd822e04d0e688c1bd6166356dba9c7a57410e5d4d5df7e3e56905` |
| audit/transitions.json | `a607dc5c470049df175c9660ece402c6ac3daabce99b6e4e78f64f3e00542baa` |
| parse-bitwise-braces/receipt.json | `529340fe12722d035ab83bd3d27723390557624569616680365e6564a0d047e5` |
| baseline/receipt.json | `da03fb404dd313253388ecb926e1aceb1dc89d503dbb6a18a7953dff79a2255e` |
| normalization/receipt.json | `328612eeb954533671187eb9d637a1263b02d9acfd7abceade05704115874dca` |
| negatives/receipt.json | `3db934f02ead3c0068bf4c55122a0089db3289426f33332a4151997096baf507` |
| demand/receipt.json | `a62300520548f773dade1325593a86f35d66101008fb1af0056c86466fb9e1f8` |
| exploratory-final/receipt.json | `955de4f02de0f28942a137f8c2784bc1070d01d4027bcf4a11c1f1b259673e68` |
| final-gate-monitor.json | `55e56319e9bde26fed0999938ec24ab17d51d5e5b2f8a66439325d1b24595944` |
| focused-monitor.json | `60d963060229497d5cf5619ad71e6fe9744a827de2129317d1e40606279a8e54` |
| binary | `471dcc68c07e055a4e2172d7c82555fb3fde16a9601311e8bd6594a88bf4d3f4` |
| normalization probe | `a2a110a923a0456b44cc8ce9cd81e6457c83dac59d5427ce623546fbf29eb5b6` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
