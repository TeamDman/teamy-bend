# Out-of-band request failure contract

Target: unchanged [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Engine candidate: `b640bf5c4ffa7bb148898ee40f7691c9b55467a2`.
This closes the request diagnostic gap from the native Nat multiplication
checkpoint. CPU parity remains incomplete; no budget, dependency, toolchain,
upstream source or GPU implementation changed.

## Language contract and cause

The original [request fixture](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/io/request_out_of_band.bend)
first decodes a real Emit and prints `3\n`. Attempting to match the pending
foreign request must then emit exactly `bend: runtime fail-stop\n` on stderr
and return status 1. The request must not run, resume, become Halt data, or reach
a fallback arm. Both pinned compiled upstream runtimes already enforce this law.

All three port runtimes already refused safely. Native returned a KernelError
through the CLI's Eyre report; generated JS and C added their internal error
prefixes. These were observable diagnostic mismatches, not resource-policy
refusals or six independent semantic defects across repeated runs.

## Correction and boundaries

Native errors now carry a private failure kind alongside their detailed message.
Only guards inspecting an actual private Request assign the language kind. The
IO driver first releases scheduler, channel, job, file and network state, then
flushes stdout, writes the raw diagnostic, flushes stderr and returns `Ok(1)`.
Detailed internal context remains available through a structured debug event
in the existing tracing setup. Writer errors, cancellation and exhausted budgets stay errors.

Generated JS recognizes the existing WeakSet-authenticated request object.
Generated C uses a failure enum and forwards it with the first worker failure;
coordinator cancellation/completion cannot discard its category. The ordinary
error entry point still produces internal diagnostics, even for identical text.
C flushes pending stdout before the language line and retains normal unwind and
worker-join cleanup. Failed stdout flushes use the internal-error path.

This is a small structured boundary, not string matching or a fixture-name
special case. No global error-prefix substitution or logging/CLI rewrite occurred.
The private kind added to public KernelError prevents downstream struct-literal
construction; existing message reads and Display remain. Rust callers now receive
`Ok(1)` for this language failure rather than `Err`, as for channel deadlock.
Pure native main still follows its existing proof-normalizer path; this checkpoint
does not qualify arbitrary effect execution through that API.

## Regression and differential evidence

The original upstream source is copied unchanged with Apache attribution. Its
regression checks native API/CLI, generated JS and C, separate stream bytes,
status and merged-stream order. Adjacent tests retain request lookalikes as data,
reject authentic requests after collection or channel transfer, preserve lazy
unused errors and prior effects, and distinguish ordinary host exceptions with
request-like messages. Forced collection plus failed diagnostic writes/flushes
checks channel cleanup. C worker failures are verified at two and four workers;
the probe requires evidence that the typed error crossed the worker boundary
and that all checked coordinator contexts and workers were released.

| Check | Result | Qualification |
| --- | --- | --- |
| Full Linux standard gate | **755 passed, 0 failed, 49 ignored** | Format, strict Clippy, all-feature build and real Node provider also passed. Five new tests; existing assertions strengthened. |
| Strict all-target/all-feature Clippy | Passed | No warning suppression. |
| Focused integration group | 107 passed | Plus one focused native channel/writer cleanup test. Full gate includes both. |
| Original request fixture, five runtimes, twice | **10/10 exact** | stdout `3\n`, stderr `bend: runtime fail-stop\n`, status 1. |
| Authored fork/request control | **18/18 exact** | Native/JS twice; both C implementations twice at 1, 2 and 4 workers, GPU off. stdout `before`, same stderr/status. This does not qualify the whole multiworker corpus. |
| Six-fixture negative matrix | **32 exact; 24 unresolved, out of 56 executions** | 12 diagnostic, 8 safety-policy, 2 native foreign and 2 upstream Bun failures. Four C observations are inapplicable for the JS-only companion. Runner correctly exits 1. |
| Positive/control renewal | 184 exact; 6 known upstream JS host failures, out of 190 | No new difference. |
| Positive comptime renewal | 160 exact out of 164 | Four default upstream-JS stack failures; supplemental stack-4096 runs pass. Every port case passes, including original hand/tmpl. |
| Supplemental generated-runtime cleanup | 11 passed | C repeated invocation releases checked contexts; JS IO context is cleared. Streams/status preserved. |

All comparisons use the same binary snapshot. All 39 upstream ledger fixtures
were renewed on this engine; the authored worker control adds no upstream rows.
Results repeat deterministically, with zero timeouts and no remaining files in
the negative execution directories. Child generation/build/execution deadlines
are 60 seconds; the new cross-runtime and worker regression children use
30-second deadlines.

The first gate found two stale request assertions; both now require the exact
language result while retaining collection/no-effect checks. Initial focused
runs caught a test-only C name collision, an early writer-fault trigger and
strict test-style lints. Their logs remain alongside the final passing evidence;
no runtime failure was hidden or test requirement removed.

## Remaining requirements

The remaining negative diagnostic observations are Nat overflow (all three port
lanes), array depth (native/JS), and scalar marshalling (JS). These remain
observable contract mismatches. C array class 17 and the three bounded port stack
refusals are explicit resource-policy differences, still unqualified. Arbitrary
native foreign calls remain missing implementation; the scalar fixture is not
excluded by proof-mode policy.

Upstream JS still needs unavailable `bun:ffi` for four effect fixtures across the
suites, and the direct IO CLI uses Bun's `import.meta.require`. Those are specific
oracle-host blockers; no install was attempted. The default Node stack limit on
two positive comptime fixtures is separately recorded, with a qualified larger
oracle stack profile. None of these is a reason to stop unrelated CPU work.

The runtime ledger still covers 39 of 1,302 source inventory rows; 1,263 rows
lack a linked current runtime receipt. Many need a different execution/checking
lane, so this is neither a bug count nor a completion percentage. Full lane/API,
loader, broader file/network and host-profile review remains missing.

The next bounded CPU step is to qualify exact array-depth and Nat-overflow
language diagnostics using the structured boundary, with separate assessment of
C's lower array allocation ceiling. Do not remove a safety limit to make a
negative match. The [acceptance criterion](cpu-parity-acceptance.md) still requires
lane/API review, exact required contracts, applicable multiworker/host profiles,
and zero missing or blocked cells before declaring the upstream pin matched.

## Reproduction

With the existing activated tools, `BEND_GPU=off` and `CARGO_NET_OFFLINE=true`:

```powershell
cargo test --locked --test runtime_request_failure
cargo test --locked --lib request_refusal_and_diagnostic_writer_errors
cargo test --locked --test compiler_executable_c_parallel request_failure_crosses_cpu_workers
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
```

Private `cpu-request-fail-stop` evidence retains commands, raw streams/statuses,
source and binary hashes, repeated comptime/worker probes, cleanup receipts,
initial failed checks and final passing logs. Earlier evidence is intact.
GPU and fresh Windows/macOS qualification are unrun; neither blocks further
bounded Linux CPU work. No push or newer upstream adoption occurred.
