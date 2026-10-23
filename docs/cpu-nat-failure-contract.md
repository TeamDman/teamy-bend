# Nat overflow failure contract

Target: unchanged [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Engine: `b26ca5e304d47707a8a86b9ba6567bfc42cdaa6d`.
The original Nat-overflow fixture now matches all five CPU runtimes twice.
This closes its diagnostic gap; full CPU parity remains incomplete.

## One Nat law, several arithmetic paths

Pinned [arithmetic lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L254)
and [C guards](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L432)
use the same 48-bit bound for multiplication, addition, doubling and successor
construction. The [original fixture](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/nat_overflow.bend)
requires empty stdout, exactly `bend: a Nat past the largest immediate 2^48-1\n`
on stderr and status 1. The earlier multiplication optimization already reached
the correct guard; its internal diagnostic wrapper still differed from upstream.

Native execution now classifies multiplication and demanded reconstructed-prefix
overflow as `NatOverflow`. Generated C classifies both `nat_mul` and `nat_chk`,
covering all their arithmetic callers. Generated JS associates actual numeric
guard errors with a private WeakMap entry; an ordinary Error with identical text
cannot acquire the language diagnostic. None of these changes tests a filename
or infers error identity from message text.

The existing boundary flushes preceding stdout and emits the language status and
line. Native cleanup completes before reporting; C preserves the category through
worker completion/cancellation and normal cleanup. Ordinary host exceptions,
writer errors, cancellation, invalid internal packed-Nat allocation and exhausted
budgets retain internal diagnostics. The native trace records the failure category
and detailed message. No arithmetic algorithm or limit changed. The shared C
value helper has a CPU failure hook; its device fallback retains the former
refusal behavior, with GPU execution unqualified.

## Validation and exact improvement

| Check | Result |
| --- | --- |
| Full Linux standard gate | **761 passed, 0 failed, 49 ignored**; format, strict Clippy, all-feature build and Node-provider contracts passed. |
| Strict all-target/all-feature Clippy | Passed without suppression. |
| Focused checks | 252 passed, zero failed, one optional profiler ignored. |
| Original Nat-overflow fixture | **10/10 exact** across five runtimes, twice each. |
| Six-fixture negative matrix | **38 exact / 56 executions**, up from 32; six diagnostic differences, eight safety-policy refusals, two native foreign failures and two upstream Bun failures remain. Four C observations are inapplicable to the JS-only companion. Runner correctly exits 1. |
| Authored seven-case Nat matrix | **66/70 exact**, up from 34/70. All 32 diagnostic mismatches corrected; four native work-limit refusals remain. |
| Authored worker Nat-overflow case | **18/18 exact**, including upstream/port C at 1, 2 and 4 workers. |
| Renewed request-worker control | 18/18 exact on this engine, including both C implementations at 1/2/4 workers. |
| Merged stdout/stderr order | 10/10 supplemental executions preserve `before` before the Nat diagnostic, including port C at 1/2/4 workers. |
| Generated-runtime cleanup | 11 probes passed, including repeated C invocation and cleared JS IO context. |
| Positive/control renewal | 184 exact / 190 executions; six established upstream-JS Bun failures retained. |
| Positive comptime renewal | 160 exact / 164 executions; four default upstream-JS stack failures retained, with supplemental stack-4096 runs passing. Every port run passes, including original hand/tmpl. |

Six new tests and strengthened adjacent assertions cover the shared Nat guard
paths, exact output/status, native unused overflow and first-demanded failure,
parked-channel cleanup, failed diagnostic writes/flushes, ordinary host errors
with matching text, and CPU worker propagation. Existing boundary products,
origin guards, false-proof, cancellation and work-limit regressions remain.

All 39 upstream ledger fixtures were renewed on one binary snapshot. Authored
controls do not add upstream inventory rows. Every repeated stream/status pair
is deterministic; no timeout occurred and negative execution directories retained
no files. Differential generation/build/execution steps have 60-second deadlines.
The before matrix uses the retained `b640bf5` binary rather than rebuilding old
source or modifying the reference checkout.

## Newly explicit native arithmetic gaps

Let `m = 281474976710655`, the largest valid Nat. These authored cases are separate
from the six-fixture negative matrix and remain required compatibility work:

| Expression, observed through Nat.show | Native Rust | Upstream C/JS and port C/JS |
| --- | --- | --- |
| `Nat.mul(2^24, 2^24)` and `Nat.mul(m, m)` | Exact overflow line/status | Exact overflow line/status |
| `Succ{m}` and `Nat.add(1n, m)` | Exact overflow line/status | Exact overflow line/status |
| `Nat.add(m, 1n)` | Work budget exhausted | Exact overflow line/status |
| `Nat.double(2^47)` | Work budget exhausted | Exact overflow line/status |
| `m` | Full maximum printed successfully | Full maximum printed successfully |

The powers in this table are mathematical shorthand; the saved Bend controls
construct them from U32-to-Nat values and multiplication. The two native gaps
also occur before this diagnostic change. Native add/double still unfold their
checked recursive bodies, consuming work before reaching the bound. This is an
algorithmic compatibility gap expressed as a safe refusal, not evidence that the
program violates the Nat law at a different value. A passing safety regression
must not relabel either lane as parity. Optimizing them requires preserving lazy
constructor results and error order; increasing limits or eagerly inspecting
unused operands would not resolve the intended contract.

## Review of Rust API consumers

The [CLI run operation](../src/cli/run/run_cli.rs) is the only production caller
of `ExecutableBook::run_main` found in the tracked repository. It forwards
`Ok(code)` to `CliOutput::exit_status`; the library entry reads that code and
[main](../src/main.rs) exits nonzero after guards are dropped. Thus `Ok(1)` emits
no extra Eyre wrapper and does not turn program failure into process success.
The other discovered calls are tests. The Node-provider crate has no use of
these APIs or dependency on the root Rust library.

`run_main` already returned nonzero `Ok` values for Halt/deadlock; request refusal
and now Nat overflow follow the same driver-outcome contract. Callers must inspect
the status, not only `Result::is_ok`. Its public documentation now says this
explicitly. Pure native main retains the proof-normalizer path; arbitrary native
pure-effect execution is not qualified by this change.

No tracked consumer constructs a `KernelError` value with a struct literal.
Nevertheless, the private field introduced by `b640bf5` is a **source compatibility
break** for unknown external struct literals and exhaustive patterns. Derived
Debug and equality now include the private category. Public message access and
Display remain available; `KernelError::new` remains crate-private. This review
adds documentation, not a claim of external ecosystem compatibility or a new
public construction API. No package release or publication occurred.

## Remaining work and reproduction

Within the six-fixture matrix, exact diagnostic gaps now remain only for native/JS
array depth and JS scalar marshalling. C's lower array ceiling, bounded stack
refusals, native arbitrary foreign support and unavailable Bun oracle lanes remain
separate. Broader corpus/API/profile review is still missing: the ledger links
39 of 1,302 inventory rows, leaving 1,263 without a linked current runtime receipt.
That is not a bug count or completion percentage.

The next arithmetic step is bounded native Nat.add/double optimization using the
newly reproduced cases while preserving demand. Array/scalar diagnostic closure
is a separate small failure-contract task. The existing
[acceptance criterion](cpu-parity-acceptance.md) is unchanged; none of these gaps
is waived. No dependency, lockfile, toolchain or upstream pin changed.

With the existing activated tools, `BEND_GPU=off` and `CARGO_NET_OFFLINE=true`:

```powershell
cargo test --locked --no-fail-fast --lib --test runtime_numeric --test compiler_executable_c --test compiler_executable_javascript --test compiler_executable_c_parallel
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
```

Private `cpu-nat-failure-contract` evidence retains exact commands, before/after
sources and receipts, binary hashes, worker/order/cleanup probes, API-consumer
search results and gate logs. Earlier evidence remains intact. GPU, fresh
Windows/macOS and unavailable Bun execution are unrun. No push was performed.
