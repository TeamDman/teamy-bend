# CPU parity against the pinned upstream

Reviewed on 2026-10-23 against upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, starting from port
`0ffb1415342df9573b3f0ff8d1c5e52285457ddc`. This is a coverage checklist,
not a completion percentage. CPU parity remains incomplete, and CPU work can
continue in the cloud without GPU hardware. Dependencies, lockfile and toolchains
were unchanged. The earlier Linux portability and task/file milestones retain
their own [portability](linux-cpu-validation.md) and
[effects](cpu-effects-validation.md) receipts.

The latest [rewrite checking checkpoint](cpu-rewrite-checking.md) records source
`4c90907a78db96cc9e46da6d25659f10b4d978cc`, full gate 901/0/49, and 213 detailed originals,
including historical/unresolved cells. There are 1,089 without detailed receipts:
429 checker negatives, one checking-only module, 106 interpreter cases and 553
compiled candidates. These are distinct applicable contracts. Twenty checking,
99 interpretation and seven runtime originals are renewed; earlier receipts
retain their historical identities. Full CPU parity remains incomplete.

The interrupted 688-vector exploration stays unqualified. No input, guard,
cancellation, resource or demand policy changed. See the
[acceptance criterion](cpu-parity-acceptance.md) and latest checkpoint below.

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

## Persistent-collections coverage follow-up

The next bounded family exercises map replacement/default reads/deletion/size,
prefix-key ordering, right-biased union in both operand orders, set deduplication
and written-array mapping. All sources below are unchanged pinned fixtures.

| Fixture | Exact stdout (empty stderr, exit 0) |
| --- | --- |
| `base/map_ops.bend` | `7\n` |
| `base/map_prefix_keys.bend` | `a=2,ab=1,b=3,.\na=20,ab=1,b=3,c=30,.\na=2,ab=1,b=3,c=30,.\nLTGTLT\n` |
| `base/map_set_array_kit.bend` | `3\n[1, 2, 3]\n2\n[x, y]\n[0, 5, 0, 7]\n[1, 6, 1, 8]\n` |

Upstream C/JS and all three port runtimes match twice each: 30/30 exact executions,
zero timeouts, deterministic statuses and both streams. No runtime correction
was warranted. `compare-cpu.ps1 -Suite collections` preserves this comparison;
`-Suite all` now contains 19 fixtures and yields 184 exact matches plus six known
upstream-JS `bun:ffi` failures across 190 executions. C now receives explicit
`--threads 1 --gpu off`; the runner snapshots and hashes its input binary.
This qualifies a serial C profile, not the remaining multiworker family.
The unchanged engine still passes the standard gate: 738/0/49.

The negative control `base/heap_queue_deque.bend` is deliberately rejected by
upstream: Deque is absent from this pin's Base. Upstream direct checking, port
execution checking and port C generation all reject `Deque.push_front` with
status 1, twice each, and no C artifact is produced. The raw diagnostics differ;
only rejection phase/name agreement is established, not exact diagnostic parity.
Implementing Deque to make this fixture succeed would contradict its expectation.

Direct upstream IO execution of the three positive programs is separately
blocked on this Node host by `import.meta.require`, while their emitted CommonJS
programs pass. The acceptance document records this direct-entry lane separately;
no reference source or host dependency was changed to conceal the distinction.
Inventory generation was repeated byte-for-byte and its wrong-pin refusal was
checked. All 1,302 rows also agree with the pinned gate's actual selection helper;
a deliberately wrong expected output makes the comparison runner fail.
Private `cpu-collections` evidence retains all receipts and exact commands.

## Runtime-negative follow-up

The [six-fixture failure comparison](cpu-runtime-negatives.md) exposed and fixed
native acceptance of the depth-32 array program. The sealed executable Array.new
path now rejects before printing its element, preserves lazy fields/erased types,
and leaves proof mode and safety limits intact. Six new regressions include
boundaries, demand, origins, work/cancellation and parked-channel cleanup.

The repeated matrix has 56 applicable executions: 26 exact matches, 16 diagnostic
mismatches, ten safety refusals, two unsupported native foreign calls and two
upstream-JS bun:ffi failures. Four C observations are inapplicable because the
scalar fixture has only a JS companion. No timeout or residual execution file
occurred; all observations were deterministic. Phase-only agreement remains open.
Eleven supplemental port C/JS ownership probes pass; native cleanup is directly
regression-tested. Full gate: 744 passed, zero failed, 49 ignored; strict all-target
Clippy passes. All prior 19-case and 16-comptime comparisons were refreshed on
engine d20c904 with their known upstream host/stack limits retained.

The ledger now links 39 fixtures, including unresolved negative cells. The next
small CPU correction is the native Nat.mul work-budget gap. Lower C allocation
bounds, exact runtime diagnostics, foreign support and broader coverage remain
open, not certified by this correction.

## Native Nat multiplication follow-up

