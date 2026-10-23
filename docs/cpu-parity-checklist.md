# CPU parity against the pinned upstream

Reviewed on 2026-10-23 against upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, starting from port
`0ffb1415342df9573b3f0ff8d1c5e52285457ddc`. This is a coverage checklist,
not a completion percentage. CPU parity remains incomplete, and CPU work can
continue in the cloud without GPU hardware. Dependencies, lockfile and toolchains
were unchanged. The earlier Linux portability and task/file milestones retain
their own [portability](linux-cpu-validation.md) and
[effects](cpu-effects-validation.md) receipts.

The [CPU acceptance criterion](cpu-parity-acceptance.md) now defines the required
mode/profile evidence before the pin can be called matched. Its reproducible
1,302-fixture inventory and 51-row runtime ledger expose missing evidence;
neither is a completion percentage.
The [U32 checkpoint](cpu-word-operations.md) closes native packed division/modulo
work exhaustion. The subsequent [representation checkpoint](cpu-word-representation.md)
adds three fixtures with 30/30 exact five-runtime observations; the reported
upstream C structural-view refusal does not reproduce. The renewed gate passes
776 tests with zero failures and 49 ignored. Prior receipt limitations remain.
The [applicability report](cpu-applicability.md) classifies the 1,251 remaining
unlinked entries: 439 checker negatives, twelve check-only modules, 164
interpreter-only cases and 636 compiled candidates. Those are different acceptance
requirements, not a count of missing CPU execution tests. Pending demand choices,
resource/host gaps and complete lane/API review remain required.

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

Latest bounded checkpoint: [function-valued main normalization](cpu-function-normalization.md)
at `f1bbf98`. The three original computed-operator functions now normalize in
eval/run; all core structures match upstream twice. One raw output is exact and
two retain only rendering differences, still open under the acceptance contract.
Argument-taking APIs, executable proof separation, cancellation and safety limits
are unchanged. Full gate: **784 passed, zero failed, 49 ignored**; strict Clippy
passes. The renewed positive matrix is **308/320 exact**, with twelve known Bun
host failures. Earlier negative/comptime and multiworker profiles are retained
at their own commits, not silently renewed.

The separate interpretation ledger plus the runtime ledger link 54 distinct
fixtures; 1,248 lack either kind of execution receipt (439 checker negatives,
twelve check-only, 161 interpretation-only, 636 compiled candidates). Linked
rows may still have failed/blocked cells. Next independent coverage is
`flatten/computed_recursion_002`, `_003` and `_004`; no normalized-result receipt
exists for them yet. The acceptance criterion and pending demand choices stand.

`Missing` below means incomplete implementation or qualification. `Blocked`
identifies a specific unavailable host capability; it does not block unrelated
CPU work.

| Status | Area | Evidence and next action |
| --- | --- | --- |
| Passed, bounded | Linux build/provider/strict C portability | Standard gate plus original five-case and task/file comparisons. Keep these as regressions. |
| Structural agreement; two raw differences open | Function-valued normalization | Three original operator-match functions match the pinned interpreter's core structure. One output is byte-identical; nested match layout and parentheses differ in two. Recursive-function normal forms remain the next unqualified family. |
| Passed, bounded | Channel terminal outcomes | New exact five-runtime control suite; cleanup and cancellation regressions retained. Broader close-wakeup scheduling has local tests but still needs selected upstream differential cases. |
| Passed in four runtimes; oracle-host blocked | Equal-duration and zero-timer ordering | Original sleep_order/sleep_timer and authored ready-before-timers control match native, port C/JS and upstream C at declared C profiles. Upstream JS requires Bun; different-duration host-sensitive cases remain unqualified. |
| Passed, bounded | JS ordinary constructor tails | All 16 positive comptime programs now match under default Node, twice each; adversarial chain and non-tail bounds remain enforced. |
| Passed, bounded | Native numeric-chain workloads | Both original 10,000-element programs now match under unchanged limits, twice each. Available-value propagation and reference sharing preserve lazy/error controls; arbitrary non-tail/deeper workloads remain bounded. |
| Passed, bounded | Persistent Map/Set/Array observations | Three unchanged collection programs, five runtimes, two repetitions: exact lookup/update/order/union/deduplication/mapping outputs. This does not certify all collection operations or large inputs. |
| Passed, bounded | Out-of-band request failure contract | Original fixture matches all five runtimes twice; exact streams/status/order, cleanup, lookalikes and worker forwarding covered. Other failure classes remain open. |
| Passed, bounded | Nat overflow diagnostics and fully demanded add/double | Original overflow and authored large/boundary controls match five runtimes twice; native successor runs preserve lazy tails and unchanged limits. Partial demand remains separately unqualified. |
| Passed, bounded | Native/JS array-depth and JS scalar diagnostics | Original supported lanes now preserve exact stderr/status; boundary/order/cleanup controls pass. C allocation and JS element budgets remain separate refusals. |
| Missing | Arithmetic demand and wildcard-request contracts | Native unused/partial overflow succeeds where compiled upstream rejects in three cases. Two more cases disagree between upstream C/JS at a private-request wildcard match. Preserve safe refusal while deciding the applicable contract; do not count one-oracle agreement as parity. |
| Missing | Parser/checker/base breadth | Full strict audit remains 364 accepted positives. Separate strict exclusions from real parser/template/descent gaps. `base/list_fold.bend` now has five-runtime executable evidence despite its strict refusal; validate other candidates in their intended mode. |
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
