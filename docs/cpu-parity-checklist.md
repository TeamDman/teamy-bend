# CPU parity against the pinned upstream

Reviewed on 2026-10-23 against upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, starting from port
`0ffb1415342df9573b3f0ff8d1c5e52285457ddc`. This is a coverage checklist,
not a completion percentage. CPU parity remains incomplete, and CPU work can
continue in the cloud without GPU hardware. Dependencies, lockfile and toolchains
were unchanged. The earlier Linux portability and task/file milestones retain
their own [portability](linux-cpu-validation.md) and
[effects](cpu-effects-validation.md) receipts.

## Passed in this review

| Evidence | Result | Boundary |
| --- | --- | --- |
| Standard Linux gate | 728 passed, 0 failed, 49 ignored; formatting, strict Clippy, all-feature build and real Node-provider contract passed | Two new regressions account for the increase from 726. This is not the earlier Windows 728-test receipt. |
| Focused native/JS/CLI/channel checks | 202 passed, 0 failed, 1 ignored | Includes forced-collection cleanup, cancellation, blocked send/receive and diagnostic write/flush failures. |
| Five new control fixtures, five runtimes, twice each | 50 exact stdout/stderr/status matches; zero timeouts; deterministic across repetitions | Channel close/send, second join, UTF-8 Halt, mixed UTF-8 streams and deadlock. All sources are unchanged upstream fixtures. |
| Existing five-case baseline | 25/25 matching executions | Rechecked on the changed runtime; no GPU execution. |
| Existing six task/file fixtures, twice each | 54 matching executions and six classified upstream-JS host failures; zero timeouts | All three port runtimes match upstream C. The three Bun-FFI-dependent upstream JS cases remain unqualified under Node. |
| Negative controls for the comparison runner | Wrong status and wrong stderr both rejected | A nonzero expected status does not allow arbitrary failures or diagnostic changes. |
| Complete strict acceptance audit | 1,302 fixtures; 364 expected positives accepted, 489 rejected, 449 nonzero-exit expectation fixtures rejected; zero abnormal exits | Exactly reproduces the previous acceptance counts. It does not compare runtime behavior or exact rejection diagnostics. |

The gate ignores 47 CUDA hardware tests and two optional profilers. Those remain
unrun; no ignore was removed and no assertion was weakened.

## Concrete correction: channel deadlock

The unchanged upstream `tests/io/deadlock.bend` prints `waiting\n`, then writes
`bend: deadlock: every computation waits on a channel\n` to stderr and exits 1.
Upstream C, upstream JS and our generated C already agreed. Before this fix,
our generated JS instead emitted:

```text
teamy-bend generated program: IO deadlock: no runnable task, timer or descriptor
```

The native runtime returned a host error, causing the CLI to add error-report
formatting, ANSI color, a source location and a backtrace hint. It now treats
channel deadlock as a language exit: flush pending stdout, write the upstream
diagnostic, return status 1, and release pending tasks and owned resources.
Generated JS makes the same terminal transition through its existing cleanup
path. Cancellation, resource exhaustion and failed diagnostic writes still return
host errors. For Rust callers, a deadlock is now `Ok(1)` rather than `Err`.

Existing weak substring checks were strengthened to exact diagnostics. New CLI
coverage checks both blocked send and blocked receive after unflushed output;
new native checks preserve write/flush error propagation. Forced-collection tests
still prove pending payloads and continuations are released.

| Unchanged upstream fixture | Stdout | Stderr | Exit |
| --- | --- | --- | --- |
| `io/chan_close_send.bend` | `TrueFalse\n` | empty | 0 |
| `io/fiber_single_join.bend` | empty | `IO.join: the channel was closed\n` | 1 |
| `io/halt_utf8_law.bend` | empty | `è❁\n` | 7 |
| `io/text_utf8.bend` | `eéé\n` | `é\né\n` | 7 |
| `io/deadlock.bend` | `waiting\n` | `bend: deadlock: every computation waits on a channel\n` | 1 |

