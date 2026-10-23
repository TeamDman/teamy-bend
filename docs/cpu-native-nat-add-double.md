# Native Nat addition and doubling

Target: unchanged [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Engine: `7549f54a5fbb3a3631f4742eaaafd1a886237dda`.
Large, fully demanded additions and doublings now reach their result or exact
48-bit overflow diagnostic within the existing native budgets. This closes the
two work-limit cases from the [previous checkpoint](cpu-nat-failure-contract.md).
Arithmetic semantic parity and full CPU parity remain incomplete.

## Preserve demand while skipping successor runs

Pinned [Nat.add and Nat.double](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L526)
produce successor constructors recursively. In the native lazy evaluator, a
positive left operand can expose an addition's outer constructor without
demanding the right operand. Computing both complete operands eagerly would
change unused-error behavior and which error occurs first.

The sealed native optimization therefore stores a private successor count and
an unevaluated tail. Each count fits the existing 48-bit bound; doubling one
count needs at most two chunks. A constructor match reveals one successor.
Nat.show and U32.from_nat consume whole runs before demanding their tail, and
keep the existing overflow check. Array.new stops at its existing depth wall
without demanding a tail beyond that wall. Collection traces the tail, and
materialization, cancellation and work limits retain their existing controls.

Only exact, transitively checked Base bodies and layouts enable these paths.
Ordinary constructors still use the checked body; strict proof execution cannot
manufacture the new representation. No safety limit, generated C/JS engine,
public API, dependency, lockfile, toolchain or upstream source changed.

Six new tests plus strengthened existing cases cover zero, ordinary arithmetic,
large left operands, maximum/overflow boundaries, nested prefixes, partial
applications, lazy invalid tails, first-error ordering, collection at every safe
point, malformed/cyclic prefixes, cancellation and exhausted work. Parked-channel
cleanup and diagnostic-writer failures now exercise add/double as well as multiply.

## Measured results

| Check | Result |
| --- | --- |
| Full Linux standard gate | **767 passed, 0 failed, 49 ignored**; format, strict Clippy, all-feature build and Node-provider checks passed. |
| Strict all-target/all-feature Clippy | Passed without suppression. |
| Focused native checks | 243 passed, zero failed, one optional profiler ignored. |
| Existing seven-case authored Nat matrix | **70/70 exact**, up from 66/70. Large-left addition and doubling now emit the required overflow line/status, preserving preceding stdout. |
| Expanded thirteen-case matrix | 130 executions, repeated twice across five runtimes. Eight cases match all five runtimes; five partial-demand cases remain unresolved, detailed below. |
| Six unchanged upstream runtime-negative fixtures | 38 exact / 56 executions; six diagnostic, eight safety-policy, two native foreign and two Bun-host gaps remain. Four C observations are inapplicable to the JS-only companion. Runner exits 1. |
| Nineteen positive/control cases | 184 exact / 190 executions; six established upstream-JS Bun failures retained. |
| Sixteen positive comptime fixtures | 160 exact / 164 executions; four default upstream-JS stack failures retained, supplemental stack-4096 runs pass. Every port execution passes, including original hand/tmpl. |
| Nat and request worker controls | 18/18 each, including upstream/port C at 1, 2 and 4 workers. |
| Merged stream order / generated-runtime cleanup | 10/10 ordered executions and 11 cleanup probes pass. |

All repeated outputs/statuses are deterministic. No timeout occurred; negative
execution directories retained no files. Generation/build/execution deadlines
remain 60 seconds. All 39 upstream ledger fixtures were renewed against the same
binary SHA256, `d92a69f775828b0e9ca3a019870cc71d05fb106caeedeb3ae37ee9203cb49b76`.
Authored cases do not increase the upstream inventory coverage count.

The expanded matrix's eight fully matching cases cover large addition, addition
at the maximum, large doubling, doubling just below overflow, nested operations,
partial application, ordinary arithmetic and an unselected invalid branch. Its
before receipt uses the retained `b26ca5e` binary; twelve native work refusals
become exact matches. The seven-case overflow matrix closes four more repeated
work refusals. None of the partial-demand outcomes changed.

## Arithmetic semantics are not fully qualified

In this table `m = 2^48 - 1`, `h = 2^47`, `head` observes only zero versus
successor, and `discard` returns `"kept"` without using its Nat argument. The
saved Bend programs construct the large values from U32-to-Nat multiplication.

| Observation | Native before and after | Upstream C/JS and port C/JS |
| --- | --- | --- |
| `discard(Nat.add(m, 1n))` | Status 0, stdout `kept\n`, empty stderr | Status 1, empty stdout, exact Nat-overflow line |
| `head(Nat.add(m, 1n))` | Status 0, stdout `positive\n`, empty stderr | Status 1, empty stdout, exact Nat-overflow line |
| `head(Nat.double(h))` | Status 0, stdout `positive\n`, empty stderr | Status 1, empty stdout, exact Nat-overflow line |

These are **six repeated native semantic differences**, not resource refusals or
diagnostic-only mismatches. Preserving native demand was an explicit constraint
of this correction. Selecting compiled eagerness as the native contract would
require a separate decision; the differences are not waived by this milestone.

Two more authored cases use this checked syntax:

```bend
def reject(op: IO.OP<Unit>) -> Nat:
  match op:
    case Emit{x}: 0n
    case _: 0n
def invalid() -> Nat: reject(IO.print("SECRET")(Unit, x => Emit{x}))
```

Both `head(Nat.add(U32.to_nat(4294967295), invalid()))` and
`head(Nat.add(Succ{invalid()}, invalid()))` produce **different upstream oracles**:

- Upstream C returns status 0, `positive\n`, empty stderr.
- Upstream JS returns status 1, empty stdout, `bend: runtime fail-stop\n`.
- Port C/JS follow the JS refusal; native returns the lazy positive result.
- No lane prints the request's `SECRET` text.

The generated C does call `invalid`: its explicit wildcard arm sinks the request
and returns zero. Its success is not evidence of lazy evaluation. The generated
JS includes an unconditional private-request guard before the match, as specified
by [JS IO.OP lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L3034).
[C match lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2657)
keeps the default arm. This is a distinct wildcard/private-request contract
disagreement. The port's safe refusal was not weakened to imitate that C result.

The 130-observation receipt records 108 matches to the selected same-runtime
oracle, ten differences (six native overflow demand, four port C wildcard
requests), eight upstream observations in C/JS-disagreement cases, and four
unqualified native observations in those cases. **108/130 is not a whole-family
pass rate**: native agreeing with one conflicting oracle does not qualify it.

## Remaining requirements and API caveat

The [acceptance criterion](cpu-parity-acceptance.md) remains unchanged: complete
lane review and exact required behavior are still needed. Only 39 of 1,302
inventory rows have linked current runtime receipts; 1,263 unlinked rows are
neither a defect count nor a completion percentage.

The next independent bounded CPU correction is native/JS array-depth and JS
scalar-marshalling diagnostics through the structured failure boundary. Keep C's
lower allocation ceiling, bounded-stack refusals and missing native foreign
support explicit. The newly recorded demand and wildcard-request contracts need
source/oracle review before deciding which runtime behavior to change. No GPU
hardware is needed for either investigation. Bun remains unavailable for its
oracle lanes; no installation was attempted.

The earlier API caveat still applies: the private field added to KernelError in
`b640bf5` breaks unknown external struct literals/exhaustive patterns, and its
derived Debug/Eq now includes the failure kind. Public message/Display remain.
No tracked repository consumer uses such literals; external compatibility is
unverified. `run_main` callers must inspect `Ok(status)`, including nonzero
statuses; the repository CLI already forwards it correctly. This family adds
no further public API change.

## Reproduction and evidence

With the existing activated tools, `BEND_GPU=off` and `CARGO_NET_OFFLINE=true`:

```powershell
cargo test --locked --no-fail-fast --lib --test runtime_numeric --test runtime_packed_words --test runtime_reclamation --test runtime_channels --test runtime_request_failure
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
```

Private `cpu-native-nat-add-double` evidence retains exact commands, source and
binary hashes, before/after arithmetic probes, original negative/positive/comptime
receipts, worker/order/cleanup probes and gate logs. The initial strict-Clippy
attempt reported three style issues, corrected without exemptions; the final
run passes. The first expanded-before probe stopped on the unexpected upstream
disagreement, then resumed with explicit per-oracle classification and retained
its original log. Earlier evidence is intact. Fresh Windows/macOS, GPU/hardware
profilers, Bun and broader unreviewed corpus lanes remain unrun. No push occurred.
