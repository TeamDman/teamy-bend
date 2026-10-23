# Array-depth and scalar failure contracts

Target: unchanged [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Engine: `76038aa090780ea42976361b0e2f7b0de5412d00`.
Native/JS array-depth and JS scalar-marshalling failures now preserve the exact
pinned stderr/status contract. All six diagnostic-only differences in the
six-fixture negative matrix are closed. Full CPU parity remains incomplete.

## Scoped correction

The existing guards already rejected at the expected point. Native array-depth
errors now carry a private `ArrayDepth` kind into the existing language-failure
boundary. Cleanup completes before stdout is flushed and the raw diagnostic is
written. Internal tracing context remains available; writer failures remain
internal errors. Generated JS registers actual array-depth and scalar errors in
the existing private WeakMap. Its Nat guard shares the same private constructor.
Ordinary host exceptions cannot acquire a language status by imitating text.

The [pinned array wall](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L5915)
and [scalar guard](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L567)
require these lines, respectively, with status 1:

```text
bend: an array past the deepest block class 31
bend: 55296 is not a Unicode scalar value
```

Scalar codes remain dynamic; no fixture name or message-string matching chooses
the error category. The original scalar fixture and its JS companion were copied
unchanged with Apache-2.0 attribution. Native arbitrary foreign execution remains
unsupported. No C/GPU engine, arithmetic demand, allocation/work/depth bound,
dependency, lockfile, toolchain or upstream pin changed.

Five new tests and strengthened existing cases cover both original fixtures,
exact boundary scalars, prior output, unselected branches, first-error order,
host-error lookalikes, collection, parked-channel cleanup and failed diagnostic
writes/flushes. JS depths 18 and 31 still produce the internal element-budget
refusal; depth 32 selects the language wall. Native's inclusive depth-31 and
lazy-element controls remain. The partial-demand arithmetic tests are unchanged.

## Validation

| Check | Result |
| --- | --- |
| Full Linux standard gate | **772 passed, 0 failed, 49 ignored**; format, strict Clippy, all-feature build and Node-provider checks passed. |
| Strict all-target/all-feature Clippy | Passed without exemptions after correcting three test literal-format findings. |
| Focused native/JS/foreign checks | 236 passed, zero failed, one optional profiler ignored; final JS-only rerun 21 passed. |
| Six unchanged upstream runtime negatives | **44 exact / 56 executions**, previously 38. Zero diagnostic-only differences remain; eight resource-policy, two native foreign and two Bun-host failures remain. Four C observations are inapplicable to the JS-only companion. Runner correctly exits 1. |
| Eleven authored array/scalar boundary controls | **66 exact / 74 executions**. Eight unchanged resource refusals: port C/JS at depth 18 and port C at depths 32/33, each twice. Scalar probes compare upstream/port JS; array probes include all five runtimes. |
| Seven authored Nat controls | 70/70 exact; arithmetic failure contracts retained. |
| Five minimal demand/wildcard reproducers | 50 executions reproduce all previous outputs/statuses; unresolved differences remain explicit in the separate proposal. |
| Nineteen positive/control cases | 184 exact / 190 executions; six established upstream-JS Bun failures retained. |
| Sixteen positive comptime fixtures | 160 exact / 164 executions; four default upstream-JS stack failures retained, supplemental stack-4096 runs pass. Every port execution passes, including original hand/tmpl. |
| Nat/request worker controls | 18/18 each, including C at 1, 2 and 4 workers. |
| Merged stream order | Six new array/scalar observations and ten renewed Nat observations pass. |
| Generated-runtime cleanup | 11 probes pass, including repeated C invocation and cleared JS IO context. |

All repeated raw streams/statuses are deterministic; zero timeout. Negative run
directories retained no files. Steps retain 60-second deadlines. All after
receipts use binary SHA256
`f5ce318a9ec7eceb5327b780e432b7af0578725caececea66af9020efac725e2`.
All 39 upstream ledger fixture hashes were checked and their receipts renewed.
The authored controls add no upstream inventory rows. The original baseline
receipt on clean `de2c72e` is retained, as are all earlier milestone logs.

Initial focused failures were four grammar mistakes in new authored tests and
one old assertion expecting native `Err`; these were corrected to exercise the
actual source grammar and new exact status/streams. The first standard gate
already passed all tests; a subsequent stricter all-target Clippy found the three
test literals, fixed without changing their values. Final lint and affected JS
checks pass. No failed attempt was erased.

## Remaining requirements and API implications

The six-fixture matrix's remaining 12 observations are **not passes**. C still
refuses large arrays at its lower resource ceiling; C/JS/native stack bounds
remain different from the upstream fixture's failure contract. Native arbitrary
foreign implementations are absent, and Bun is not installed for the affected
upstream JS oracle. Neither resource refusals nor host limitations are waived.

The five arithmetic demand/oracle disagreements have a separate
[source-backed proposal](cpu-demand-contract.md), exact results and standalone
reproducers. The recommendation is strict live executable values and uniform
private-request refusal, with the pinned C wildcard anomaly explicitly recorded.
It is **research only, awaiting review**. No eager forcing, request acceptance
or conflicting-oracle exception has been implemented.

The [acceptance criterion](cpu-parity-acceptance.md) still requires complete lane
review and qualified behavior for the fixed pin. Only 39 of 1,302 inventory rows
have linked current runtime receipts; 1,263 unlinked rows are not a bug count.
Independent CPU coverage can continue without changing the proposed contract.
The unlinked [sleep_order fixture](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/io/sleep_order.bend)
is a concrete next candidate: four equal-duration timers must print in registration
order. Qualify it under explicit serial/multiworker C profiles and supported JS/
native lanes, retaining any missing oracle-host support. Broader channel wakeup
and host-error cases still need applicability review. No new execution of this
family is claimed here, and it needs no GPU.

Native array-depth language failure now returns `Ok(1)` rather than `Err`, just
like the existing Nat/request boundary. The sole production repository caller,
the CLI run operation, forwards the status; no signature changed. External
callers must inspect the code. The earlier private KernelError field still
breaks unknown external struct literals/exhaustive patterns, and derived
Debug/Eq includes its kind. External ecosystem compatibility is unverified.
Public message/Display remain; ordinary host diagnostics are not suppressed.

## Commands and evidence

With existing activated tools, `BEND_GPU=off`, `CARGO_NET_OFFLINE=true`:

```powershell
cargo test --locked --no-fail-fast --lib --test runtime_numeric --test compiler_executable_javascript --test compiler_executable_foreign
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --test compiler_executable_javascript
cargo build --locked --all-features
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
```

Private `cpu-array-scalar-diagnostics` evidence contains exact commands, original
and renewed receipts, source/binary hashes, guard/demand probes, worker/order/
cleanup results and gate logs. No private paths or raw host diagnostics are
committed here. Fresh Windows/macOS, GPU/hardware profilers, Bun and broader
unreviewed corpus/API lanes remain unrun. There is no executor blocker and no
push; all prior local history remains intact.
