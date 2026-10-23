# Function-valued main normalization checkpoint

Candidate `f1bbf98ac5c5c42b16ee6746991b1d3c04d1013c` fixes entry interpretation
for three unchanged `flatten/computed_operator_match_002*.bend` fixtures against
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
All three now agree structurally with the actual interpreter, twice. The first
also matches raw output; two retain **presentation-only differences**, recorded
as unresolved rendering contracts rather than exact-byte passes. CPU parity is
not complete and no pending demand choice is implemented.

## Defect and correction

All three programs already passed strict checking. Before the correction,
`eval` exited 1 with `main needs 1 arguments; got 0`; `run` exited 1 with
`main must have no declared parameters`. Actual upstream exited 0 and printed
the normalized function. The preflight retains two repetitions of each command,
including complete original stderr; those diagnostics are not published here
because they include host paths.

Pinned [book_run](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L454)
normalizes the stored `main.v` for a pure entry. It does not call a function
without its arguments. [term_snf](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3033)
normalizes lambda bodies and match arms while preserving recursive references.
The port incorrectly used its complete-application API in `eval`, and prohibited
declared parameters in `run` before interpretation.

The additive `CheckedBook::normalize_definition` API resets the existing engine
and normalizes the stored body. Pure `run` uses the same internal operation.
Whole-type IO detection remains unchanged: a function returning IO is a function
value, not an IO action. The existing normalizer, display implementation,
fuel/depth limits, native arithmetic and effect driver are unchanged.

`evaluate`, `evaluate_data`, batch and serve retain complete-argument checking.
`ExecutableBook` still cannot provide proof normalization through the new public
API; a compile-fail regression enforces that distinction. Regression controls
cover wrong argument types/arity, unknown definitions, complete application,
function-valued IO without execution, cancellation and unsafe recursive
normalization refusing at the existing limit before any output.

This corrects observable entry behavior: consumers that expected parameterized
`entry_kind` to fail now receive `Pure`. It does not relax argument-taking APIs.
The previously documented `KernelError` private-field compatibility concern and
`run_main` returning `Ok(1)` for a completed failing Bend program remain explicit;
external consumers have not been inventoried by this checkpoint.

## Exact results and comparison boundary

Each displayed term has one final LF in raw `run`/upstream stdout; all six
interpreter observations and all twelve port CLI observations exit 0 with empty
stderr. `eval` retains its Facet JSON `{entry, value}` envelope, with the same
term string as `run`. Two repetitions are byte-stable; no timeout occurred.

| Fixture suffix after `computed_operator_match_002` | Upstream stdout term | Port stdout term | Qualification |
| --- | --- | --- | --- |
| none | `x => sel(dbl(x))` | `x => sel(dbl(x))` | Exact bytes/status/streams. |
| `_dbl` | `\{Z: Z{}; S: (p => S{S{main(p)}})}` | `\{Z: Z{}; \{S: p => S{S{main(p)}}; \{}}}` | Same ordered match/core structure; nested fallback syntax differs. |
| `_sel` | `\{Z: R1{}; (y => RK{y})}` | `\{Z: R1{}; y => RK{y}}` | Same default lambda; parentheses differ. |

The bounded comparison helper parses actual captured oracle output, then compares
references, constructor names/arguments, applications, lambda binding distances
and ordered match arms/defaults. It performs no evaluation, eta conversion,
case reordering or text substitutions. Unsupported core forms fail closed.
Controls reject different binders, application order, constructor payloads and
defaults. Parsing the compact and nested spellings yields the same ordered core
tree; this explains the two differences without erasing their raw bytes.

These originals contain no Base import, so the pinned
[test gate](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L48)
selects checking and interpretation, not C/JS executable printers. Function
results also have no machine data printer under upstream
[show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1197).
Adjacent port C/JS regressions keep that refusal. This is a source-backed lane
decision, not an unavailable runtime or a missing dependency.