The [bounded correction](cpu-native-nat-mul.md) replaces needless Peano expansion
for demanded packed operands with checked 48-bit multiplication. Zero-left demand,
ordinary constructor fallback, partial calls, origin guards, proof separation and
all safety limits remain intact. The original overflow now reaches its Nat law;
its Eyre diagnostic still differs, so the native cell remains unqualified.

Six new regressions pass. Full gate: 750 passed, zero failed, 49 ignored; strict
all-target Clippy passes. The repeated negative family now has 26 exact matches,
18 diagnostic mismatches, eight policy/budget refusals, two unsupported native
foreign calls and two upstream-JS host failures over 56 executions. All 39 ledger
fixtures were renewed; prior positive/comptime results are retained. An authored
product/boundary control adds 10/10 exact five-runtime executions without changing
the upstream-fixture count. No timeout; old evidence is retained.

Remaining differences are not all correctness bugs or all policy exceptions.
Diagnostic byte differences are contract gaps; C class/deep-stack limits are
explicit safety policies, native foreign support is missing implementation, and
Bun/default-Node-stack limitations belong to the oracle host/profile. None silently
waives an acceptance cell. Next: the existing request-out-of-band failure contract.

## Request fail-stop follow-up

The [bounded failure-contract correction](cpu-request-fail-stop.md) gives genuine
out-of-band requests a structured failure category in native/C execution and
uses JS's authenticated request identity. Raw stdout/stderr and status now match
upstream; unrelated host errors retain their diagnostics. Output order, collection,
channel cleanup, lazy demand and C worker error forwarding remain tested.

Full gate: **755 passed, zero failed, 49 ignored**; strict all-target Clippy passes.
The original request fixture matches all five runtimes twice. An authored fork
control matches all 18 observations, including both C implementations at 1, 2 and
4 workers. Eleven supplemental cleanup probes pass. These are bounded profiles,
not general scheduler parity. All 39 ledger fixtures were renewed on `b640bf5`;
positive/comptime outcomes are unchanged, with zero timeouts.

The negative matrix now has 32 exact matches, 12 diagnostic differences, eight
safe resource-policy refusals, two unsupported native foreign calls and two
upstream Bun failures over 56 executions. Its exit remains 1. Next: exact Nat
and array diagnostic contracts, treating C's lower array ceiling separately.
Dependencies, toolchains, reference pins and all earlier evidence are preserved.

## Nat diagnostic follow-up

The [Nat failure-contract correction](cpu-nat-failure-contract.md) extends the
structured boundary to actual Nat overflow. Generated arithmetic shares its
existing guard; native multiplication and reconstructed-prefix bounds now own
exact language diagnostics. Host errors with identical text stay internal.
Cleanup, first-error ordering, lazy unused operands and safety limits are retained.

Full gate: **761 passed, zero failed, 49 ignored**; strict all-target Clippy passes.
The six-fixture matrix now has 38 exact matches, six diagnostic differences, eight
safe resource-policy refusals, two native foreign failures and two upstream Bun
failures over 56 executions. Its exit remains 1. All 39 ledger fixtures were
renewed on `b26ca5e`; positive/comptime outcomes are unchanged. Worker/order/cleanup
probes pass, with zero timeout. The API review identifies the CLI as the sole
production run_main caller and documents the prior KernelError source break.

An authored Nat family improves from 34 to 66 exact observations out of 70.
Four repeated native refusals remain: large-left Nat.add and large Nat.double
consume work through recursive checked bodies while compiled runtimes reach the
overflow law. These are newly explicit algorithmic compatibility gaps, not fixes
provided by the diagnostic boundary. Keep them unqualified pending bounded,
demand-preserving arithmetic work. Exact array/scalar diagnostics also remain.

## Native Nat add/double follow-up

The [demand-preserving arithmetic correction](cpu-native-nat-add-double.md) stores
bounded successor runs with lazy tails for the exact sealed Base bodies. Ordinary
constructor fallback, first-error order, collection and safety limits remain.
The existing authored Nat matrix now matches 70/70, closing four repeated native
work refusals. The expanded matrix closes twelve additional work refusals across
six large/boundary/composition cases; eight of its thirteen cases match all five
runtimes twice.

Three partial-demand overflow cases retain native lazy success where compiled
upstream rejects: six repeated semantic differences, not diagnostic differences.
Two wildcard/private-request cases expose different upstream C and JS outcomes;
port C/JS refuse safely, while native preserves lazy success. Neither agreeing
with one oracle nor preserving a local regression establishes parity there.
The checkpoint records every status/stream and the lowering distinction. These
pre-existing contracts remain open; no eager forcing or weakened refusal was
introduced to disguise them.

Full gate: **767 passed, zero failed, 49 ignored**; strict all-target Clippy passes.
All 39 upstream ledger fixtures were renewed on `7549f54`, with previous
negative/positive/comptime outcomes intact. Worker/order/cleanup probes pass;
no timeout. Authored controls add no upstream inventory rows. The earlier
KernelError source-compatibility break and run_main nonzero Ok caveat remain
explicit; this change adds no public API. Next independent bounded CPU work:
native/JS array-depth and JS scalar-marshalling diagnostics. Demand/oracle
contracts need review before choosing a behavior change.

