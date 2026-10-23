# CPU timer-order qualification

Target: unchanged [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Candidate: `6bfc10ac69cf523dd0fb1c302e92c9ee63e26362`.
The native and generated C/JS engines are unchanged from `42dd816`.
This checkpoint extends the differential runner and evidence; no demonstrated
timer defect required an implementation change.

## Required order without exact-time assertions

The runner's new `scheduler` suite includes two unchanged upstream fixtures and
one authored control already exercised by the native/JS scheduler tests:

| Case | Required stdout, status 0 and empty stderr |
| --- | --- |
| [sleep_order](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/io/sleep_order.bend) | `a\nb\nc\nd\n` |
| [sleep_timer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/io/sleep_timer.bend) | `woke\n` |
| [ready-before-timers](../tests/fixtures/cpu-scheduler/ready-before-timers.bend) | `main\nready\na\nb\n` |

Equal-duration timers are registered in order, so their deadlines are ordered or
tied; the queue retains registration order. Ready tasks drain before timer
promotion, including zero timers. Spawn appends work without yielding the parent,
and the event loop waits for children after main returns. These are ordering
requirements, independent of an exact number of elapsed milliseconds.

The pinned [C poller](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L5462)
waits for the earliest deadline, then scans parked actions in registration order
and promotes every due action. The
[JS poller](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L6136)
uses the same ordered filtering. If the host oversleeps, multiple timers can
become overdue together; registration order then need not equal deadline order.

Consequently, the different-duration
[spawn_sleep fixture](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/io/spawn_sleep.bend)
is not added as a fixed-order regression here. Its golden `main,b,a,done` is a
normal-host observation, not an unconditional ordering guarantee under arbitrary
descheduling. This does not qualify the fixture or authorize arbitrary output
permutations. Its permitted outcomes still need a declared host profile or
registration/deadline evidence before full qualification.

Existing [controlled-clock regressions](../src/runtime/executable/scheduler_tests.rs)
passed for both the earliest-deadline case (`b,a`) and an oversleep where both
timers are due (`a,b`), as well as ready-before-overdue work, flush-before-wait,
deadline overflow, bounded cancellation waits, idle work-budget preservation,
failed waits and collected continuation cleanup. These tests assert simulated
clock events, not wall-clock race outcomes. No elapsed-time assertion was added.

## Exact results

| Check | Result |
| --- | --- |
| Scheduler suite, five runtimes, three repetitions | **36 exact / 45 executions**; nine upstream-JS Bun-host failures. |
| Additional upstream/port C at 2 and 4 workers, three repetitions | **36/36 exact**. Combined with the serial suite: 72 exact / 81 observations, nine host-blocked. |
| Full Linux standard gate | **772 passed, 0 failed, 49 ignored**; exit 0. Format, strict Clippy, all-feature build and Node-provider checks passed. |
| Strict all-target/all-feature Clippy | Passed, exit 0. |
| Focused scheduler/native/JS/C checks | 217 passed, zero failed, one optional profiler ignored. |
| Expanded positive suite, 22 cases twice | 208 exact / 220 executions; twelve upstream-JS Bun failures. This includes the scheduler cases at two repetitions and is separate from the three-repeat receipt above. |
| Six runtime-negative fixtures | 44 exact / 56; eight resource-policy, two native foreign and two Bun gaps remain. Runner exit 1 is intentional. |
| Sixteen positive comptime fixtures | 160 exact / 164; four upstream-default-JS stack failures retained, supplemental stack-4096 runs pass. All port runs pass, including hand/tmpl. |
| Pending demand-contract cases | All 50 observations unchanged. Ten additional forced-wildcard observations isolate the conflicting C guard; see the decision brief. |

All repeated streams/statuses are deterministic, with zero timeouts. C profiles
are explicit `--threads 1/2/4 --gpu off`; these small event-loop cases qualify
those configurations, not every parallel computation. Generation, compilation
and execution deadlines remain 60 seconds. Port C retains strict warning flags.
All final snapshots have binary SHA256
`c51608d576f06322e3f8738dc4237ee922057da70b4c877a5f08d1efacb4fa0e`.

The unmodified upstream JS poller calls `io_sys().poll` even for a zero-duration
wait; `io_sys` loads `bun:ffi`. Under installed Node 24.19.0 it fails before waking.
The two original fixtures have empty stdout before that failure; the authored
control prints exactly `main\nready\n` first. Those exact partial streams, status 1
and missing-module diagnostics are retained. No Bun install, shim or modified
upstream runtime was used. These are blocked oracle lanes, not matching outcomes.

## Coverage and next work

The ledger now links **41** of the 1,302 upstream inventory rows, leaving **1,261**
without linked current receipts. The two new timer rows pass in native, port C/JS
and upstream C but remain blocked in upstream JS, so neither is fully qualified
across all required lanes. The authored control adds no upstream inventory row.
All prior 39 rows were renewed on this candidate; the acceptance rules are
unchanged. This is not a full-parity declaration or a completion percentage.

Remaining work includes the twelve negative observations, pending arithmetic
demand/private-request choices, broader checker/evaluator/API and host coverage,
native arbitrary foreign support, and unavailable Bun/direct-IO oracle lanes.
Mixed-duration timing behavior remains unqualified as described above. Independent
CPU verification can continue; unavailable GPU hardware is not a CPU blocker.

The [plain-language decision brief](cpu-demand-choice-brief.md) separates unused
arguments, partially inspected Naturals and private-request wildcard matches.
It supplies minimal Bend examples, exact results, source grounding and tradeoffs.
It is research only: no demand policy or conflicting-oracle exception was adopted.
The earlier KernelError external literal/pattern compatibility caveat remains;
callers must inspect nonzero run_main Ok statuses. This checkpoint changes no API.

## Reproduction

With activated, frozen tools, `BEND_GPU=off` and `CARGO_NET_OFFLINE=true`:

```powershell
cargo test --locked --no-fail-fast --lib --test runtime_scheduler --test compiler_executable_scheduler --test compiler_executable_c_parallel
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-scheduler-dir> -Suite scheduler -Repetitions 3
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
```

Private `cpu-timer-order` evidence retains exact commands, hashes, preflight and
final receipts, C worker profiles, controlled-clock test logs, demand controls,
and gate exits. Previous evidence is intact. Fresh Windows/macOS, Bun, GPU and
hardware profilers remain unrun. All commits are local; no push, dependency,
toolchain, lockfile, pin or production runtime change occurred.