## Validation and reproduction

The standard gate passes **784 tests, zero failed, 49 ignored**. Strict all-target,
all-feature Clippy passes. Focused kernel, CLI, executable, generated-JS and serve
checks pass **67 tests**; the focused C function-printer control adds one pass.
The final normalization runner records 24 bounded commands: six upstream,
twelve port CLI and six structural probes. All exit 0, with empty stderr;
each child has a 60-second process-tree deadline. Longest observation: 107 ms.

The renewed positive CPU matrix has **308 exact matches in 320 executions**
(32 cases, five runtimes, twice). Twelve upstream-JS observations retain the
known `bun:ffi` host failure. All port lanes and available upstream lanes match;
no timeout or new discrepancy occurred. C uses `--threads 1 --gpu off`.
The runner exits 0 because it explicitly records these known host limits;
that exit does not count the twelve blocked observations as passes.

Use the existing activated tools with `CARGO_NET_OFFLINE=true`, `BEND_GPU=off`:

```powershell
cargo test --locked --no-fail-fast --test kernel_normalization --test kernel_core --test cli_proofs --test executable_check --test compiler_executable_javascript --test serve_protocol
cargo test --locked --test compiler_executable_c function_main_remains_unprintable_by_the_generated_c_data_printer
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo build --locked --features extended_observability --example compare_normalization --bin teamy-bend
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <reference> -OutputDirectory <fresh-normalization-directory> -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-directory> -Suite all -Repetitions 2
```

The [interpretation ledger](cpu-interpretation-evidence.csv) records source and
receipt hashes separately from the machine-runtime ledger. The final receipt
identifies a clean candidate and port binary SHA-256
`852cc10cf0b6e2cb0c64eb6de49e95b369afadabd36a6e03ae2ba02feff6a343`;
probe `89d13f3f93872f3410891d0a0a44dd136a94b0565f1ba9573af58ecadd01d270`.
The receipt SHA-256 is
`9a37f7e92c9de47d4eb3ed42b87c1e4e085cec3f08ab17cc32a969111e493b15`.
The renewed positive receipt SHA-256 is
`11b07a8cdd690629c09460f6e1ead79c1f230ef0fd7af2da2da62d1049e70ee0`.
Private `cpu-function-normalization` evidence preserves before/after binaries,
exact commands, raw bytes, timings, source identities, initial build/Clippy
corrections and completed gate logs. Original fixtures are copied unchanged with
Apache attribution; no raw diagnostics or machine paths are published.

## Remaining acceptance work

The interpretation ledger adds three distinct rows to the 51 runtime-linked
fixtures: **54 with some execution evidence, 1,248 without linked execution or
interpretation receipts**. The latter comprise 439 checker negatives, twelve
check-only modules, 161 interpretation-only fixtures and 636 compiled candidates.
These are different evidence requirements, not 1,248 runtime bugs. Two linked
interpretation rows still have raw rendering gaps; linked does not mean complete.
The [acceptance criterion](cpu-parity-acceptance.md) remains unchanged.

The next concrete independent family is
`flatten/computed_recursion_002.bend`, `_003.bend` and `_004.bend`: three pure
function-valued mains whose normalized match arms retain recursive `main(p)`
calls under helper applications. They are already strict-check accepted but have
no normalized-result receipt. Source review confirms no host effects, Base import,
pending demand decision or new dependency is needed. Qualify their actual normal
forms next, including recursion/reference preservation; do not infer a pass from
this operator family.

Earlier negative/comptime and multiworker timer receipts remain at their recorded
commits; this checkpoint does not relabel them as current. Their resource-policy,
native foreign, default upstream-JS stack and Bun-host gaps remain required work,
along with pending demand choices, the broader API inventory and platform lanes.
No GPU or fresh platform check was run. Dependencies, lockfile, toolchains and
reference/docs pins remain frozen. Local commits only; no push.