## Array-depth and scalar diagnostic follow-up

The [diagnostic correction](cpu-array-scalar-diagnostics.md) classifies native
array-depth and authentic JS array/scalar failures through the existing boundary.
Guards, budgets, evaluation order, arithmetic demand and ordinary host diagnostics
are unchanged. Five new tests plus stronger originals cover boundaries, prior
output, first-error order, lazy branches, lookalikes, cleanup and writer failures.

Full gate: **772 passed, zero failed, 49 ignored**; strict all-target Clippy passes.
The six-fixture matrix improves to 44 exact out of 56: all six diagnostic-only
differences closed, with eight resource-policy, two native foreign and two Bun
observations still open. Runner exit 1 remains intentional. Additional guards
match 66/74 with eight explicit resource refusals. Nat controls remain 70/70;
worker/order/cleanup and previous positive/comptime outcomes are retained.
All 39 upstream ledger fixtures were renewed on `76038aa`, without adding an
inventory row; no timeout occurred.

The [demand-contract proposal](cpu-demand-contract.md) adds five independent
reproducers and 50 repeated observations. Three native demand differences and
two upstream C/JS wildcard-request disagreements remain unchanged. Pinned paper,
guide, compiler and regression sources support the proposed strict live-value
and private-request contracts. This is awaiting review: no semantics change or
conflicting-oracle exception was adopted. The downstream API caveat remains;
array language failures now use the existing nonzero Ok status convention.

## Timer-order qualification follow-up

The [scheduler checkpoint](cpu-timer-order.md) adds unchanged sleep_order and
sleep_timer fixtures plus an authored zero-timer control to the differential
runner. It changes no production runtime. Required registration/ready order
matches in native, port C/JS and upstream C; the unmodified upstream-JS poller
needs unavailable Bun, even for zero timers. Three repetitions at serial C plus
additional C profiles 2/4 yield 72 exact of 81 observations, nine host-blocked.
No elapsed-time assertion was added. Controlled-clock tests retain distinct
deadline-order and overdue-registration-order results; mixed-duration spawn_sleep
is not silently qualified by its usual wall-clock golden.

Full gate remains **772 passed, zero failed, 49 ignored**; strict all-target
Clippy passes. Focused scheduler checks pass 217 tests with one profiler ignored.
The expanded positive suite records 208 exact of 220, twelve Bun failures;
negative/comptime outcomes are unchanged. No timeout or timer defect was observed.
The ledger now links 41 upstream fixtures at `6bfc10a`, leaving 1,261 unlinked
inventory rows; both new rows retain blocked JS cells. The authored control adds
no upstream inventory row. The acceptance criterion is unchanged.

The [decision brief](cpu-demand-choice-brief.md) explains the five pending
arithmetic cases in plain language. An additional forced-wildcard control proves
that native already refuses demanded private requests while upstream C swallows
its wildcard case. All original 50 outcomes remain unchanged. No proposed demand
policy or conflicting-oracle exception was implemented; API caveats remain.

## Remaining coverage checklist

Latest bounded checkpoint: [array normalization depth correction](cpu-array-depth.md),
`6644368` (implementation `e9410bf`). Tail reduction reuses its depth frame
while retaining every work charge.
Both originals and the four-slot regression match 36/36 repeated executions and
6/6 strict eval reports. The original write failure is closed in its actual CLI
mode. Full gate **831 passed, zero failed, 49 ignored**; focused, guard, baseline,
normalization and strict-audit checks pass. Negative/demand classifications remain
unchanged and their runners still return 1. The [partial printer checkpoint](cpu-array-syntax.md)
and failed candidate remain historical evidence.

Previous bounded checkpoint: [typed comparison and array-index functions](cpu-paren-index.md),
`694a210`. Both unchanged selected-main functions match strict actual core and
exact text twice. Actual C/JS generators refuse their function types, with no
artifacts; no concrete numeric inputs or array reads are claimed. Full gate:
**825 passed, zero failed, 49 ignored**; 59 focused tests and strict Clippy pass.
Normalization: 42 fixtures, 330 successes, 108 printer refusals, six F32 exclusions.
Minimum workspace headroom: 4.33 GiB, no exhaustion. No engine/policy change;
historical non-normalization receipts and unwaived diagnostic cells remain intact.

Previous bounded checkpoint: [parenthesized functions](cpu-paren-functions.md),
`5037d9a`. Three unchanged function results preserve tuple-component annotations,
nested typed arithmetic and list-element/shift structure and exact text twice.
Actual C/JS generators refuse the function results without artifacts. Full gate:
**825 passed, zero failed, 49 ignored**; 59 focused tests and strict Clippy pass.
Normalization: 40 fixtures, 314 successes, 100 printer refusals and six F32
exclusions. Minimum workspace headroom: 4.57 GiB, no exhaustion. No engine/policy
change; all non-normalization receipts remain historical and mismatches stay open.

