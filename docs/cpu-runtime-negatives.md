# CPU runtime-negative comparison

Target: unchanged [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Engine candidate: `d20c904d6e9ef563ca186c5ed1dcceec938e0bfc`.
This is a bounded failure-contract checkpoint, not complete CPU parity.
Dependencies, toolchains, safety limits and reference sources remain frozen.

## Demonstrated defect and correction

Before this change, native execution of `io/array_depth_bound.bend` printed
`7\n` and exited 0. Both upstream compiled runtimes reject depth 32 before any
output. Native Rust evaluated the ordinary shared `ANode` tree without the
executable array-depth law; cheap structural sharing did not make that program
valid. The correction validates the demanded depth of sealed `Array.new` against
31, then uses its original checked constructor body. It preserves lazy elements,
erased arguments, partial applications, original budget charging and cancellation.
It does not install that execution rule in the proof kernel or change user
lookalikes. The original pinned fixture is retained unchanged with attribution.

Six new regressions cover the original program, the inclusive 31/32 boundary,
large packed depths, reconstructed unary depths with poisoned unused tails,
partial applications and error order, cancellation/work limits, sealed origins,
proof-mode separation, collection at every transition, and parked-channel cleanup.
The fixed native fixture now exits 1 with empty stdout. Its CLI still renders
an Eyre diagnostic containing `an array past the deepest block class 31`;
raw stderr does **not** match upstream and its ledger cell remains open.

## Exact outcomes and classification

Each eligible runtime ran twice with a 60-second deadline. C explicitly uses
`--threads 1 --gpu off`; `BEND_GPU=off` is scoped to children. The synthetic
`BEND_TEST_NO_SUCH_VAR_09AF` key is removed only from each child environment.
Both streams and exit status were deterministic; no timeout or signal death
occurred. All sources and the sole foreign companion were copied byte-for-byte.

The upstream expected stderr lines below end in a newline. Stdout is empty
except where specified. Upstream C and JS match these contracts except the
explicit host-blocked JS cell.

| Fixture | Upstream execution contract | Port C | Port JS | Native Rust |
| --- | --- | --- | --- | --- |
| `io/array_depth_bound.bend` | Exit 1; `bend: an array past the deepest block class 31` | Safety limit: `array budget exhausted`; current allocation class limit is 17 | Same depth refusal, different diagnostic prefix | Fixed acceptance defect; same depth refusal, Eyre diagnostic differs |
| `io/marshal_char_scalar.bend` | Exit 1; `bend: 55296 is not a Unicode scalar value` | Not required: upstream fixture supplies only a JS foreign companion | Same invalid scalar, different prefix | Unsupported arbitrary JS foreign implementation `echo`; the scalar law is not reached |
| `io/request_out_of_band.bend` | Stdout `3\n`; exit 1; `bend: runtime fail-stop` | Refuses request inspection; different diagnostic | Refuses request inspection; different diagnostic | Refuses request inspection; different diagnostic |
| `io/stack_fault_trap.bend` | Exit 1; `bend: memory fault (machine stack overflow?)` | Continuation budget refusal | Call-depth budget refusal | Continuation-depth budget refusal |
| `io/stderr_failure.bend` | Stdout `to-stdout\n`; stderr `to-stderr\nNo such file or directory\n`; exit 2 | Exact match | Exact match | Exact match |
| `run/nat_overflow.bend` | Exit 1; `bend: a Nat past the largest immediate 2^48-1` | Nat multiplication refusal; different diagnostic | Same Nat bound; different prefix | Step budget exhausted before establishing the Nat-overflow law |

The request fixture never prints `SECRET`; the stderr failure never prints
`unreached`. Those ordering observations do not erase diagnostic differences.
The stack fixture's smaller port limits intentionally prevent a host stack
fault. Those safe refusals remain distinct from the upstream failure contract;
no limit was removed or relabelled as a passing oracle comparison.

For stderr failure, upstream JS prints its expected two initial stream prefixes
and then fails loading `bun:ffi`, exiting 1 instead of 2. Bun is not installed;
no installation, shim or upstream edit was attempted. Native `echo` support is a
missing implementation, not this Node/Bun host blocker. The direct upstream IO
CLI remains separately unqualified as recorded in the acceptance document.

Of 60 potential observations, four C observations are inapplicable because of
the JS-only companion. The **56 actual executions** comprise:

- **26 exact stream/status matches** (20 upstream observations and six port observations).
- **16 diagnostic mismatches**, with refusal/order agreement only.
- **10 bounded safety refusals** (six stack, two C array, two native Nat).
- **Two unsupported native foreign calls** and **two upstream-JS host failures**.

Every nonmatching required cell remains open. `compare-cpu-negatives.ps1`
therefore exits **1**, even though it completes the full collection successfully.
Unexpected results, changed repetition bytes and leftover artifacts are recorded
as such and cannot count as passes. A shared nonzero status is never sufficient.

## Cleanup and validation

All 56 child processes exited and their initially empty execution directories
remained empty. The native regression additionally checks that a depth refusal
releases parked scheduler/channel roots under forced collection. Five port-C
instrumented copies run their original program twice in one process and verify
coordinator storage, host owner, failure guard, IO/task context and lock release
after each invocation. Six port-JS copies verify the IO context is null after
failure. These 11 probes preserve the recorded stream/status bytes. They inspect
port ownership boundaries, not arbitrary upstream heap leak freedom.

The standard gate passes **744 tests, zero failures, 49 ignored**; strict
all-target/all-feature Clippy passes. The refreshed 19-fixture positive/control
suite retains **184 exact matches and six known upstream-JS host failures** over
190 executions. All 16 positive comptime programs were renewed: **160 matches
across 164 executions**, including supplemental upstream JS stack-4096 runs;
four default upstream-JS stack failures remain recorded. All port comptime
executions pass, including original `hand.bend` and `tmpl.bend`. No timeouts.
GPU and fresh Windows/macOS/multiworker qualification were not run.

## Reproduction and remaining work

```powershell
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-evidence-dir> -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
```

Private `cpu-runtime-negatives` evidence preserves before/after captures, exact
commands, source/binary hashes, supplemental cleanup probes and prior checkpoints.
The [ledger](cpu-runtime-evidence.csv) now links 39 upstream fixtures, including
these unresolved cells; it is not a count of fully qualified fixtures.

The next small CPU correction is the native `Nat.mul` resource gap demonstrated
by the unchanged overflow fixture. Investigate a sealed, demand-preserving
compact-number path with zero/unused-operand, boundary, error-order and work-limit
controls. Do not raise the work budget to obtain the expected error. Diagnostic
contract alignment, C allocation coverage, foreign support, API/lane review and
multiworker/host qualification remain separate open items. The
[acceptance criterion](cpu-parity-acceptance.md) is unchanged: required missing,
failed and blocked cells must reach zero before the named scope is complete.
