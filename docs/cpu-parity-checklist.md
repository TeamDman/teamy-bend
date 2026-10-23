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

## Remaining coverage checklist

`Missing` below means incomplete implementation or qualification. `Blocked`
identifies a specific unavailable host capability; it does not block unrelated
CPU work.

| Status | Area | Evidence and next action |
| --- | --- | --- |
| Passed, bounded | Linux build/provider/strict C portability | Standard gate plus original five-case and task/file comparisons. Keep these as regressions. |
| Passed, bounded | Channel terminal outcomes | New exact five-runtime control suite; cleanup and cancellation regressions retained. Broader close-wakeup scheduling has local tests but still needs selected upstream differential cases. |
| Missing | Native and JS whole-program depth/resource parity | Two unchanged 10,000-element comptime workloads fail while upstream C and qualified upstream JS succeed. Reduce the continuation growth before changing representation; retain fail-closed budgets. |
| Missing | Parser/checker/base breadth | Full strict audit remains 364 accepted positives. Separate strict exclusions from real parser/template/descent gaps; e.g. `base/list_fold.bend` reports a template-position limitation. Validate each candidate in its intended mode. |
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

The next small CPU step is to reduce the shared `comptime/hand.bend` and
`comptime/tmpl.bend` continuation-depth failure, identify where bounded explicit
traversal can replace growing recursive continuations, and validate the original
whole programs across native/JS/C. Do not simply increase the safety limits or
start newer-upstream adoption before the pinned behavior is understood.

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

To reproduce the remaining gap directly, run the unchanged `hand.bend` and
`tmpl.bend` under `teamy-bend run`, then generate executable JavaScript and C
with `compile --executable --target javascript|c`. Compare against the pinned
upstream emitter; use `node --stack-size=4096` for the upstream JS oracle and
`--threads 1 --gpu off` for C. Do not change the port's resource budgets.

The frozen `Cargo.lock` SHA256 is
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
No source in upstream or teamy-docs was modified. This review does not establish
full CPU parity, new GPU evidence, or permission to adopt newer dependencies.