Previous bounded checkpoint: [spacing-rule functions](cpu-spacing-functions.md),
`f485b65`. Three unchanged returned functions preserve spaced modulo/greater-than,
nested generic closers, an unused binder and an open Nat literal tail. Strict
actual-core and raw-text comparisons agree twice; actual C/JS printers refuse
the function results without artifacts. Full gate **825 passed, zero failed,
49 ignored**; 59 focused tests and strict Clippy pass. Normalization: 37 fixtures,
290 successes, 88 printer refusals and six F32 exclusions. Minimum workspace
headroom: 4.82 GiB, no exhaustion. No engine/policy change; all non-normalization
receipts remain historical and diagnostic mismatches remain open.

Previous bounded checkpoint: [default-Nat functions and checker diagnostics](cpu-default-nat-functions.md),
`0093d01`. Two unchanged function originals agree in strict normal form, actual
core and raw text twice. Actual C/JS printers refuse their function results with
no artifacts. Full gate **825 passed, zero failed, 49 ignored**; 58 focused tests
and strict Clippy pass. Normalization: 34 fixtures, 266 successes, 76 printer
refusals and six F32 exclusions. List checking and named definitions are renewed;
two authored false equalities retain diagnostic differences, now characterized
by phase and reporting cause. No engine or policy change. Minimum workspace
headroom during the monitored gate: 4.82 GiB; no exhaustion.

Previous bounded checkpoint: [checking-only list proofs](cpu-list-reverse-sort.md),
`c900e3f`. The unchanged no-main original checks exactly in both CLI compatibility
paths; strict port checking also passes. Five named definitions agree in text
and actual core twice. False reverse and lost-duplicate controls reject at the
intended proofs; raw diagnostic differences remain open. No program execution
or generated lane is claimed. Full gate **825 passed, zero failed, 49 ignored**;
58 focused tests and strict Clippy pass. The 32-fixture normalization matrix is
renewed without changing its exclusions. All runtime/demand and other checking
receipts remain historical. Minimum gate headroom: 4.83 GiB; no disk failure.

Previous bounded checkpoint: [Nat literal round-trip](cpu-nat-literal-roundtrip.md),
`4083b32`. One unchanged original now matches exact compound output and actual
normalized core under strict checking. Upstream C/JS printers actually refuse
the function-containing type. A standalone open Nat function also matches exactly;
41 direct printer controls verify syntax independently of computation. No parser,
evaluator, checker, demand-policy or safety-budget changes occur. F32 executable
observation/core-regression and strict exclusions remain distinct.

Full gate **823 passed, zero failed, 49 ignored**; 69 focused tests and strict
Clippy pass. Normalization covers 28 originals plus four controls: 250 successful
observations, 68 printer refusals and six F32 exclusions. Five-runtime baseline:
50 exact executions, including the recovered 16/17 regressions. Negatives remain
44/56 exact; demand remains 92/100 exact, six approved wildcard exceptions and
two unwaived error-order observations. Both negative runners exit 1. Nine runtime
rows and ten demand rows are renewed at `4083b32`; the other 66 runtime rows keep
their earlier source identities. No unrelated receipt is relabelled.

Repeated disk pressure was traced to retained large test binaries, multiple
dependency fingerprints and Git-metadata-triggered relinking. A bounded cleanup
manifest protects source/evidence and requires 12 GiB before broad builds. The
monitored final gate stayed above 4.89 GiB; no disk failure or raised Bend limit.

Earlier checkpoint: [checker-depth accounting](cpu-checker-depth.md),
`2f349c8`. Checking no longer enters an extra depth frame merely to infer the
same term; its work charge and recursive-child guards remain. The exact combined
source is unchanged and now matches native, port C/JS and pinned C/JS twice:
**10/10 exact** streams/status observations. The original is also in the full
positive suite. The [depth ledger](cpu-float-specials-depth-evidence.csv) retains
both historical refusals and current passing rows. Full gate: **818 passed,
zero failed, 49 ignored**; strict all-target Clippy and 32 focused tests pass.

The broader historical primary receipts below remain at `2f349c8`. Positive:
**618/630 exact**, twelve Bun-FFI gaps. Direct entries: four exact and 44
host-blocked observations in that matrix, plus six separately renewed collection
host failures. Negative: **44/56 exact**, eight resource-policy, two native-foreign
and two Bun-host gaps. Demand: **92/100 exact**, six approved C-wildcard exceptions
and two unwaived Nat-first ordering differences. Normalization: 176 successful
observations and 28 expected printer refusals, exact structure/text. Comptime:
**156/160 primary exact**, four default upstream-JS stack failures and four
supplemental stack-4096 passes. All repeats are stable, with zero timeouts.
Historical O0/O2, worker, platform and order-investigation receipts retain their
actual commits; they do not qualify this new source.

