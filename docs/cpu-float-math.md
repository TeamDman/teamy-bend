# Float math: exact host-profile qualification

Coverage source: `85402873712a38a583e7718d04e1e4651edffa56`, local branch
`qualify/cpu-float-math-20261023`, based on `7dd7580`.
Target: [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The three originals and a paired bit/text control agree across the five primary
runtimes. No production defect or cross-oracle math discrepancy was demonstrated
on this host. No tolerance, new compatibility exception, production change,
dependency/toolchain change, raised budget, installation or push is included.

## Applicability and exact original results

The three Base-importing IO programs are compiled candidates under the pinned
[gate selection](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L48).
Both upstream targets actually generate successfully; the
[result printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
selects the IO driver. This does not qualify pure F32 proof-mode normalization.
Original bytes, comments, goldens and Apache-2.0 attribution are unchanged.

| Upstream original | Exact stdout bytes |
| --- | --- |
| [compile/float_unary_math.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/compile/float_unary_math.bend) | `2.7182817 0.99999994 3 3 1.4142135\n` |
| [compile/float_trig.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/compile/float_trig.bend) | `0.84147096 0.5403023 1.5574077\n` |
| [compile/float_tanh.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/compile/float_tanh.bend) | `1.1752012 1.5430807 0.7615942 1\n` |

Native Rust, port C, port JS, upstream C and emitted upstream JS match each
original twice, exit 0 and empty stderr: **30 exact executions**. The unary
fixture's comment about closing on 1.0 does not override its actual golden:
`log(exp(1))` prints `0.99999994`. The trig comment's “six digits” likewise does
not substitute a different formatting contract for the current source/output.

All three direct upstream IO entries still exit 1 with empty stdout and exactly
`TypeError: require is not a function\n`, twice. Pinned
[io_run](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1271)
requires Bun's import.meta.require, unavailable under Node. These six direct
observations remain blocked; emitted CommonJS success does not erase them.
No upstream source adaptation or Bun installation was made.

## What must match, and what remains host-dependent evidence

Pinned [F32 representation](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L57)
is a 32-bit Word. [F32.bits](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1602)
returns U32; executable lowering returns the C word or converts JS's float into
[a Float32Array and Uint32Array](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L553).
Bits are observable language output, not merely internal implementation details.
The pinned [test judge](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L81)
uses exact expected-text equality after its own stream tidying. Our runner is
stricter about transport: separate raw stdout/stderr bytes and process status,
without epsilon, ULP allowance, decimal rounding or whitespace normalization.

For these unary functions, pinned
[numeric lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L225)
calls C's double-precision math function and casts its result to float, or calls
JS Math and then Math.fround. Each nested operation is rounded before its caller
uses the result. Computing the whole exp/log composition in binary64 and rounding
only at the end would implement a different pipeline. The native port follows
this boundary with f64 math followed by an f32 cast; port C/JS use the matching
lowering. No arithmetic implementation needed changing here.

The pinned [Base declarations](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1500)
provide types for these operations rather than definitions or precision proofs;
the [README](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/README.md#L224)
explicitly describes F32 as axiomatic. Combined with the host-library calls,
this supports a limited inference: the inspected source does not supply one
Bend-owned portable transcendental algorithm or a cross-platform bit-accuracy
proof. It does **not** justify a tolerance or a waiver when tested oracles differ.
Any future difference must retain its exact inputs, bits, text and host/build
profile and be investigated as an open cell. No such difference appeared in this
bounded family; other inputs and math-library versions remain unqualified.

## Bit observations and adjacent boundaries

The authored MPL-2.0 `float-math-observations.bend` reports the twelve original
result expressions as `unsigned-bits text` plus LF. It additionally observes
sqrt negative zero, log-zero negative infinity, exponential overflow and the
textual domain-error result. It adds no inventory row.

| Expression | Binary32 bits (unsigned decimal) | F32.show text |
| --- | --- | --- |
| exp(1) | 1076754516 | 2.7182817 |
| log(exp(1)) | 1065353215 | 0.99999994 |
| log2(8) | 1077936128 | 3 |
| log10(1000) | 1077936128 | 3 |
| sqrt(2) | 1068827891 | 1.4142135 |
| sin(1) | 1062693540 | 0.84147096 |
| cos(1) | 1057640768 | 0.5403023 |
| tan(1) | 1070029091 | 1.5574077 |
| sinh(1) | 1066822910 | 1.1752012 |
| cosh(1) | 1069908907 | 1.5430807 |
| tanh(1) | 1061353430 | 0.7615942 |
| tanh(20) | 1065353216 | 1 |
| sqrt(-0) | 2147483648 | -0 |
| log(0) | 4286578688 | -inf |
| exp(100) | 2139095040 | inf |
| sqrt(-1) | Unobserved | nan |

All five runtimes match the sixteen lines twice, exit 0 and empty stderr: **ten
additional exact executions**. The first fifteen pairs compare exact bits and
text independently as observable output, so equal-looking decimal text cannot
hide differing bits. The final line deliberately observes only F32.show: NaN
payload/sign observations are unrun, not passed or assigned a tolerance. Pinned
[C](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L457)
and [JS](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L538)
printers explicitly return nan for that classification. The CLI regression also
preserves raw program bytes under `--output-format json`.

## Validation and receipt identities

The full repository gate passes **812 tests, zero failed, 49 ignored**, exit 0.
Formatting, strict Clippy, all-feature build and the real Node TCP/UDP provider
pass. Ignored checks remain 47 GPU and two optional profiler tests. Focused math,
float-runtime, parse-float and adjacent numeric tests pass **30 tests**, zero
failed/ignored. Strict all-target/all-feature Clippy also passes.

The positive matrix covers fifty fixtures: 44 originals and six authored controls.
It records **488/500 exact primary observations**, with twelve known Bun-FFI host
failures. Thirty separate direct-entry attempts produce four exact observations
and 26 import.meta.require host failures. Runner exit 0 means recorded
classifications held, not that blocked cells passed. All repeats are stable;
none of the 830 child commands times out. The longest child took 1.720 seconds.

The host remains Debian x86_64 with installed Clang 19.1.7, Node 24.19.0 and Rust
1.96.0; no compiler/library installation or update occurred. Upstream C uses
pinned Linux [cli_build](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253)
flags `-std=c11 -O3 SOURCE -lpthread -lm -o EXECUTABLE`, execution
`--threads 1 --gpu off`. Port C keeps strict warnings and existing link flags.
No fast-math flag or relaxed comparison was introduced. Each child retains its
60-second deadline.

Production and shared runner classification logic are unchanged. Negative,
demand, normalization, comptime and status-only audit receipts retain `bd6a228`;
they were not rerun or relabelled. Negative **44/56 exact** retains eight
resource-policy, two native-foreign and two Bun-host gaps. Demand **92/100 exact**
retains six approved C wildcard exceptions and two unwaived Nat-first error-order
differences. Comptime retains **156/160 primary exact**, four default upstream-JS
stack failures and four exact stack-4096 supplements. Eighteen normalization
originals retain exact structure/text agreement. The audit remains 366 accepted
positives, 487 refused, all 439 checker negatives rejected. Older O0/O2, worker,
checking, platform and failure-order evidence keeps its own provenance.

Commands with the existing environment activated and
`CARGO_NET_OFFLINE=true BEND_GPU=off`:

```sh
rustup run nightly-2026-05-24 cargo fmt --all -- --check
cargo test --features extended_observability --test cpu_float_math --test cpu_float_runtime --test cpu_parse_floats --test runtime_numeric --test parser_numeric
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/positive" -Suite all -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

REFERENCE is the clean pinned checkout; EVIDENCE is a fresh private directory.
The focused differential option is `-Suite float-math`. Exact child argv,
fixture/binary hashes, raw streams, statuses, timings and deadline fields are
retained in the private receipts. One-pass preflight used preserved `47a5096`
on checkout `7dd7580`; the exploratory dirty candidate remains separate. Final
qualification snapshots clean `8540287` before the full gate. No other Cargo
build, commit or source edit overlapped that gate. Rebuildable test executables
were cleaned only after their checks completed; source, qualified CLI snapshots
and all evidence remain preserved.

Retained SHA-256 identities:

| Artifact | SHA-256 |
| --- | --- |
| positive receipt | `cac3e12716387268dd612e757e6d58b2f8845e629152a2008833338a5852cc15` |
| preflight receipt | `f36b6a2267076c92daaec8e1dc93dcb351be1d7eb3be110fd19e646fd67fc8a8` |
| candidate receipt | `e6f9ea9d3ca13471b78115e5f3857d7ba46c579627e611a04f0fe5bb0dc1d062` |
| Clean CLI binary | `275170475ae37110a05eff83301e4c0c8ceee31ab2dda7e89b823c2821bcd23e` |
| Unchanged Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |

## Acceptance and next family

The runtime ledger links **66 originals**: 44 positive originals renewed here,
six negatives and sixteen comptime originals retained at `bd6a228`. Eighteen
interpretation originals also retain that commit. Their union is **84**;
**1,218** inventory entries lack these receipts: 439 checker negatives, twelve
check-only modules, 153 interpreter-only cases and 614 compiled candidates.
These are distinct applicable modes, not 1,218 runnable CPU failures. Fifteen
parse/compile direct-entry rows were renewed; older collection direct-entry
observations retain their historical provenance. Authored controls add no rows.

Full CPU parity remains incomplete under the unchanged
[acceptance criterion](cpu-parity-acceptance.md). Host/resource/foreign gaps,
competing-error order, exact checker diagnostics, complete lane/API review and
downstream KernelError/run_main compatibility remain open. No fresh Windows/macOS,
GPU, expanded-worker or other host-library qualification is claimed.

Next source-inspected but **runtime-unqualified** family: `run/float_ops.bend`,
`run/float_printing.bend`, `run/float_text.bend`. These Base/IO compiled candidates
exercise remaining numeric operations/conversions, exact decimal rendering and
F32.read's Maybe results/round-trip. Their goldens are respectively
`1.5 -1.25 1024 0.7853982 1.5707964 0 0.7853982 3 7\n`,
`2 100.25 0.33333334 0.0009765625 123456\n`, and
`2.5 0.1 none none 1.25\n`. Verify generation and actual outputs; stale comments
about six-digit printing or percent-g cannot establish the pinned contract.
Existing local unit witnesses do not replace five-runtime original receipts.
