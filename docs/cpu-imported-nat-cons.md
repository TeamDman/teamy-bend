# Fixed Nat literal constructors and cons qualification

Tested source `67cdcdff91f69e823b77157cf5dab7a48ecf350f` includes the parser
correction `6d12e3d69f8da10c14a17bf6dd98cc8cf7ef8eb3` and cons coverage. The
imported-Nat acceptance difference was a port defect, not a disagreement between
pinned upstream C and JS or a new policy choice. All six wrongly accepted
variants now reject. Constructor/parse diagnostic wording remains different
and unqualified. Full CPU parity remains incomplete.

## Fixed literal sugar, resolved explicit constructors

Pinned upstream `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599` spells Nat
[literal sugar](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L56)
with fixed Zero/Succ constructors. Its
[numeric parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2210)
constructs those names directly, without module alias resolution. Explicit
constructor names still resolve in their module. The
[module guide](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/guide/GUIDE.md#L461)
describes aliases; it does not promise that literal sugar captures similarly
named local constructors. The concrete pinned parser settles this distinction.

For example, this helper module declares a different type from the global Nat:

```bend
# helper.bend
type Nat is Data:
  Zero{}
  Succ{pred: Nat}
def value() -> Nat: 1n
```

```bend
# main.bend
import helper.bend as H
def main() -> H.Nat: H.value
```

`1n` introduces global Succ/Zero, so upstream rejects it against helper.Nat.
Replacing it with explicit `Succ{Zero{}}` is accepted, producing
`helper.Succ{helper.Zero{}}`. The port previously resolved the literal's names
as helper.Succ/helper.Zero and incorrectly accepted it. Two substitutions in
`Parser::number` now keep the sugar names fixed. No evaluator, checker rule,
demand contract, guard or execution budget changed.

The same correction covers zero, successor, open-tail literals, a trivial
local equality, numeric patterns and a local Nat shadowing imported Base.
`0n+Zero{}` adds no sugar constructors and retains its explicit local tail;
`1n+Zero{}` adds global Succ and rejects against the local type. Numeric patterns
without a declared global Zero reject during parsing/loading, as in the pinned
[pattern parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1689).
Explicit local constructors/patterns/operators, root-level Nat and ordinary
imported Base literals retain their positive regressions. The literal expansion
cap stays 128, with a rejection regression above it.

The original exploratory two-file false proof is preserved byte-for-byte in
`tests/fixtures/cpu-imported-nat/original`. It previously reached reflexivity in
the port after assigning local constructors. It now rejects the invalid literal
before reflexivity, matching upstream's cause. A separate trivial equality
control proves the fix prevents unintended acceptance, not merely a different
message for an already-false proof.

## Bounded comparisons and remaining diagnostic difference

Eleven authored cases run twice through the pinned direct CLI, C generation,
JS generation, and port check/run/eval. The old immutable f317911 binary is
retained; its receipt records a4765d3, the documentation-only descendant checked
out when that binary was invoked. New receipts identify clean 67cdcdf.

Before correction, six cases were incorrectly accepted in all three port lanes:
36 observations. After correction, the 132-observation matrix contains:

- 84 rejections across seven invalid cases, six lanes and two repetitions.
- 32 successful checking/interpretation observations for four positive controls.
- Four successful C/JS generations of the valid Base control.
- Twelve C/JS build-applicability refusals for three valid no-Base controls.

The pinned [CLI front end](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L134)
loads and validates the book before choosing either emitter. For every invalid
case, upstream direct/C/JS stderr and status are identical. Twenty-two structured
probes confirm six checking rejections, one parse/load rejection and four
acceptances, each twice; no program execution has started in those probes.
Their status 0 means successful observation, not acceptance of invalid code.

Valid no-Base controls check successfully but both upstream emitters refuse with
`Error: a build needs import Base`. That is not a language rejection or a port
failure. Positive port run text matches upstream direct output; typed check/eval
reports are validated separately. No runtime execution is claimed for the Base
control merely because its C/JS generation succeeds.

The [authored-case ledger](cpu-imported-nat-evidence.csv) preserves hashes,
phase, status and diagnostic distinctions. Raw constructor/pattern diagnostics
still differ: upstream supplies an expected/observed type or declared-constructor
message and Bend source excerpt; the port supplies its constructor error with
host context. Base shadowing specifically reports Nat versus helper.Nat upstream,
versus an unavailable Succ constructor in the port. Same rejection/cause is
not exact diagnostic parity, and no exception has been adopted. Every repetition
is deterministic; no comparison times out at the 60-second deadline.

## Production invoke callers

All production invoke paths were reviewed. `Command::invoke` and `Cli::invoke`
forward each subcommand's result. `lib::run` reads `CliOutput::exit_code()` before
emitting output and returns that code; the worker join preserves it and binary
main exits nonzero after owned guards drop. Cache/Home dispatchers also forward
results. No production caller discards the code; no caller change was needed.

Actual-binary regressions renew check/eval/run failures under text, JSON and CSV
settings: status 1, empty stdout, exact language stderr. The prior proof
presentation API caveat remains: `Ok(CliOutput)` alone is not proof acceptance;
callers must inspect exit_code. Public library checking still returns Err.
External downstream callers were unavailable and are not claimed audited.

## Cons applicability and results

The unchanged Apache-2.0 originals are copied with attribution into
`tests/fixtures/cpu-cons`. Both actually generate C and JS with the pinned
compiler, confirmed by four initial successful printer probes. The added
`parse-cons` suite in `scripts/compare-cpu.ps1` includes the originals and a
separately attributed pure-entry derivative of cons_operator.

| Original | Available generated/native output | Direct pinned CLI / strict proof applicability |
| --- | --- | --- |
| parse/cons_operator.bend | `391\n`, status 0, empty stderr | Direct IO is blocked by Node's missing upstream import.meta.require behavior; strict eval excludes the IO book |
| parse/cons_plus_pattern.bend | `12\n`, status 0, empty stderr | Direct normalization and strict eval pass |

Cons terms, character/list cons patterns and reusable `+h` head patterns agree.
The derivative changes only the IO main to return the original main.out as U32.
It adds no inventory row. Five generated/native lanes across three programs,
twice, give 30 exact executions; direct pure normalization adds four exact
observations. The two direct-IO observations remain host-blocked. The runtime
runner exits 0 for completed supported lanes, but preserves matched=false for
the blocked original; this is not six-lane completion.

Three pure entries additionally have 18 exact normalized-text, strict-report and
actual-core observations. Four strict eval observations reject the original IO
book. These exclusions do not substitute for executable comparisons. No cons
production defect was demonstrated.

## Validation and evidence boundaries

All current source receipts identify clean **67cdcdf**. Exact commands, streams,
statuses, deadlines, source/binary hashes and child-exit records remain in the
private cpu-imported-nat-cons evidence directory.

| Check | Result |
| --- | --- |
| Standard Linux gate | **852 passed, 0 failed, 49 ignored**, exit 0; 171.93 seconds |
| Focused parser/template/CLI/cons tests | **49 passed** |
| Internal depth/work guard tests | **5 passed** |
| Strict all-target/all-feature Clippy | Passed |
| Complete checking audit | 1,302 fixtures; zero status transitions from f317911; all 439 checker negatives reject; other accepted 366 / rejected 497 |
| Previous exact proof diagnostics and positive definitions | 144 renewed observations; pass |
| Cons comparison | 34 exact, two direct-IO host blocks; 18 normalization/core/report matches and four proof exclusions |
| Five-case CPU baseline, twice | 50 exact executions |
| 42-case normalization suite, twice | Exact structure/text; 330 successes, 108 printer refusals, six F32 proof exclusions |
| Six runtime-negative fixtures, twice | 44 exact, eight resource refusals, two native foreign gaps, two Bun host blocks, four C non-applicable observations; exit 1 |
| Ten demand controls, twice | 92 exact, six previously approved wildcard exceptions, two unresolved competing-error observations; exit 1 |

The standard gate includes six added tests, three parser and three cons. Existing
47 CUDA and two optional-profiler ignores remain; no assertion was weakened.
The first focused candidate failed because a test redefined Nat.add after Base;
the test setup was corrected and its failed log retained. Wider effects/comptime,
F32.read/error-order investigations and platform receipts retain their actual
historical identities. GPU, desktop and fresh Windows/macOS checks were unrun.

Runtime links increase from 81 to **83 originals**. Eleven runtime rows, all 38
interpretation rows, ten demand controls, three checking controls and the positive
list checking row are current. The other 72 runtime rows remain historical.
Runtime/interpretation union is **121 originals**; 1,181 remain unlinked to those
receipt kinds: 439 checker negatives, twelve checking-only, 153 interpreter-only
and 577 compiled candidates. These are applicability categories, not 1,181
missing executable tests. The [acceptance criterion](cpu-parity-acceptance.md)
still requires complete lane/API review and closure of required missing/failed
cells. Authored controls add no inventory rows.

## Reproduction and disk prevention

Use the existing activated tools, offline Cargo and BEND_GPU=off. This checkpoint
uses temporary `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0` to
omit Rust debug symbols. Optimization level, overflow checking, Bend budgets,
guards and cancellation are unchanged. No Cargo profile source, dependency,
lockfile, reference source or installed toolchain changed.

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
cargo test --locked --features extended_observability --test parser --test parser_templates --test cli_checker_diagnostics --test cpu_cons --quiet
cargo test --locked --features extended_observability --lib depth_tests --quiet
cargo clippy --locked --all-targets --all-features -- -D warnings
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
python3 <private-evidence>/proof-regression.py
python3 <private-evidence>/compare-checking.py --output <fresh-checking> --binary <snapshot-binary>
python3 <private-evidence>/audit.py
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-cons> -Suite parse-cons -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-baseline> -Suite baseline -Repetitions 2
python3 <private-evidence>/cons-normalization.py
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-normalization> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <fresh-negatives> -Suite negatives -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <fresh-demand> -Suite demand -Repetitions 2
```

C retains upstream CLI's O3 profile, -lpthread -lm, --threads 1 --gpu off and
strict port warnings. No new dependency or tool was installed; no push occurred.

Disk growth came from debug-symbol-heavy root test artifacts plus preserved
historical binary snapshots. Root-only cache reclamation did not satisfy the
old 12-GiB reserve, so the build stopped. After verifying target had no tracked
files or running products, cargo clean removed only rebuildable target artifacts;
protected historical snapshots kept their hashes. Its reported 2.7 GiB logical
removal freed less physical space because older snapshots retain hard links.
The old reserve still failed, and that build also stopped.

The symbol-free profile uses a **10-GiB prebuild reserve**, with unchanged
**2-GiB workspace / 1-GiB temporary-space stop floors**. This is an explicit build
storage adjustment, not a Bend resource-policy change. Full-gate minimums were
**9.63 GiB workspace / 4.38 GiB temporary**. Later validation preflight reclaimed
only inactive root products to restore reserve. Final verification had about
**9.69 GiB workspace / 4.39 GiB temporary**. No exhaustion occurred; source and
all prior evidence remain intact. Check headroom again before the next build.

Next independent family: parse/def_typed and parse/list_literal_sugar, selected
from compiled candidates without runtime receipts. Establish actual printer
applicability first, then compare typed/erased telescopes and list-literal/cons
behavior. These sources were inspected only; no runtime qualification is claimed.
Constructor diagnostics, resource/foreign/host, F32.read, competing-error,
loader/API/platform and wider-corpus requirements remain open.

## Receipt hashes

| Receipt/input | SHA256 |
| --- | --- |
| before/receipt.json | `639a5eeaf373eac0b33233d6eca7c40a179936ced828c9b1a29cdd17b012747a` |
| after/receipt.json | `24e67bb532ef0146753343961b1ac2eb61c721f6c2e50f444cfcc1e5a81fe2ea` |
| phase/receipt.json | `126bd7f123fb1417db69d7b7bec91b188f7924c14bffdcd125e49db56599bc22` |
| invoke-audit-renewal.json | `a4ad1a68e5fcb2a353eea579e991e71f7a726aeb5e29fbce50bf95b9dfec880e` |
| cons-printer/receipt.json | `f3102666ace8261b16ff08aec23370ca2887dad779a0f9eedf82bdcfc6aedcf2` |
| parse-cons/receipt.json | `02deba4f394f015185b84154fc0bb8f127996eb9045c88e27812b0fe19b0c28b` |
| cons-normalization/receipt.json | `6cb807a9822b85e0350b9e80f1c9c1178e30ee68de1511ff2674b3723269df52` |
| proof-regression/receipt.json | `48a2ef7b96af0fcbd67542cb78fb894d4c05df3b83b49db12e89533e66395bec` |
| audit/receipt.json | `8f90b01aa85f5695dad7166923e4bdeffea89d7e25d69a461ad479f3fd7a55c1` |
| audit/transitions.json | `45fd9869e882a251e1ef885c79b66228e1c5054bdc9cc94d7e4ef861eb971b91` |
| baseline/receipt.json | `5a4c2336bc8c9498ace3d3ad5f1aa9b61fba63ae661b32046b0c64957e3cecf7` |
| normalization/receipt.json | `daa09a080278b21359d1d431d99cc9778da2e5d954872ba1408566a1c07d277e` |
| negatives/receipt.json | `1e87a5cee391c564ae22c5f82a16eb8586c72699f93d1b3eba7311756972a815` |
| demand/receipt.json | `eba2288cbe8befeccd03e43c9d62f07dabfe6cf7a722423b9b68703869a514c4` |
| full-gate.json | `b2bbf9573a06f0217af6c4fb633ef1877628e65cbd4cfbe0de7696c7b60cf6e9` |
| verified-summary.json | `90e692f1fa473913b12455ac6166336ac11faff88ba40e78c850ec23eed669f2` |
| ledger-counts.json | `d2019b0ca8f0ff20562be45da2084c2b139c8bfed350bf238d04cbff79c70b49` |
| proof-regression/teamy-bend | `6841060ed93b5e3e2592cdacc76f8455bcc8868d3b6bef6a9d9b6dce1223aea4` |
| proof-regression/compare_normalization | `7fd59feadcafa5846dd87171f74794a6e07da1a93177d0147df89651fdb97d1c` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