The renewed [read probe](cpu-float-read-evidence.csv) still exits 1: native/port C
match C and port JS matches JS, with three unresolved cross-backend contracts.
All ten executions exit 0 with empty stderr; no parsing policy is chosen. The
strict audit retains 366 accepted positives, 487 refused positives and all 449
nonzero-expectation fixtures rejected, including all 439 checker negatives.
No acceptance status changed, and no timeout or abnormal exit occurred. These
are status-only results, not raw-diagnostic or executable parity.

The current primary ledgers link 115 distinct originals; 1,187 lack
runtime/interpretation receipts (439 checker negatives, twelve check-only,
153 interpreter-only, 583 compiled candidates). The checking ledger now has
two exact positive acceptances and three earlier phase/construct agreements
with differing diagnostics. Thus two of the twelve no-main entries have explicit
checking qualification despite remaining absent from runtime/interpretation
ledgers. Authored controls add no inventory rows. Next: qualify
parse/array_read_annotation and parse/array_read_write in their actual executable
modes, beginning with printer applicability. Their audit status is not runtime parity.
This family needs no GPU hardware, new dependency or new F32.read policy.

`Missing` below means incomplete implementation or qualification. `Blocked`
identifies a specific unavailable host capability; it does not block unrelated
CPU work.

| Status | Area | Evidence and next action |
| --- | --- | --- |
| Passed for two originals and one authored minimizer | Concrete array syntax | Exact pure-native, generated C/JS and direct-interpreter results plus strict eval; actual core checks pass. Tail recursion no longer inflates depth; pending-depth and work guards remain enforced. |
| Passed in strict function normalization; upstream printers inapplicable | Typed comparison and array-index companions | Both original selected-main functions match core/text twice; eight repeated actual printer refusals follow four initial checks. No concrete comparison inputs or array reads claimed. |
| Passed in strict function normalization; upstream printers inapplicable | Parenthesized function companions | Three originals retain exact typed tuple/nested arithmetic/list-shift structure and text. Twelve actual printer refusals verified; no concrete invocation. |
| Passed in strict function normalization; upstream printers inapplicable | Spacing-rule function companions | Three originals retain exact modulo/comparison/open-Nat function structure and text, including the unused binder. Twelve actual printer refusals verified; no invocation or new rewrite coverage. |
| Passed in strict function normalization; upstream printers inapplicable | Default Nat function companions | Both originals retain arithmetic precedence/comparison helper structure and exact text. Actual function-printer refusals verified; no concrete invocation claim. |
| Passed checking-only; authored negative diagnostics remain different | Concrete list reverse/sort proofs | Exact original checking output and five named normal forms with independent core comparison. No main or runtime lanes; wrong reverse and dropped duplicate refuse at the expected proofs. |
| Passed, bounded | Linux build/provider/strict C portability | Standard gate plus original five-case and task/file comparisons. Keep these as regressions. |
| Passed in primary runtimes; direct IO oracle-host blocked | Application, grouping and operator chains | Six original parse fixtures match five runtimes twice, including brace annotations, Nat precedence and U32 suffix propagation. Pure direct CLI agrees; five IO entries need import.meta.require. Strict proof IO exclusions remain separate. |
| Passed in primary runtimes; direct IO oracle-host blocked | F32 literals and patterns | Three original fixtures and an authored bit/pattern control match five runtimes twice. Corrected bounded Word-pattern compilation, exact U32 printing and pinned two-stage literal rounding. Finite rejection, malformed-word printing and deep-pattern guards remain covered. |
| Passed in interpreter/native function mode; upstream printers inapplicable | F32 suffix function companions | Three unchanged originals preserve normalized structure and exact text. Core regression and parsed-output probe are distinguished; strict proof F32 exclusions stay recorded. No numeric execution or parsing-policy claim. |
| Passed in strict interpretation; upstream printers inapplicable | Brace suffix functions and Nat literals | The three brace originals retain actual-core/exact-text agreement. The new Nat round-trip and standalone open Nat control now also match; compound function-containing printer refusal is verified. This does not establish every printer form. |
| Passed in strict interpretation/native function mode; upstream printers inapplicable | U32 suffix function companions | Three originals preserve arithmetic/call nesting and reusable shift/bitwise let structure. Strict eval, actual core and raw text match; both upstream printers refuse the function results. |
| Passed in primary runtimes; direct IO oracle-host blocked | Float rounding, accumulation and storage | Three original runtime fixtures and a focused bit/text control match five runtimes twice, including signed-zero field/array storage and adjacent infinity/NaN classifications. No production fix or NaN payload-parity claim. |
| Passed in primary runtimes; direct IO oracle-host blocked | Unary/trig/hyperbolic float math | Three originals and a bit/text control match five runtimes twice. All twelve original expressions agree in exact binary32 bits; no tolerance or math-library exception. Other host profiles and NaN payload observations remain unqualified. |
| Passed in primary runtimes; direct IO oracle-host blocked | Float operations, printing and shared text inputs | Three exact originals and a bit/text control match five runtimes twice. Read None is an ordinary value, not a process error. No tolerance or NaN payload claim. |
| Passed in primary runtimes; direct IO oracle-host blocked | Finite float arithmetic and conversions | Three originals and an authored bit/text control match five runtimes twice, including 2^24 precision, adjacent even-tie integer conversions and positive truncation. No runtime F32.read or untested-host claim. |
| Passed in primary runtimes; direct IO oracle-host blocked | Float comparison digests and special values | Three originals and a factored control match five runtimes twice. Exact finite/infinity/zero bits and text are distinct from NaN text/classification; payload bits remain unobserved. |
| Passed, bounded | Combined float observation control | Exact unchanged original matches all five runtimes twice at 2f349c8. Same-term inference reuses the active checking frame, retaining fuel charges and all recursive guards. Historical refusals remain; no limit increase or exception. |
| Missing; unresolved upstream contracts | Runtime F32.read rounding, hexadecimal and embedded-NUL inputs | Three authored inputs differ between pinned C and JS; ports match their respective backends, native follows C. The separate runner retains exit 1. Choose a uniform contract or explicitly reviewed backend exception before closing these cells; no decision is made here. |
| Passed, bounded | Function-valued normalization | Fifteen original operator/recursive functions match pinned core structure and raw output twice. Nat-fuel/reusable-binding controls retain complete application results. Grouped application printing fixes two presentation differences; structural comparison and evaluation are unchanged. |
| Passed, bounded | Rewrite-term rendering | Named/unnamed, shadowed and nested motives match pinned output with all core fields preserved. Three new original normalization fixtures and four authored controls pass. Adjacent negative raw diagnostics remain open. |
| Passed, bounded | Channel terminal outcomes | New exact five-runtime control suite; cleanup and cancellation regressions retained. Broader close-wakeup scheduling has local tests but still needs selected upstream differential cases. |
| Passed in four runtimes; oracle-host blocked | Equal-duration and zero-timer ordering | Original sleep_order/sleep_timer and authored ready-before-timers control match native, port C/JS and upstream C at declared C profiles. Upstream JS requires Bun; different-duration host-sensitive cases remain unqualified. |
| Passed, bounded | JS ordinary constructor tails | All 16 positive comptime programs now match under default Node, twice each; adversarial chain and non-tail bounds remain enforced. |
| Passed, bounded | Native numeric-chain workloads | Both original 10,000-element programs match under unchanged limits, twice each, renewed after approved strict executable demand. Constructor-tail compression and available-value propagation retain bounded non-tail recursion and error controls. |
| Passed, bounded | Persistent Map/Set/Array observations | Three unchanged collection programs, five runtimes, two repetitions: exact lookup/update/order/union/deduplication/mapping outputs. This does not certify all collection operations or large inputs. |
| Passed, bounded | Out-of-band request failure contract | Original fixture matches all five runtimes twice; exact streams/status/order, cleanup, lookalikes and worker forwarding covered. Other failure classes remain open. |
| Passed, bounded | Nat overflow diagnostics and fully demanded add/double | Original overflow and authored large/boundary controls match five runtimes twice. Approved strict executable demand now validates before shallow observation; proof/data laziness and fixed limits remain separate. |
| Passed, bounded | Native/JS array-depth and JS scalar diagnostics | Original supported lanes now preserve exact stderr/status; boundary/order/cleanup controls pass. C allocation and JS element budgets remain separate refusals. |
| Implemented; narrow accepted exception | Approved arithmetic demand and wildcard refusal | Three Nat cases now agree across all five runtimes twice. Three private-wildcard controls preserve refusal and explicitly label the accepted upstream C exception. This is not byte parity with that C behavior. |
| Missing | Competing failure diagnostics | `nat_first` chooses a different first error in upstream C at O0/O1/O2/O3. Source and closure/bound controls identify a C lowering difference. The core left-to-right model excludes runtime numeric/foreign errors; no new order or exception is imposed. |
| Passed in CLI profile; oracle build limitation in O0 | Upstream Word compilation | `word_live_tail` matches at O1/O2/O3, including pinned cli_build's O3 flags. O0 fails Clang register allocation before execution; its failure stays recorded separately. |
| Missing | Parser/checker/base breadth | Latest strict audit at 67cdcdf has 366 accepted other fixtures, 497 refused other fixtures and all 439 checker negatives refused. Status-only agreement does not qualify diagnostics or runtime behavior. Separate strict exclusions from real parser/template/descent gaps. `base/list_fold.bend` now has five-runtime executable evidence despite its strict refusal; validate other candidates in their intended mode. |
| Missing | Remaining negative contracts and execution of accepted proofs/data | All ten runtime-negative fixtures now have observations; the six-family diagnostic-only gaps are closed, but resource/foreign/oracle-host gaps remain. Acceptance audit alone covers neither. Audit refusals can happen for the wrong reason; runtime-error fixtures may legitimately type-check. |
| Missing | Wider executable corpus coverage | Only selected families have actual five-runtime output comparisons. The pinned corpus contains 1,302 fixtures, including 100 IO, 101 compile and 240 flatten fixtures; these are source counts, not a common runnable denominator. |
| Missing | Arbitrary foreign effects in native Rust | Generated C/JS support companion imports; the native interpreter explicitly refuses arbitrary foreign implementations. This is an implementation gap, not a GPU requirement. |
| Missing | Broader network/file/error comparisons | The gate covers loopback and host adapters. Independently qualify selected upstream socket readiness, partial transfer and error fixtures without blindly executing the whole effect corpus. |
| Missing | Large pure-library programs and optimization | Existing whole-program string/number kits hit bounds despite passing shorter chunks. Native/JS storage/traversal and compiler specialization remain open; chunk comparisons are not whole-program parity. |
| Missing | Upstream loader/CLI/module parity | JS module marshalling, imports, browser loader and command-surface behavior need their own inventory. The Rust CLI's typed wrappers are not upstream language equivalence. |
| Missing / host-dependent | Window/audio/interactive effects | The rewrite's documented unfinished effects are separate from headless CPU checking and computation. Device integration needs an appropriate host and any dependency additions require approval. |
| Blocked on current direct-IO oracle host | Upstream `io_run` entry | Three collection direct-CLI probes fail on Node because upstream uses Bun `import.meta.require`; standalone emitted JS passes. A direct-entry receipt still needs a supported host. |
| Blocked on current JS host | Existing task/file fixtures, stderr failure and timer poller | `bun:ffi` cannot load in Node; Bun is not installed. Original sleep_order/sleep_timer and the authored zero-timer control add explicit blocked observations. Exact partial streams/statuses are retained. Upstream C remains available; no dependency installation was attempted. |
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

