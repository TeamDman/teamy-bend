# Float runtime and storage qualification

Coverage source: `47a50962dc77cf85cd829a65162638376ffa1a80`, local branch
`qualify/cpu-float-runtime-20261023`, based on `1871874`.
Target: [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
All three originals and a focused observation control agree in the five primary
runtimes. No production defect was demonstrated. Changes add attributed fixtures,
CLI regression coverage and a comparison-suite entry; production code, frozen
dependencies/toolchains, safety limits and approved demand semantics are unchanged.
No installation or push.

## Applicable lanes and original outputs

All three import Base and have IO mains. Pinned
[gate selection](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L48)
selects compiled candidates. Actual C and JS generation succeeds for each;
[show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
selects the IO driver. Their comments about stuck pure F32 axioms do not remove
these executable lanes. No pure-F32 normalization or function-printer claim is
inferred from IO success.

| Unchanged upstream original | Behavior exercised | Exact stdout |
| --- | --- | --- |
| [compile/float_builtin_agreement.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/compile/float_builtin_agreement.bend) | Floor, ceil, trunc and abs applied to a negative finite value, then F32.show | `-2 -1 -1 1.5\n` |
| [compile/float_accumulation.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/compile/float_accumulation.bend) | Nat-controlled accumulation using conversions, division, multiplication and addition, ending in a U32 digest | `2775\n` |
| [compile/float_unboxed_ctor.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/compile/float_unboxed_ctor.bend) | A one-F32-field constructor and two array swaps, ending in a U32 digest | `375\n` |

Native Rust, port C, port JS, upstream C and emitted upstream JS match stdout
byte-for-byte, with empty stderr and exit 0, twice: **30 exact observations**.
No original was shortened or changed; Apache-2.0 attribution is retained.

All three direct upstream IO entries exit 1, stdout empty, stderr exactly
`TypeError: require is not a function\n`, twice. The pinned
[io_run](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1271)
passes Bun's import.meta.require, absent under Node. The six direct observations
remain host-blocked even though emitted CommonJS passes. No source adaptation,
Bun installation or whole-fixture exact-parity claim hides those failures.

## Bits, printed text and nonfinite scope

The original accumulation/storage results are integer digests; they do not
observe every intermediate F32 bit. The authored MPL-2.0 control
`float-runtime-observations.bend` adds paired binary32-bit and text observations
for the relevant operations, including an inexact ten-step sum. It adds no
upstream inventory row. Pinned
[numeric lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L220)
rounds arithmetic results to binary32; unary rounding operations use C double
math followed by a float cast, or JS Math followed by Math.fround.

| Observation | Binary32 bits, as unsigned decimal | F32.show text |
| --- | --- | --- |
| floor(-1.5) | 3221225472 | -2 |
| ceil(-1.5), trunc(-1.5) | 3212836864 | -1 |
| abs(-1.5) | 1069547520 | 1.5 |
| floor(-0), ceil(-0.25), trunc(-0.75) | 2147483648 | -0 |
| abs(-0) | 0 | 0 |
| Ten successive binary32 additions of 0.1 to zero | 1065353217 | 1.0000001 |
| Negative zero through a FloatCell field or array swap | 2147483648 | -0 |
| 1.25 through an array swap | 1067450368 | 1.25 |
| floor(positive infinity), ceil(negative infinity) | 2139095040; 4286578688 | inf; -inf |
| trunc(NaN produced by zero/zero) | Not compared | nan |

The control prints each numeric bit/text pair separated by one space and LF,
then the NaN text alone. Every runtime matches all fifteen lines twice, exit 0
and empty stderr: **ten additional exact executions**. The CLI regression also
preserves raw program bytes with `--output-format json` selected.

The originals use finite values; infinity and NaN are adjacent inputs to their
rounding intrinsics, not behavior attributed to those original workloads.
[Pinned C text rendering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L457)
and [JS text rendering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L538)
spell NaN as nan and preserve negative-zero text. No exact arithmetic-NaN
payload/sign contract is asserted; that would be stronger than these observations.
This does not qualify all float math, source literals, F32.read or pure term
printing. Existing numeric tests remain supporting evidence for their own cases.

The first authored control incorrectly destructured a computed Array.swap
expression. Both upstream and port refused it before execution: a match must
scrutinize a parameter or field. Moving the destructuring into a parameter helper
corrected the control without changing its intended operations. The initial
candidate receipt and failed focused run remain retained; they are not port
semantic defects. The unchanged original fixtures already passed before this
control correction.

## Validation and evidence identity

The standard repository gate passes **811 tests, zero failed, 49 ignored**,
exit 0. Formatting, strict Clippy, all-feature build and the real Node TCP/UDP
provider pass. The ignored checks remain 47 GPU and two optional profiler tests.
Focused float/adjacent numeric tests pass **29 tests, zero failed/ignored**;
strict all-target/all-feature Clippy also passes.

The repeated positive matrix covers 46 fixtures: 41 upstream originals and five
authored controls. It records **448/460 exact primary observations** and twelve
known Bun-FFI failures. Twenty-four separate direct-entry attempts produce four
exact observations and twenty import.meta.require host failures. Runner exit 0
means the classifications held, not that the blocked cells passed. All repetitions
are stable, and none of the 760 child commands times out. The longest child took
1.718 seconds.

The comparison uses the pinned Linux
[cli_build profile](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253):
`-std=c11 -O3 SOURCE -lpthread -lm -o EXECUTABLE`, followed by
`--threads 1 --gpu off`. Port C retains `-Wall -Wextra -Werror -pedantic` and its
existing link flags. Every child has a 60-second deadline. No profile or resource
budget was raised.

Production code and shared runner classification logic are unchanged. Negative,
demand, normalization, comptime and strict-audit receipts retain their actual
`bd6a228` identities. They were not rerun or relabelled: negative **44/56 exact**,
with eight resource-policy, two native-foreign and two Bun-host gaps; demand
**92/100 exact**, six approved C wildcard exceptions and two unwaived Nat-first
error-order differences; comptime **156/160 primary exact**, four default upstream
JS stack failures plus four exact stack-4096 supplements. The eighteen original
normalization cases retain exact structure/text agreement. The latest status-only
strict audit still records 366 accepted positives, 487 refused and all 439 checker
negatives rejected. Historical O0/O2, worker, checking, platform and failure-order
receipts stay separate. No current pass is manufactured for a retained open cell.

Commands with the existing activation sourced and
`CARGO_NET_OFFLINE=true BEND_GPU=off`:

```sh
rustup run nightly-2026-05-24 cargo fmt --all -- --check
cargo test --features extended_observability --test cpu_float_runtime --test cpu_parse_floats --test runtime_numeric --test parser_numeric
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/positive" -Suite all -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

REFERENCE names the clean pinned checkout; EVIDENCE is a fresh private directory.
The focused differential option is `-Suite float-runtime`. Private receipts
retain exact child argv, source/binary hashes, separate raw streams, status,
timing and timeout fields. The one-pass preflight used the preserved clean
`bd6a228` binary on checkout `1871874`; exploratory dirty candidates are labelled
separately. The final matrix snapshots the clean `47a5096` binary before the full
gate. No other Cargo build or commit overlapped that gate, and no source changed
during it. Rebuildable package executables were removed before building and
after completed tests to recover disk space. The qualified CLI, source and all
prior evidence were preserved; no active test was interrupted.

Retained SHA-256 identities:

| Artifact | SHA-256 |
| --- | --- |
| positive receipt | `dec04a920a473323824588e1bacd88a1d7e06a1df20c69f959fc2c0fa898540d` |
| preflight receipt | `51ad1e3c4aba50b2c354ba3732e492fcc9b41f48fe8351a3a5674edeb4d7bfec` |
| candidate receipt | `234005c4c10177e15df8adba5d512159d87fb214b8bb17e5c77bb99e9727ee57` |
| candidate-fixed receipt | `457e1e12fd48e36fe53d92ef98f854edfb8ba777a6c2f013582a87a9933f1842` |
| Clean CLI binary | `f743d7798264812de1495ecd006dfda5a7055321fd303ffe94bcf3edf2d3c6e8` |
| Unchanged Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |

## Acceptance and next independent family

The runtime ledger now links **63 upstream originals**: 41 positive originals
renewed here, six negatives and sixteen comptime originals retained at `bd6a228`.
Eighteen interpretation originals also retain that commit. The union is **81**;
**1,221** inventory entries lack runtime/interpretation receipts: 439 checker
negatives, twelve check-only modules, 153 interpreter-only cases and 617 compiled
candidates. These categories require different modes; they are not a count of
runtime defects. Twelve parse/compile direct-entry rows were renewed; older
collection direct-entry observations keep their own provenance.

Full parity remains incomplete under the unchanged
[acceptance criterion](cpu-parity-acceptance.md). Host/resource/foreign gaps,
competing-error order, exact checker diagnostics, complete lane/API review and
downstream KernelError/run_main compatibility remain open. No fresh Windows/macOS,
GPU or expanded-worker qualification is claimed. No environment blocker prevents
continuing bounded CPU verification.

Next source-inspected but **runtime-unqualified** family:
`compile/float_unary_math.bend` (`2.7182817 0.99999994 3 3 1.4142135\n`),
`compile/float_trig.bend` (`0.84147096 0.5403023 1.5574077\n`) and
`compile/float_tanh.bend` (`1.1752012 1.5430807 0.7615942 1\n`). They are Base/IO
compiled candidates exercising libm/Math operations and decimal formatting.
Establish actual target generation and cross-runtime outputs; do not assume
platform-independent bit equality from host math or from existing local unit
witnesses. Differences in bits, text or oracle behavior must remain distinct.
