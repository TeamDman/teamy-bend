# Ragged rows: concrete applications and runtime observations

Exact tested source: **`623c6bacfc82993e731c4c124306924c1d68d8eb`**. Pinned upstream:
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
No ragged-row behavior mismatch was found in this bounded grid. Full CPU parity
remains incomplete.

## What is qualified

The [five original fixtures](../tests/fixtures/cpu-ragged-rows/README.md) are
byte-identical to the pin and retain Apache attribution. The actual pinned
[flattener](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2679)
and original source rows establish the relevant first-match/default behavior.

| Original | Vectors | Behavior covered |
| --- | ---: | --- |
| `_000` | 9 | Nested Nat patterns, all six result constructors and their fields |
| `_001` | 9 | Leading default dominance, remaining rigid branches |
| `_002` | 27 | Overlap across three columns and trailing default |
| `_003` | 27 | W fuel, W selector and Nat depth varied independently; all three outcomes |
| `_004` | 9 | Earlier default row wins when its rigid column matches |

Inputs use constructor depths 0..2. All source rows and result constructors are
reached; the finite grid does not prove arbitrary-depth equivalence. Two pinned
interpreter runs directly apply arguments to the unchanged original definitions.
Their complete constructor results agree with an independently written branch
model. The saved TSV drives `CheckedBook::evaluate` regressions comparing both
exact values and core structure. Original function normal forms also compare
exact text and actual strict normalized core. CLI `run` text is captured separately;
no separate executable-core probe is claimed for that writer.

Original mains return functions and lack Base. Actual pinned C/JS requests refuse
them. Base-importing function wrappers reach the function-printer refusal. Neither
case provides executable qualification for an original main.

The authored pure wrappers import unchanged originals and project each result
to a list containing its one-based constructor ID followed by **every returned
field's depth**. These are complete observations for this bounded input family,
not checksums. Their exact output matches the pinned interpreter, port pure CLI,
upstream C/JS and generated port C/JS. Strict evaluation also matches. Pure
`ExecutableBook::run_main` uses `Engine::normalize_definition`, so this evidence
is explicitly normalization rather than native effect-driver execution.

The separate IO adapters call the same wrappers and print the observations with
`List.show`/`IO.print`. Tests assert `ExecutableEntry::Io` and execute the native
runtime. Native IO and all four generated C/JS lanes match exactly, twice. The
unadapted pinned Node CLI IO lane has no `import.meta.require`; the actual
result is exit 1, empty stdout and `TypeError: require is not a function\n` on
stderr. Those ten calls remain host-blocked. No Bun installation or adapted oracle is substituted. Strict
proof checking rejects these IO sources because IO is outside that mode; the five
refusals are proof exclusions, not runtime passes.

## Scoped printer-refusal correction

