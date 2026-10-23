# Float operations, printing and runtime text parsing

Coverage source: `2dd113e1e0e044242bdf6b70fb0b2101758c3d3b`, local branch
`qualify/cpu-float-text-20261023`, based on `fcf74a7`.
Target: [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The three originals and shared observation control agree across five primary
runtimes. A separate boundary probe demonstrates three pinned C/JS F32.read
contract splits; each port matches its corresponding backend, with native Rust
matching C. Those differences remain unresolved, not accepted exceptions.
No production defect was demonstrated or parsing policy changed. Dependencies,
toolchains, pin, safety budgets and approved demand rules are frozen. No install
or push.

## Original applicability and exact outputs

All three import Base and have IO mains. Both upstream targets actually generate
successfully, confirming their compiled-candidate applicability under pinned
[gate selection](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L48)
and the [IO result path](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204).
Pure proof-mode numeric reduction is a different lane and is not inferred here.
Original sources, comments and Apache-2.0 attribution remain unchanged.

| Exact upstream fixture | Exact stdout bytes |
| --- | --- |
| [run/float_ops.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/float_ops.bend) | `1.5 -1.25 1024 0.7853982 1.5707964 0 0.7853982 3 7\n` |
| [run/float_printing.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/float_printing.bend) | `2 100.25 0.33333334 0.0009765625 123456\n` |
| [run/float_text.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/float_text.bend) | `2.5 0.1 none none 1.25\n` |

Native Rust, port C, port JS, upstream C and emitted upstream JS each reproduce
these outputs twice, exit 0 and empty stderr: **30 exact primary executions**.
All three direct upstream IO entries remain host-blocked twice: exit 1, empty
stdout and `TypeError: require is not a function\n`. The pinned
[IO driver](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1271)
requires Bun's import.meta.require, absent under the installed Node host. Emitted
CommonJS succeeds. No upstream adaptation, installation or whole-fixture parity
claim hides the six direct failures.

The comments about six significant digits, percent-g and a printing tie split
are stale. Actual pinned [C text rendering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L457)
tries successive scientific precisions until strtof recovers the same binary32
value, then selects fixed/exponent spelling. [JS rendering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L538)
uses successive toExponential precisions and Math.fround(Number(text)). Actual
outputs retain `0.33333334` and `0.0009765625`; no six-digit truncation is applied.
This qualifies the tested values and host, not every decimal or host library.

## Shared bit/text and invalid-read controls

The authored `float-text-observations.bend` adds ten exact executions, five
runtimes twice, with eighteen output lines. It pairs bits and text for the
operation/conversion results below; separately prints U32 conversion `3`, read
bits and the exact None results. All statuses are 0 and stderr is empty.
Raw CLI program output also remains unchanged under `--output-format json`.

| Observation | Unsigned binary32 bits | Text |
| --- | --- | --- |
| mod(7.5, 2) | 1069547520 | 1.5 |
| neg(1.25) | 3214934016 | -1.25 |
| pow(2, 10) | 1149239296 | 1024 |
| atan2(1, 1), atan(1) | 1061752795 | 0.7853982 |
| asin(1) | 1070141403 | 1.5707964 |
| acos(1) | 0 | 0 |
| U32.to_f32(7) | 1088421888 | 7 |
| div(1, 3) | 1051372203 | 0.33333334 |
| literal 0.0009765625 | 981467136 | 0.0009765625 |

Read inputs `"0.1"`, `"-0"` and `".5"` return Some with bits 1036831949,
2147483648 and 1056964608 respectively. `"1.5x"`, empty text and `"1.5 "` all
return None. Reading F32.show(1.25) returns bits 1067450368. None is an ordinary
Maybe result: these cases do **not** fail the process or emit a diagnostic.
Text and bits are compared exactly, without tolerance. NaN payload behavior is
not exercised or qualified by this family.

## Three unresolved F32.read contracts

The separate authored `float-read-boundaries.bend` and
[read evidence ledger](cpu-float-read-evidence.csv) retain four inputs and exact
outputs across two repetitions. Every execution completes with exit 0, empty
stderr and stable output. Three inputs differ between pinned C and JS; the
final-newline rejection agrees.

| Input as a Bend string literal | Native Rust / port C / upstream C | Port JS / upstream JS | Classification |
| --- | --- | --- | --- |
| `"1.0000000596046448"` | Some, bits **1065353217** | Some, bits **1065353216** | Unresolved rounding difference |
| `"0x1.8p+1"` | Some, bits **1077936128** (3.0) | None | Unresolved hexadecimal grammar difference |
| `"1.5\0tail"` | Some, bits **1069547520** (1.5) | None | Unresolved embedded-NUL difference |
| `"1.5\n"` | None | None | Exact shared rejection result |

The probe prints labels `decimal`, `hex`, `newline`, `nul`, in that order,
followed by a space and the decimal bits or `none`, each terminated by LF. Thus
its C/native stdout is exactly:

```text
decimal 1065353217
hex 1077936128
newline none
nul 1069547520
```

Its JS stdout is exactly:

```text
decimal 1065353216
hex none
newline none
nul none
```

One small reproducer for any row is:

```bend
import Base

def bits(result: Maybe<&2, F32>) -> String:
  match result:
    case None{}: "none"
    case Some{value}: U32.show(F32.bits(value))

def main() -> IO(Unit):
  IO.print(bits(F32.read("1.0000000596046448")))
```

Pinned [C F32.read](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L492)
uses strtof and accepts when the original byte length is nonzero and the end
pointer references NUL. It does not compare that pointer with the original
string's final byte; the embedded-NUL prefix therefore succeeds on this profile.
Hexadecimal parsing and direct binary32 conversion are also observed here.
Pinned [JS F32.read](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L561)
uses its decimal/inf/nan regex and Number followed by Math.fround. That excludes
these hex/NUL spellings and rounds the particular decimal through binary64.
The native parser already documents its C-locale contract; generated port C/JS
follow their corresponding pinned implementations. No differing port output
relative to those backend oracles was demonstrated.

This is distinct from [source float literal parsing](cpu-parse-floats.md), where
all pinned backends share the Number-then-fround frontend. Applying that earlier
literal correction to runtime F32.read would change existing native/C behavior.
There is no single output matching both upstream read implementations for these
three inputs. A uniform C-style rule would change JS grammar/rounding; a uniform
JS-style rule would change native/C accepted inputs and values. Keeping separate
backend contracts as a parity exception would also require an explicit decision.
None of those choices is made here; the three acceptance cells remain open.
Independent finite arithmetic qualification can continue meanwhile.

`scripts/compare-float-read.py` keeps this distinction executable. It snapshots
source/binary identity, generates and compiles with the same pinned O3 profile,
runs all five runtimes twice with 60-second deadlines, and compares raw status/
streams. It records each port-to-backend match separately from upstream C/JS
agreement. Its **exit remains 1** for the unresolved split even when all three
backend matches are true. Generation, compilation, runtime, timeout or repetition
failures have separate outcomes. The probe is excluded from the positive suite's
single shared expectation and recorded here instead; exclusion is not a waiver.

## Validation and provenance

The clean coverage commit passes the standard Linux gate: **813 passed, zero
failed, 49 ignored**, exit 0. The 47 GPU and two optional-profiler ignores remain
unrun. Focused checks pass **31/31**, and strict all-target/all-feature Clippy,
pinned formatting and the clean all-feature build pass. The broad positive O3
matrix covers 54 fixtures: 47 unchanged originals and seven authored controls,
900 commands total. Of 540 primary executions, **528 match exactly** and twelve
retain existing Bun-FFI host failures. Thirty-six separate direct observations
include four exact and 32 import.meta.require failures. Its runner exits 0 with
those explicit environmental classifications; blocked cells are not passes.
All repeats are stable, with zero timeouts and a longest child of 1.701 seconds.

The separate read probe captures 16 commands and ten successful, stable runtime
executions. All three port-to-backend comparisons match, but the three differing
input rows keep outcome `unresolved_upstream_read_contract` and runner **exit 1**.
No child timed out; the longest took 0.724 seconds (rounded upward). This failure
is not included in the positive matrix's exact count. It remains a required open
contract, independently of the passing repository gate.

Upstream C uses pinned Linux [cli_build](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253)
flags `-std=c11 -O3 SOURCE -lpthread -lm -o EXECUTABLE` and executes with
`--threads 1 --gpu off`. Port C retains strict warnings and existing link flags.
Clang 19.1.7, Node 24.19.0, Rust 1.96.0 and the installed Linux host remain frozen.
No generalization to other C libraries, JS hosts or untested inputs is claimed.

Production and shared positive-runner classification logic are unchanged.
Negative, demand, normalization, comptime and status-only audit receipts retain
`bd6a228`, without rerunning or relabelling them. Negative **44/56 exact** keeps
eight resource-policy, two native-foreign and two Bun-host gaps. Demand **92/100
exact** keeps six approved C wildcard exceptions and two unwaived Nat-first
order differences. Comptime retains **156/160 primary exact**, four default JS
stack failures plus four exact stack-4096 supplements. Eighteen normalization
originals retain exact structure/text agreement. The audit remains 366 accepted
positives, 487 refused and all 439 checker negatives rejected. Historical O0/O2,
worker, checking, platform and failure-order identities remain intact.

Commands with the existing environment activated and
`CARGO_NET_OFFLINE=true BEND_GPU=off`:

```sh
rustup run nightly-2026-05-24 cargo fmt --all -- --check
cargo test --features extended_observability --test cpu_float_text --test cpu_float_math --test cpu_float_runtime --test cpu_parse_floats --test runtime_numeric --test parser_numeric
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/positive" -Suite all -Repetitions 2 -OracleOptimization O3
python3 scripts/compare-float-read.py --reference "$REFERENCE" --output "$EVIDENCE/read"
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

REFERENCE names the clean pin; EVIDENCE is a fresh private directory. The focused
positive option is `-Suite float-text`. Exact child argv, raw streams, statuses,
hashes and timings remain in receipts. One-pass preflight used saved `8540287`
on checkout `fcf74a7`; exploratory dirty candidates are separate. Final positive
and read receipts snapshot clean `2dd113e` before the full gate. No other Cargo
build, commit or source edit overlaps that gate. Completed test products are
rebuildable; immutable byte-identical recent CLI snapshots were deduplicated
with every path and hash verified. All source, raw logs and evidence remain.

Private evidence hashes (SHA256):

| Evidence | SHA256 |
| --- | --- |
| preflight receipt | `28d82a34b82a7c99311cd72c168021104d3c98caa99d2144b30a0792e769383c` |
| candidate receipt | `ec3b0bb95e625e8fdfafd9add238327fb717ccdba4fb7a0f8a8b28b419b90844` |
| read-candidate receipt | `e15768a23c13cd5db897a8a4d0e44db7e450ff97f4404b2793673f961297f0b3` |
| positive receipt | `9bd4eff90eeef2da106c2c77166eaf6c0eeff1a96730cdb041e691ee7de543a1` |
| read receipt | `e6271da112c3806190ee3516767708a3ebfdf3cddb3858181208d0de015b9a60` |
| Clean all-feature binary snapshot | `504307038ecc3ad55d98fca577791ce84e89c8752f314d1a2f56102baf4b0d91` |
| Unchanged Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |

## Acceptance and next independent family

The runtime ledger now links **69 originals**: 47 positive originals renewed
here, six negatives and sixteen comptime originals retained at `bd6a228`.
Eighteen interpretation originals retain that commit. The union is **87**;
**1,215** inventory entries lack these receipts: 439 checker negatives, twelve
check-only modules, 153 interpreter-only cases and 611 compiled candidates.
These are distinct required modes, not a count of runnable CPU defects. Seven
positive authored controls and the separate four-input read probe add no upstream
inventory rows. Eighteen parse/compile/run-float direct entries were renewed;
older collection direct-entry observations retain their actual provenance.

Full CPU parity remains incomplete under the unchanged
[acceptance criterion](cpu-parity-acceptance.md). The three new read contracts,
existing competing-error order, host/resource/foreign cells, exact checker
errors, complete lane/API review and downstream KernelError/run_main compatibility
remain open. No new exception is accepted. Fresh Windows/macOS, GPU, additional
workers and other host-library profiles are unrun.

Next source-inspected but **runtime-unqualified** family:
`base/float_roundtrip.bend` (`144\n`), `compile/float_roundtrip.bend` (`45\n`),
`run/float_arithmetic.bend` (`8\n`). These Base/IO candidates cover finite
arithmetic/conversions and do not depend on resolving the runtime-read split.
Verify actual generated-target applicability and outputs; local unit witnesses
or source comments are not a substitute for original-fixture comparisons.