## Approved demand checkpoint

`208b343a286a8501fc3a479a4eb7b40bb01832d1` implements the reviewed contract
on a separate local branch after the clean rewrite-printer checkpoint. The
[checkpoint and commands](cpu-approved-demand.md) record **799 passed, zero
failed, 49 ignored**, strict Clippy, all 16 comptime originals, 13 normalization
controls and renewed positive/negative matrices. All ten authored demand controls
are recorded separately; no upstream inventory count is increased. The original
resource/host/foreign gaps remain open, as does the newly demonstrated C error-order
difference. No dependency, toolchain, pin or safety limit changed, and no push
occurred. The next independent small CPU family is the three recursive-normalization
companions identified in the checkpoint; a separate review is needed before
changing first-error order or waiving that difference.

## Constructor diagnostics checkpoint

Source `6442b2ca1362e36f84b74044091a0151fb17726d` adds five exact original
checking-negative qualifications and renews the existing false-proof controls.
See [the report](cpu-constructor-diagnostics.md) and
[category counts](cpu-coverage-summary.csv). Full gate: 866/0/49; full checking
audit: 1,302 observations, zero status changes, all 439 expected negatives
rejected. Detailed original receipts: 140; without detailed receipts: 1,162.
Four constructor/pattern diagnostic controls remain unmet, along with the
previous host/resource/API/semantic disagreements. No missing cell is a pass.
The scoped source and later documentation checkpoints are local only.