The pinned [show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
reports the main type and a common unprintable-result explanation. It checks the
IO-name/Base prerequisite before that diagnostic. The port previously exposed
backend-specific host error context. Known refusal branches now attach owned
diagnostic text, using the main type and IO-declaration presence. Nested function
and erased fields report the enclosing main type. These checks still reject at
the same places; no successful compilation now requires a new prerequisite.

All internal `CompileError` Display/equality messages remain unchanged. No public
signature changes; private diagnostic representation and derived Debug content
change. The existing CLI boundary writes/flushes stderr and returns status-bearing
`Ok(CliOutput)` with exit code 1. Repository dispatch propagates that value and
the entry point checks it. External callers remain unaudited and must inspect
`exit_code()`. Internal diagnostics remain available to tracing. Unclassified
compiler/budget errors retain their host boundary.

Regressions cover original refusals, function and Type values, erased/function
fields, writer/flush errors, cancellation and retained output files under `--force`.
No checker, evaluator, term-sharing or demand-policy implementation changed.
Existing structural, source-hole, work/depth, cancellation and first-error controls
pass in the full gate; no limit was raised or warning suppressed.

## Results and provenance

| Check | Result |
| --- | --- |
| Full gate at tested source | 933 passed, 0 failed, 49 ignored; 100 targets |
| Format / strict Clippy / all-feature build / TCP-UDP provider | Passed |
| Pure differential matrix | 5 originals, 81 vectors, 140 process observations; twice |
| IO differential matrix | 100 observations; 50 exact runtime successes, 10 direct-oracle host blocks, 5 proof exclusions, 5 checking probes, 30 generation/build successes |
| Printer matrix | 14 cases twice: 112 exact comparisons + 28 oracle captures |
| Printer phase probes | 56 accepted-source probes; emitter refusal is a later phase |
| Checking-status audit | All 1,302; 439 expected negatives rejected, others 366 accepted / 497 rejected; zero transitions |
| Positive / hole controls | 14 pass; 18 observations retain three unresolved disagreements |

The 49 ignores remain unrun: 47 hardware tests and two optional profilers. The
earlier `4022f8f81b6ecb47a3e80f7ed7b9c1527e90b31e` gate passed 932 tests before adding the explicit IO adapter regression.
The 237-pass focused receipt is precommit evidence for the initial source change.
Neither receipt is relabeled as the current gate. The initial wrapper's unqualified
constructor spelling and wrong list quantity were harness errors; corrected
wrappers passed, and failed captures remain. Two invalid exploratory drafts also
expose parse/check diagnostic differences; they are separately retained without
original coverage credit or a waiver.

Current coverage is **287 / 1,302 originals with detailed receipts**. The remaining
1,015 are 360 checker negatives, one checking-only module, 101 interpreter cases
and 553 compiled candidates. Linked receipts include partial, blocked and historical
cells. The [acceptance criterion](cpu-parity-acceptance.md) still requires complete
applicable lanes, API coverage, preserved failure/resource contracts and a named
candidate gate, not a percentage inferred from test totals.

## Open imported-constructor printer discrepancy

Minimal helper module:

```bend
import Base
type Item is Data:
  Box{value: U32}
def make() -> Item: Box{7}
```

Separate main module:

```bend
import Base
import helper.bend as H
def main() -> H.Item: H.make()
```

| Lane | Exact outcome, repeated |
| --- | --- |
| Pinned interpreter / upstream C O3 / port pure CLI | Exit 0, `helper.Box{7}\n`, empty stderr |
| Port generated C/JS | Exit 0, `Box{7}\n`, empty stderr |
| Pinned generated JS | Empty streams; killed at the three-second bound, not a language exit code |
| Local non-imported control, all six lanes | Exit 0, `Box{7}\n`, empty stderr |

The original larger imported-result probe timed out at 30 seconds. The minimized
generated JS constructs tag `Box` via
[name_own/js_expr](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2980),
while its descriptor stores `helper.Box`. The
[show_val](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L5995)
search has no unmatched-tag termination. Source and generated bytes support this
diagnosis; no upstream file was modified. Port generated-name spelling and the
upstream JS failure remain open. Projecting constructor IDs/fields qualifies the
ragged branch behavior, not this printed-name contract. No new exception is adopted.

Next independent semantic work: `computed_ref_call_000/001`, `_f` and `_step`.
These six ledger-selected originals exercise computed calls passed to worker
definitions. An initial 20-vector grid can cover Nt depths 0..3 and both Bool
constructors. Only source/applicability review was done; no execution credit.
Existing erased-hole/TODO, resource/host/foreign, F32.read, competing-error-order,
external API and unfinished 688-vector requirements remain unwaived.

## Reproduction and storage

The frozen host profile is:

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Focused tests are `kernel_ragged_rows`, `compiler_ragged_rows`,
`cli_compile_executable` and library tests. Host debug overrides do not change Bend
runtime optimization. The primary upstream C command matches the pinned Linux
[CLI CPU profile](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253):

```sh
node "$BEND_REFERENCE/bend2/main.ts" "$FIXTURE" -o "$WORK/up.c"
clang-19 -std=c11 -O3 "$WORK/up.c" -lpthread -lm -o "$WORK/up"
"$WORK/up" --threads 1 --gpu off
"$BEND_BINARY" compile "$FIXTURE" --executable --target c --output "$WORK/port.c"
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$WORK/port.c" -pthread -lm -ldl -o "$WORK/port"
"$WORK/port" --threads 1 --gpu off
```

JS generation uses `.js` upstream output and port `--target javascript`, followed
by Node execution. Native IO uses `run`; pure evidence uses `run` and JSON `eval`.
The full gate took 318.64 seconds. Ordinary matrix children are bounded
at 30 seconds, audit children at 60, and outer runs at 1,200; the minimized known
JS printer loop has a separately recorded three-second bound. Exact commands,
source/generated-artifact hashes, outputs, statuses and cleanup are retained in
private receipts. Public ledgers retain compact hashes and no machine paths.

Disk preflight requires 10 GiB before large builds, with stop floors of 2 GiB in
the workspace and 1 GiB in temporary storage. The current gate's minimum workspace
headroom was 8.736 GiB. Rebuilding Cargo caches costs roughly
1.3 GiB; retained historical evidence dominates the broader storage footprint
(about 21 GiB under the tools/evidence root). Inactive Cargo caches alone were
reclaimed; source and unique historical evidence remain. The earlier preflight
shortfall stopped the build and is retained. Exact binaries use verified lossless
archives and temporary working copies. Two inactive example binaries were also
losslessly archived before retiring their Cargo hardlinks. Final headroom is
10.001 GiB, narrowly above the reserve; recheck it before the next build.
The optional extra 32 MiB cushion is not met. Do not delete unique historical
evidence to create that cushion.

Dependencies, Cargo.lock, upstream/docs references, toolchains, budgets, guards,
cancellation, approved demand behavior and network policy remain frozen. No
installation, push, desktop or GPU work occurred.

| Evidence | SHA-256 |
| --- | --- |
| Exact tested binary | `c66aebaf8a8befea10ed5b24b96fb49124c19944b437f03c707756d5c3c5a3a4` |
| Lossless binary archive | `f6a8bc7535a81860e37d93e503e4942cf63139159e6f423486daa9b03a50daf6` |
| snapshot-io.json | `a83c528586ce75c4d742bca4f6acef13caa6f0c10b607cf40043ee4e6e6300b5` |
| full-gate-io.json | `23c81158171dce1c91438d0029c45eef70a2e70cb95e5e4f82cfe3667d1f357e` |
| full-gate-io.log | `3ec33d506051f8f07adf829a7d23c65b01fd0966c44a80e76f4f7dc4f27c130f` |
| renewed-matrix/receipt.json | `fadaef0a4d08bb8e24f950b687d46fcc5ff822a6efb5c3ba9ab71e071f0fe45e` |
| io-qualified/receipt.json | `243bbfdb4ab48655b043a14772bb4ff7806de95f31aca9c1f8b758aaa89cf5c9` |
| printer-renewed/receipt.json | `2f902bd2d52aef9c807aad188f4147a2bc10adf52f711aef517c250d025b0199` |
| printer-phases-io.json | `1416c66939e4df0ddbd0d1fe05cbcf3a2f4f4b58234fc3070df26c591cff65de` |
| audit-io/receipt.json | `83cfb9cf9b18f1ef5516739e0665a9272740d2a84a69efca9bde3b0b03cfd964` |
| controls-io.json | `20323e462a3518c7a7f0f3ecbb7f9deb05ee8ea78fed24efd801f6e66d0ff422` |
| imported-printer-renewed/receipt.json | `e1e814ad840ac765072b055e2e78581ac16dce1a6bf4ca9e4fdeea922131780b` |