The control suite compares bytes in each stream separately. A matching combined
terminal transcript would not establish the same contract.

## Executable coverage versus strict checking

Strict proof checking deliberately excludes execution assumptions such as IO
and opaque numeric primitives. A diagnostic-based triage of the 489 rejected
positives finds 430 naming unavailable effect/numeric surfaces, 29 naming a
resource bound, and 30 needing other triage. These are first-diagnostic groups,
not 489 independent semantic bugs or a claim that fixing the first diagnostic
will make a program pass. Some remaining cases concern unsafe execution,
foreign imports or interactive effects that do not belong in strict proof mode.

To test that distinction, this review additionally generated and executed all
16 positive fixtures in upstream `tests/comptime`. Each source was inspected:
the fixtures compute values and print them; they do not perform file/network
effects. The two marked offload cases ran with GPU disabled. Compiled C runs used
`--threads 1 --gpu off`; port C retained strict warning flags. Generation,
compilation and execution each had a 60-second bound. These were single-pass
coverage probes, not performance measurements or determinism certification.

| Runtime | Exact expected output, empty stderr, exit 0 | Remaining observations |
| --- | --- | --- |
| Upstream C | 16/16 | CPU oracle, including `hand` and `tmpl`, each printing `50005000\n`. |
| Port C | 16/16 | All strict C compilations and executions passed. |
| Upstream JS, default Node stack | 14/16 | `hand` and `tmpl` exit 1 with `bend: memory fault (machine stack overflow?)`. |
| Upstream JS, two supplemental runs with `--stack-size=4096` | Both remaining cases pass | Existing Node, per-process flag only; source and global environment unchanged. This qualifies the oracle under that stack setting, not default Node behavior. |
| Port JS | 14/16 | `hand` and `tmpl` exit 1 with `executable JavaScript call depth budget exhausted`. |
| Native Rust | 14/16 | `hand` and `tmpl` exit 1 with `data runtime continuation depth exhausted`. |

The default matrix contains 74 matching executions and six nonmatching ones,
with no timeouts. Two additional upstream-JS executions resolve its stack
limitation; the four port resource-limit failures remain. This is direct evidence
of unfinished CPU work even though generated C already handles these programs.
No port limit was raised and no fixture was shortened to manufacture parity.
The old statement that all comptime fixtures reject in the strict audit therefore
does not mean executable template support is absent.

## Continuation follow-up

The next bounded change starts from local commit
`eb6c5902a1bb9d070907dcdb80455584aa1a735d`; the earlier receipts above remain
historical evidence. That checkpoint passed **733 tests, zero failures and
49 existing ignores** in the standard Linux gate. Strict Clippy over all test targets also
passes. Its extra check found one trait-parameter naming lint in the previous
deadlock regression; renaming that parameter changes no assertion or behavior.

Generated JavaScript now uses the existing trampoline for the last live field
of an ordinary tagged constructor. Earlier fields evaluate in source order;
erased fields stay omitted. A private construction marker carries the unfinished
record, and only its completed value can return to Bend or a foreign callback.
Native scalar/string/array constructors keep their existing representation and
non-tail call guard. Constructor chains have a 131,072-node bound per trampoline
invocation, in addition to the unchanged 2,000,000-transition budget and
512-call guard. An infinite constructor producer fails explicitly rather than
exhausting the JavaScript stack. Tests cover both unchanged upstream workloads,
erased fields, closure fields, signed-zero bits, prefix-error order, and the
existing genuine non-tail, string and transition bounds.

The native interpreter already uses explicit continuations. Its initial failure
had 4,090 memoization updates among 4,096 frames. Adjacent updates now share a
memoized target through a collector-traced alias. A checked chain of 2,200 tail
calls and a 10,000-step list builder now complete; the retained pre-change binary
fails the latter at the continuation bound. Forced collection, sharing, cyclic
evaluation, cancellation and
nested numeric depth regressions pass. No step, frame, arena or output limit
changed. This does **not** finish the two native workloads: deferred arithmetic
and pattern matching still retain real pending work. The whole programs now
reach the work budget; scaled 500/1,000-element diagnostic versions still reach
the continuation bound. The 100-element version returns `5050` before and after.
The native evaluator's documented lazy behavior must not be replaced with eager
argument forcing merely to make this benchmark pass.