## Constructor provenance checkpoint

Source `d986964456afaf7622628144da2e2745db740691` closes the four previously
unmet constructor/pattern diagnostic controls. The
[report](cpu-constructor-origins.md) records exact source provenance through
substitution, generic pattern-load diagnostics and unchanged budgets/term
semantics. Full gate: 871/0/49; all 1,302 checking statuses unchanged. Coverage
remains 140 detailed originals / 1,162 without detailed receipts. The next
independent category is ordinary no-main checking; broader host/resource/API/
semantic disagreements remain open. No source or evidence was discarded, and
nothing was pushed.

## No-main and multi-scrutinee checkpoint

Source `2417a27107e4adb5d5546ace6cd4e854fc2a0a92`: nine no-main modules accept,
thirty named definitions match exact core/text, seventeen multi-column function
originals match, and 100 concrete argument vectors agree. Standalone compilation
now refuses missing main while default checking stays successful. See the
[report](cpu-no-main-interpreter.md) for artifact/diagnostic distinctions and
public emitter behavior compatibility.

Full gate 876/0/49; all 1,302 checking statuses unchanged. Coverage is 166 detailed
originals / 1,136 without detailed receipts. Compilation diagnostic differences
and false-fusion expected-type/source rendering remain open alongside previous
unresolved contracts. Next: that diagnostic correction or ten nested-field-match
originals. The foreign io/far_types checking row remains separate. No push.

## Compilation/rewrite and nested-field checkpoint

Source `02b244e529e978a683b7f77a79ae04089665748d` closes the preceding no-main
compilation stderr and false-fusion type/source diagnostic gaps. The
[report](cpu-compile-rewrite-diagnostics.md) records preserved rejection, the
repository caller audit and explicit external API caveats. Ten unchanged
nested-field originals match exact core/text; 171 concrete application vectors
are verified against the pinned interpreter. Function printing remains inapplicable.

