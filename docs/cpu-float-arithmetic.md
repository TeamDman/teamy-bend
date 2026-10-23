# Finite float arithmetic and conversion qualification

Coverage source: `8ad7f43936841889331eb711b2818bc4d9defdf3`, local branch
`qualify/cpu-float-arithmetic-20261023`, based on `e99da79`.
Target: [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The three originals and authored bit/text control agree across all five primary
runtimes. No production defect was demonstrated. The separate runtime F32.read
probe retains three upstream C/JS disagreements and exit 1; corresponding-backend
matches do not establish cross-backend agreement. No parsing policy, tolerance,
new exception, dependency, toolchain, budget or approved-demand change. No push.

## Original applicability and outputs

| Unchanged original | Exact stdout |
| --- | --- |
| [base/float_roundtrip.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/base/float_roundtrip.bend) | `144\n` |
| [compile/float_roundtrip.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/compile/float_roundtrip.bend) | `45\n` |
| [run/float_arithmetic.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/float_arithmetic.bend) | `8\n` |

All three import Base and have IO mains. Actual upstream C and JS generation
succeeds. Native Rust, port C, port JS, upstream C and emitted upstream JS match
these bytes, empty stderr and exit 0 twice: **30 exact primary executions**.
The two colliding float_roundtrip filenames are disambiguated in local copies;
their source bytes, comments and Apache-2.0 attribution remain unchanged.

The first program checks annotated operator chains and precedence: 4 + 14 * 10.
The second converts exact U32 values to floats, divides 15 by 2, multiplies by 6
and converts back. The third sums eight Boolean checks covering arithmetic,
binary32 precision at 2^24 and finite casts. Its digest does not expose every
intermediate value, so the additional control records selected bits and text.

All three direct upstream IO entries fail twice with exit 1, empty stdout and
`TypeError: require is not a function\n`. The pinned
[direct IO driver](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1271)
requires import.meta.require, unavailable on this Node host. Separately emitted
CommonJS works. These six direct observations remain blocked, not passing.
Strict proof checking is another lane: the pinned [README](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/README.md#L224)
describes F32 as axiomatic, and runtime comparisons do not establish float proofs.

## Exact bit and conversion observations

The authored `float-arithmetic-observations.bend` adds ten exact executions:
five runtimes, twice each, thirteen stdout lines, exit 0 and empty stderr.
Each row prints unsigned binary32 bits, a space, text and LF. Positive
truncation prints the separate integer line `3\n` immediately after the 3.9 row.

| Expression or observation | Bits | Text |
| --- | --- | --- |
| 6.25 - 2.25 | 1082130432 | 4 |
| 3.5 * 2 / 0.5 | 1096810496 | 14 |
| 15 / 2 | 1089470464 | 7.5 |
| 6 * (15 / 2) | 1110704128 | 45 |
| (1 / 10) + (2 / 10) | 1050253722 | 0.3 |
| 3 / 10 | 1050253722 | 0.3 |
| 16777216 + 1 | 1266679808 | 16777216 |
| U32.to_f32(16777217) | 1266679808 | 16777216 |
| U32.to_f32(16777219) | 1266679810 | 16777220 |
| 39 / 10 | 1081711002 | 3.9 |
| U32.to_f32(123456) | 1206984704 | 123456 |
| ((1 / 10) + (2 / 10)) * 1000000 | 1217559552 | 300000 |

Integer operands in the control pass through U32.to_f32 and F32.div. The selected
adjacent integer ties round to even in both directions. The positive 3.9 cast
truncates to 3. [Pinned lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L206)
uses C float operations/rewrapping and JS Math.fround; the
[bounded conversion](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L427)
handles F32.to_u32. These exact observations qualify the tested profile and
values, not all inputs, host libraries, NaN payloads or GPU execution.
The CLI regression preserves the same raw program output with `--output-format json`.

## What the pin does and does not specify about F32.read

The [guide](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/guide/GUIDE.md#L448)
says numeric show produces String and read returns Maybe. The
[Base declaration](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1606)
provides the String-to-Maybe<F32> type, without a lexical grammar or rounding law.
The [runtime paper](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/docs/BendRT/main.typ#L493)
states consistent float semantics and byte-comparing harness checks in its
CPU/GPU discussion. This supports consistency intent; it does not define decimal
text conversion or establish agreement for every JS input.

The [float_text regression comment](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/float_text.bend#L1)
is more specific: it describes strtof grammar behind a whole-string gate.
That suggests C-style lexical acceptance and complete consumption, but its
adjacent percent-g rendering description is stale and its actual examples do
not test the three boundary inputs. Neither it nor the type declaration fully
resolves the implementation conflicts. In particular, the stated whole-string
intent conflicts with C's embedded-NUL prefix acceptance.

Runtime [C read](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L492)
uses strtof and checks `n > 0 && *end == 0`, without checking the end pointer
against the original byte length. Runtime
[JS read](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L561)
uses a decimal/inf/nan regex, Number, then Math.fround. The native implementation
already follows C-locale parsing, while generated port C/JS follow their
respective upstream implementations. This is a demonstrated upstream
cross-backend difference, not a demonstrated port-to-corresponding-backend defect.

The renewed [read ledger](cpu-float-read-evidence.csv) preserves exact outcomes:

| Bend string input | Native Rust / port C / upstream C | Port JS / upstream JS |
| --- | --- | --- |
| `"1.0000000596046448"` | Some, bits 1065353217 | Some, bits 1065353216 |
| `"0x1.8p+1"` | Some, bits 1077936128 (3.0) | None |
| `"1.5\0tail"` | Some, bits 1069547520 (1.5) | None |
| `"1.5\n"` | None | None |

All ten executions return exit 0 with empty stderr and stable output. None is a
value, not a process error. The runner still returns **1** for the three contract
splits. Final-newline rejection remains a shared result, not a fourth split.

For the decimal case, exact rational arithmetic explains the difference. The
midpoint between binary32 1 and its next larger value is
`1.000000059604644775390625` (1 + 2^-24). The input lies above it by
`0.000000000000000024609375`, less than half a binary64 ULP there. Direct
binary32 conversion rounds upward; Number first rounds to the exact midpoint,
then Math.fround chooses the even lower value. This is evidence for the selected
input, not a host-wide decimal accuracy claim.

[Source float literals](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2207)
are a different contract: the shared frontend explicitly uses Number then
Math.fround and rejects nonfinite literals. The earlier
[literal correction](cpu-parse-floats.md) follows that code for all targets.
It does not authorize changing runtime F32.read to JS rules. Choosing one common
grammar/rounding/full-consumption policy, or accepting backend-dependent
exceptions, remains a separate decision. None is implemented or presumed here.

## Validation and provenance

The clean coverage commit passes the standard Linux gate: **814 passed, zero
failed, 49 ignored**, exit 0. The 47 GPU and two optional-profiler ignores remain
unrun. Focused checks pass **32/32**, strict all-target/all-feature Clippy passes,
and the clean all-feature build passes. The broad positive O3 matrix covers
58 fixtures: fifty unchanged originals and eight authored controls, 970 child
commands. **568/580 primary executions match exactly**; twelve retain existing
Bun-FFI host failures. Forty-two separate direct observations include four exact
and 38 import.meta.require failures. Its runner exits 0 with explicit host
classifications; blocked cells are not passes. All repeats are stable, with zero
timeouts and a longest child of 2.134 seconds.

The renewed read probe has 16 child commands and ten successful, stable runtime
executions, no timeouts and a longest child of 0.721 seconds. All three
port-to-backend comparisons match, but the three conflicting input rows retain
`unresolved_upstream_read_contract` and runner **exit 1**. This probe remains
separate from the positive matrix and adds no shared-contract pass.

Upstream C uses the pinned Linux [cli_build profile](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253):
`-std=c11 -O3 SOURCE -lpthread -lm -o EXECUTABLE`, with
`--threads 1 --gpu off`. Port C retains strict warnings. Rust 1.96.0,
nightly-2026-05-24 formatting, Clang 19.1.7 and Node 24.19.0 remain frozen.
Every child has a 60-second deadline; receipts preserve raw streams, exact argv,
status, hashes, timing and repetition comparisons.

Commands after activating the existing environment and setting
`CARGO_NET_OFFLINE=true BEND_GPU=off`:

```sh
cargo test --features extended_observability --test cpu_float_arithmetic --test cpu_float_text --test cpu_float_math --test cpu_float_runtime --test cpu_parse_floats --test runtime_numeric --test parser_numeric
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/positive" -Suite all -Repetitions 2 -OracleOptimization O3
python3 scripts/compare-float-read.py --reference "$REFERENCE" --output "$EVIDENCE/read"
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

REFERENCE names the clean pin; EVIDENCE is a fresh private directory. The focused
comparison option is `-Suite float-arithmetic`. Preflight used the preserved
all-feature `2dd113e` binary on checkout `e99da79`. The exploratory dirty candidate
used its saved gate-feature binary. Final receipts use clean `8ad7f43`; no other
Cargo build, source edit or commit overlaps the gate. Completed root test
executables were reclaimed with manifests; byte-identical immutable snapshots
were deduplicated while preserving every evidence path and hash. No source,
raw log or historical receipt was discarded.

Private evidence SHA256 hashes:

| Evidence | SHA256 |
| --- | --- |
| preflight receipt | `f84c96f80a8fad02b066ad6f752e20e15755bc87c95d78fb690dd6aff587ea00` |
| candidate receipt | `4d42db29b59f707daa989eb36b818356f27419261b7b6fce70d46146515013e4` |
| positive receipt | `6355111f1825508364bc0a6fbcb7a6fc258196cb4b9faa0ef179a9e90f6d784f` |
| read receipt | `d0badfbe70da91c73cf560cda0261fd08c169cb3e07cc6d9d95befb03349f03a` |
| Clean all-feature binary | `170b41fa9b051e84d9ad88b31f10fc2e7fa1cafe7cfba4bfa56253883f14b50c` |
| Unchanged Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |

## Remaining acceptance and next independent family

The runtime ledger now links **72 originals**: fifty positive originals renewed
here, six negatives and sixteen comptime originals retained at `bd6a228`.
Eighteen interpretation originals retain that commit. The union is **90**;
**1,212** inventory entries lack these receipts: 439 checker negatives, twelve
check-only modules, 153 interpreter-only cases and 608 compiled candidates.
These are different required modes, not a count of runnable CPU defects. Eight
positive authored controls and the separate read probe add no inventory rows.
Twenty-one parse/compile/base/run-float direct entries are renewed; earlier
collection direct observations retain their original provenance.

Production and comparison classification logic are unchanged. Unaffected
negative/demand/normalization/comptime/audit matrices were not rerun or relabelled:
negative 44/56 exact retains eight resource-policy, two native-foreign and two
Bun-host gaps; demand 92/100 exact retains six approved wildcard exceptions and
two unwaived Nat-first error-order differences; comptime 156/160 primary exact
retains four default-JS stack failures plus four exact stack-4096 supplements.
Normalization retains eighteen originals with exact structure/text agreement.
The strict audit remains 366 accepted positives, 487 refused, all 439 checker
negatives rejected. Historical profiles, checking diagnostics and downstream
KernelError/run_main API caveats remain intact.

Full CPU parity remains incomplete under the unchanged
[acceptance criterion](cpu-parity-acceptance.md). The three read contracts,
competing-error order, host/resource/foreign cells and complete lane/API review
remain open. No GPU or new dependency is needed for the next independent family:
`run/float_text_roundtrip` (`233\n`), `compile/float_compare` (`1999985\n`),
`run/float_specials` (`12\n`). Their sources and applicability hints were read;
actual original-fixture generation/runtime qualification is **unrun**. Despite
its name, float_text_roundtrip uses arithmetic and to_u32, not F32.read. The
specials fixture observes comparisons/conversions, not NaN payload bits. Fresh
Windows/macOS, additional worker and GPU profiles remain unrun.