Reduced native cases isolate the remaining work: at 10,000 steps, a constant-head
builder followed by `first` returns `1`, but demanding the incremented range
head or summing even constant heads reaches the continuation bound. Each reduced
500-element case succeeds. Combining mapping and summing still fails at that
smaller size. These are diagnostic variants, not replacements for the unchanged
upstream workloads. They identify deferred numeric chains as the next target;
list-tail construction alone no longer requires an update frame per call.

All 16 positive comptime fixtures were regenerated and executed twice:

| Runtime | Exact output/status matches | Remaining observations |
| --- | --- | --- |
| Upstream C | 32/32 | CPU, one worker, GPU disabled. |
| Port C | 32/32 | Strict C11 warnings remain errors. |
| Upstream JS, default Node stack | 28/32 | Both repetitions of `hand` and `tmpl` hit the upstream stack failure. |
| Upstream JS, supplemental 4 MiB stack | 4/4 | Both whole programs print `50005000\n`, exit 0, empty stderr. |
| Port JS, default Node stack | 32/32 | Includes both whole programs; no Node stack flag. |
| Native Rust | 28/32 | Both repetitions of the whole programs hit the unchanged work budget. |

That is 156 matching and eight classified nonmatching executions, including the
four supplemental oracle runs, with zero timeouts. Repetitions have identical
stdout, stderr and status. The existing baseline/effects/control suites also
ran twice: 154 exact matches and six previously classified upstream-JS
`bun:ffi` failures, zero timeouts. All three port runtimes match their C oracle.

One further audit candidate, unchanged `base/list_fold.bend`, passes all five
executable runtimes, including filtering, both folds, predicates, search, stable
sorting and IO iteration. Its strict-audit refusal is not evidence of an absent
executable List template implementation. This was a single bounded coverage
probe, not a full Base-library qualification.

A supplemental 100-record float/closure probe exposes a pre-existing reference
backend difference: both C emitters print `100`, while both JS emitters under
this Node version print `99` for the checksum of quiet-NaN payload bits. The retained
pre-change port JS binary reproduces `99` three times; native execution reaches
its work budget. This is not a new constructor-tail regression or a resolved
float-parity claim. The new constructor-tail regression uses signed zero for
exact field-bit preservation; existing NaN tests remain unchanged. The broader
NaN/host representation question needs separate oracle qualification.

## Native numeric-chain follow-up

Starting from `f085f0b85057decf4c360ff586e34445c1ba1dee`, both original
10,000-element programs now also pass native execution. The correction propagates
already available U32 values: closed literal words and packed values can satisfy
sealed `U32.inc`, `U32.add`, `U32.mul` and `U32.shl` operations without building
another deferred arithmetic chain. Increment joins the existing checked-body
optimizations and uses the ordinary body whenever its argument is not packed.
No general argument evaluation or ordinary Word-field decoding is performed by
this inspection. Unknown, partial, cyclic or non-value arguments retain the
existing lazy path; strict proof programs never enable this optimization.

Bound variables reuse their existing thunk identities. Cached global references
do likewise, while an uncached global reference stays suspended without retaining
the caller's local environment. This avoids unnecessary retention and collection
work in the template version. It does not unfold an unresolved reference or
execute a request early. Inspection walks have a 32-transition cutoff and charge
the same work budget; reaching the cutoff falls back to normal evaluation.
The 2,000,000-step, 4,096-frame and 131,072-slot arena bounds are unchanged.
Diagnostic evaluator counts were 1,463,168 work units for `hand` and 1,858,579 for
`tmpl`; these are implementation work counts, not elapsed-time benchmarks.