Full gate **882 passed, zero failed, 49 ignored**; 76 normalization cases,
306 no-main observations, 60 rewrite/phase observations, 144 prior proof controls
and 50 baseline executions pass. All 1,302 checking statuses are unchanged.
Coverage is 176 detailed originals / 1,126 without detailed receipts, including
historical and unresolved cells. The existing resource, host, foreign, API,
F32.read and error-order requirements are unchanged; full CPU parity is incomplete.
Next independent family: four nested_multi_scrutinee originals. No push.

## Nested patterns checkpoint

Source `062c7ec150168922ff642d07b1fc72ce5c14144c` qualifies four nested multi-scrutinee,
five nested-constructor and five fallback-reconstruction originals. All fourteen
match exact interpreted core/text and 476 concrete application vectors agree
with the pinned interpreter. The [report](cpu-nested-patterns.md) distinguishes
unprintable function results from generated execution. No production source or
language contract changed.

Full gate **883 passed, zero failed, 49 ignored**; 90 normalization cases,
306 no-main observations, 60 rewrite/phase controls, 144 prior proof controls
and 50 baseline executions pass. All 1,302 checking statuses remain unchanged.
Coverage is 190 detailed originals / 1,112 without detailed receipts, including
historical and unresolved cells. Full CPU parity remains incomplete; no prior
host/resource/foreign/API/F32.read/error-order requirement is waived. Next:
field-match-order and destructuring-let applicability/qualification. No push.

## Field-order/destructuring checkpoint

Source `de31e54e8e35f35b1c0fa4c05415d8dd91ca9b10` qualifies eighteen unchanged field-order/destructuring
originals: sixteen function mains with exact core/text and 399 concrete applications,
one IO main with ten exact generated/native executions, and one exact parse/load
negative. The private binder-origin diagnostic fix preserves public ParseError,
terms, evaluation, budgets and rejection. [Report](cpu-destructure.md).

Full gate **889 passed, zero failed, 49 ignored**; 106 normalization cases,
306 no-main observations, 60 local-binder and 60 rewrite observations,
144 prior proof controls and 50 baseline executions pass. All 1,302 checking
statuses are unchanged. Direct upstream IO remains host-blocked; the 688-vector
exploratory remainder and historical unmet cells remain unqualified.

Coverage is 208 detailed originals / 1,094 without detailed receipts, including
historical/unresolved cells. Category counts are 9/11/58/130 and remaining
430/1/106/557. Full CPU parity remains incomplete; host/resource/foreign/API,
F32.read and competing-error requirements are unwaived. Next: the reusable
plus-binder family with its negative companion. Final headroom 10.04 GiB;
all evidence retained, frozen inputs/policies, no push.

## Plus-binder checkpoint

Source `e9d9b30c5029e18dc5600b407e5e0abab94a8b99` qualifies four original plus-binder IO mains and their
negative companion. Forty generated/native executions and ten helper core/text
normal forms match. The shared diagnostic boundary now preserves marker spelling,
import-filtered source excerpts and executable-compilation rejection without
changing term identity, acceptance or safety budgets. [Report](cpu-plus-binders.md).

Full gate **896 passed, zero failed, 49 ignored**. Renewed: 216 new negative,
588 constructor/origin, 306 no-main, 144 proof, 60 local-binder and 60 rewrite
observations; all 106 normalization cases and 50 baseline executions. All
1,302 checking statuses are unchanged. Direct upstream IO remains host-blocked.
The prior 688-vector exploratory remainder remains unqualified.

Coverage: 213 detailed originals, including historical/unresolved cells, versus
1,089 without detailed receipts. Remaining categories: 429 checker negatives,
one checking-only module, 106 interpreter cases and 553 compiled candidates.
Full parity remains incomplete. Next: the three recorded rewrite diagnostic
differences. Shared snapshots reduce new evidence duplication; all historical
and failure evidence survives. Frozen inputs/policies, no installs or pushes.

## Rewrite checking checkpoint

Source `4c90907a78db96cc9e46da6d25659f10b4d978cc` closes exact diagnostics for rewrite_non_equation,
rewrite_ctor_brace and dead_motive_syntax without changing their failure phases.
The first was already exact; the other two now retain normalized content and
original source through the existing failure boundary. Equality printing uses
pinned operand precedence. [Report](cpu-rewrite-checking.md).

Full gate **901 passed, 0 failed, 49 ignored**. The new 260-observation
matrix and renewed normalization/constructor/proof/no-main/runtime controls pass.
All 1,302 checking statuses remain unchanged. Coverage stays 213 detailed originals
and 1,089 without detailed receipts; linked rows include historical/unresolved
cells. Twenty checking, 99 interpretation and seven runtime originals are renewed.
The 688-vector exploration remains unqualified; existing host/resource/foreign,
F32.read, error-order and API requirements remain open. External invoke callers
must inspect exit_code; they are not audited here.

Next: annotated non-equation evidence, generated rewrite proof-binder provenance,
and missing-IH diagnostics. Frozen inputs/policies; historical evidence preserved;
no installs or pushes.