New tests preserve an unused invalid Word tail, then demand it after printing
`before`: the original request-matching error occurs, without executing the
request or printing `after`. Other controls cover increment wrapping and sealed
origins, suspended errors/requests, cyclic inspection, deep environment fallback,
a million-iteration ready accumulator, cancellation, and both unchanged whole
fixtures. Existing non-tail numeric depth, lazy arithmetic, forced collection,
proof refusal and output-bound checks remain intact.

| Check | Result |
| --- | --- |
| Standard Linux gate | 738 passed, zero failed, 49 existing ignores |
| Strict Clippy across all test targets | Passed without suppression |
| Focused native checks | 237 passed, zero failed, one existing profiler ignore |
| All 16 positive comptime fixtures, two repetitions | Native, port C, port JS and upstream C each match 32/32 executions |
| Upstream JS | 28/32 with default stack; four supplemental 4 MiB-stack executions match |
| Whole `hand` and `tmpl`, all qualified runtimes | Exactly `50005000\n`, empty stderr, exit 0 in both repetitions |
| Existing baseline/effects/control suites, twice each | 154 exact matches; six known upstream Node `bun:ffi` failures; zero timeouts |

The matrix has 160 exact matches among 164 executions, with four classified
upstream default-stack failures and zero timeouts. All repeated outputs and
statuses are deterministic. This closes the identified native numeric-chain gap
for these fixtures; it does not establish full CPU parity. Unrelated corpus,
foreign-effect, host and hardware qualifications below remain open.

## Remaining coverage checklist

`Missing` below means incomplete implementation or qualification. `Blocked`
identifies a specific unavailable host capability; it does not block unrelated
CPU work.

| Status | Area | Evidence and next action |
| --- | --- | --- |
| Passed, bounded | Linux build/provider/strict C portability | Standard gate plus original five-case and task/file comparisons. Keep these as regressions. |
| Passed, bounded | Channel terminal outcomes | New exact five-runtime control suite; cleanup and cancellation regressions retained. Broader close-wakeup scheduling has local tests but still needs selected upstream differential cases. |
| Passed, bounded | JS ordinary constructor tails | All 16 positive comptime programs now match under default Node, twice each; adversarial chain and non-tail bounds remain enforced. |
| Passed, bounded | Native numeric-chain workloads | Both original 10,000-element programs now match under unchanged limits, twice each. Available-value propagation and reference sharing preserve lazy/error controls; arbitrary non-tail/deeper workloads remain bounded. |
| Missing | Parser/checker/base breadth | Full strict audit remains 364 accepted positives. Separate strict exclusions from real parser/template/descent gaps. `base/list_fold.bend` now has five-runtime executable evidence despite its strict refusal; validate other candidates in their intended mode. |
| Missing | Exact negative diagnostics and execution of accepted proofs/data | Acceptance audit alone covers neither. Audit refusals can happen for the wrong reason; runtime-error fixtures may legitimately type-check. |
| Missing | Wider executable corpus coverage | Only selected families have actual five-runtime output comparisons. The pinned corpus contains 1,302 fixtures, including 100 IO, 101 compile and 240 flatten fixtures; these are source counts, not a common runnable denominator. |
| Missing | Arbitrary foreign effects in native Rust | Generated C/JS support companion imports; the native interpreter explicitly refuses arbitrary foreign implementations. This is an implementation gap, not a GPU requirement. |
| Missing | Broader network/file/error comparisons | The gate covers loopback and host adapters. Independently qualify selected upstream socket readiness, partial transfer and error fixtures without blindly executing the whole effect corpus. |
| Missing | Large pure-library programs and optimization | Existing whole-program string/number kits hit bounds despite passing shorter chunks. Native/JS storage/traversal and compiler specialization remain open; chunk comparisons are not whole-program parity. |
| Missing | Upstream loader/CLI/module parity | JS module marshalling, imports, browser loader and command-surface behavior need their own inventory. The Rust CLI's typed wrappers are not upstream language equivalence. |
| Missing / host-dependent | Window/audio/interactive effects | The rewrite's documented unfinished effects are separate from headless CPU checking and computation. Device integration needs an appropriate host and any dependency additions require approval. |
| Blocked on current JS host | Three existing upstream task/file fixtures | `bun:ffi` cannot load in Node; Bun is not installed. Exact failed outputs are retained. No dependency installation was requested or performed. Upstream C remains available for their CPU oracle. |
| Unrun, hardware-dependent | CUDA/Metal/sanitizers | Desktop or suitable GPU hardware is needed for these checks. They are not prerequisites for the CPU tasks above. |
| Unrun here | New Windows/macOS qualification of this change | Historical Windows results qualify older commits, not this candidate. |

The isolated native numeric-chain milestone is complete. Before claiming CPU
parity, continue the missing coverage above with a separately bounded set of
upstream executable fixtures and exact negative outcomes. Larger lazy graphs,
arbitrary foreign calls and host-specific effects remain distinct requirements.
This checkpoint adopts no newer upstream code or dependency and needs no GPU
hardware to continue CPU work.

## Reproduction and retained evidence

Use the existing activated tools with `BEND_GPU=off` and
`CARGO_NET_OFFLINE=true`. No fetch/install is needed. From the port checkout:

```powershell
cargo build --locked --features extended_observability --example audit_upstream --bin teamy-bend
cargo run --locked --features extended_observability --example audit_upstream -- <reference> <snapshot-binary>
cargo test --locked --features extended_observability --lib --test runtime_channels --test compiler_executable_channels --test cli_effects
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-control-dir> -Suite control -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-baseline-dir> -Suite baseline
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-effects-dir> -Suite effects -Repetitions 2
```

The audit used a frozen baseline binary and a 900-second outer deadline; it
completed normally. The runtime correction does not change parser/checker code.
Raw audit JSON, category inventory, triage, command arrays, source and binary
hashes, stdout/stderr bytes, exits, timings and timeout flags are retained in the
private `cpu-parity-review` evidence directory, along with the bounded comptime
probe and negative harness controls. Every original evidence directory remains
intact. Do not publish raw diagnostics containing machine-specific paths.

The follow-up adds these checks with the same activated, offline tools:

```powershell
cargo test --locked --features extended_observability --no-fail-fast --lib --test runtime_review --test runtime_numeric --test runtime_reclamation --test compiler_executable_javascript --test compiler_executable_app
cargo clippy --locked --all-targets --features extended_observability -- -D warnings
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-all-dir> -Suite all -Repetitions 2
```

The unchanged `hand.bend` and `tmpl.bend` copies in
`tests/fixtures/cpu-comptime` retain upstream attribution. Their JS regression
runs in the standard gate with a 30-second child deadline. The private
`cpu-continuation-depth` evidence directory retains exact commands, source and
binary hashes, output bytes, statuses, timeouts, the twice-run 16-case probe,
scaled and isolated native diagnostics, List-fold and NaN probes, and initial failed focused
checks as well as final passing logs. Diagnostic instrumentation is absent from
the committed runtime. Original evidence directories are intact.

The numeric follow-up evidence is retained separately in the private
`cpu-ready-numeric` directory: full/focused/Clippy logs, exact commands and
hashes, repeated comptime and existing-suite receipts, and diagnostic work
counts. Temporary diagnostic instrumentation was removed before qualification.
To reproduce the now-passing native cases, run either whole fixture under
`teamy-bend run`; compare with `compile --executable --target javascript|c`.
Only the upstream JS oracle needs `node --stack-size=4096`; C uses
`--threads 1 --gpu off`. Keep port resource budgets unchanged.

The frozen `Cargo.lock` SHA256 is
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
No source in upstream or teamy-docs was modified. This review does not establish
full CPU parity, new GPU evidence, or permission to adopt newer dependencies.
