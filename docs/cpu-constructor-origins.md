# Constructor provenance and pattern-load diagnostics

Tested source **`d986964456afaf7622628144da2e2745db740691`** closes the four
remaining diagnostic cases from the constructor checkpoint
`e9af40fd8be18206e3fb0ab46476f5b3fc0e70f9`. The target remains upstream Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

Substituted `1n+n` and `Wrong{n}` now retain their actual Bend source locations.
Nat/List unknown-pattern failures now match the pinned parse/load diagnostic.
All four preserve rejection phase, empty stdout and exit 1. No language,
acceptance, demand, evaluation or resource policy changed. Full CPU parity
remains incomplete.

## Mechanism and invariants

Parser flattening and kernel substitution rebuilt constructors, losing the
identity used by the optional source map. An explicit, private provenance table
now observes the actual inputs and outputs of substitution and operator
qualification. It follows corresponding children of those transformations,
skips replaced variables, and preserves a replacement's own origin. It does
not search for similar syntax, infer a source line from names or error strings,
or attach fields/annotations to a Term.

The existing public substitution implementation and Term/TermRef types are
unchanged. Observation occurs after the original transformation returns; it
cannot select a different result allocation. Weak anchors prevent pointer
reuse without retaining term values/fields or increasing strong ownership.
Dead transformed entries are retired periodically. Source spans retain their
own weak allocation anchors and immutable source text. There is no unsafe
pointer dereference or global/thread-local registry.

Weak anchors are confined to the private **immutable** load/check path, which
never calls Rc::get_mut or Rc::make_mut on a TermRef. The tracker is cleared
from successful checked engines, and the CLI drops its source map before
normalization or execution. Public loaders/checkers remain available without
tracking. Source metadata does not enter the AST, result schema, generated
program, evaluator or serialized language output.

Regressions verify unchanged term/debug representations and printed values,
shared child pointers, replacement identity, strong reference counts, release
of discarded term values, and preserved provenance across 1,024 substitutions.
Tracked and untracked checking use identical fuel, depth and fresh-name
counters for accepted and rejected books. Successful normalized definitions
also agree after the tracker is dropped. Existing budgets, guards and
cancellation behavior remain unchanged; metadata housekeeping is not a new
Bend resource allowance or acceptance rule.

Unknown constructor patterns use a separate private loader failure wrapper.
The parser records the structured constructor name and actual source excerpt
at the rejection point; that failure propagates without recovery. The CLI
formats it through the same checked stderr writer. Public ParseError fields,
Display and public loader signatures remain unchanged. Untracked/unrelated
load errors retain the host-error fallback. Cancellation and write/flush
failures continue to propagate; stderr alone cannot produce success.

This follows pinned [unknown-pattern rejection](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1702)
and [source-aware error formatting](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1437).
The [constructor inference/checking rules](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3399)
are unchanged. The fix handles generic unknown constructor names, rather than
special-casing Zero, Nil or individual fixtures.

## Qualification

The previous 36 inputs, including all five pinned originals and all helper
files, are unchanged and hash-verified. Six authored controls add nested fields,
imported nested fields, match flattening, operator qualification, template
instantiation and an unknown inner pattern constructor. The existing
context-open fixture is unchanged; its regression now requires exact diagnostics
instead of the historical safe host fallback. No rejection test was weakened.

The **42-case matrix has 588 deterministic observations**, with 60-second child
deadlines and no timeout or abnormal exit:

- **31 checking negatives and three parse/load negatives** match exact stderr,
  empty stdout and status across upstream direct/C/JS front ends and port
  check/eval/run, twice: **408 exact rejection observations**.
- Eight positives retain all 96 previous observations: 64 successful checks or
  pure interpretations, 24 verified no-Base build refusals, eight successful
  generations. These generations are not claimed as program executions.
- **84 direct phase probes** distinguish checking, parse/load and acceptance.
  Port regressions separately assert load-versus-check failure. Rejected
  generation requests leave no C/JS artifact.

The scoped qualification summary now exits **0**, with no remaining discrepancy
in this matrix. The preceding checkpoint's exit-1 receipt and all historical
inputs/captures remain intact. This does not establish diagnostic parity for
all parser/checker failures or unreviewed language facilities.

| Validation at clean d986964 | Result |
| --- | --- |
| Standard check-all.ps1 gate | **871 passed, 0 failed, 49 ignored**, 97 result-bearing targets; exit 0 |
| Focused parser/templates/diagnostics/normalization | 77 passed |
| Depth-guard controls | 5 passed |
| Diagnostic/cancellation/writer controls | 10 passed |
| Origin controls | 5 passed: three new provenance controls plus two existing sealing controls |
| Prior false-proof/adjacent/positive/normalization matrix | 144 successful observations |
| Full normalization suite | 49 cases; 386 successful observations, 124 verified build/printer refusals, 6 strict F32 exclusions; exact structure/text |
| C/JS/native serial baseline | 50 exact executions across five runtimes, two repetitions |
| Full pinned checking-status audit | 1,302 observations; **zero status transitions** from 6442b2ca1362e36f84b74044091a0151fb17726d |

All 439 expected checker negatives still reject. The other audit outcomes are
366 accepted / 497 rejected; those rejections remain status evidence, not
qualified compatibility. The 49 ignored hardware/profiler tests remain unrun.
Runtime-negative/demand matrices retain their prior actual receipts and were
not newly executed here. Candidate compiler/Clippy failures and the summary
filter-count correction are retained separately; final receipts identify the
actual clean tested source.

## Remaining CPU acceptance work

Detailed original coverage is unchanged; six authored controls do not add
upstream inventory rows:

| Category | Inventory | Detailed | Without detailed receipt |
| --- | ---: | ---: | ---: |
| Checker negatives | 439 | 8 | 431 |
| Checking only | 12 | 2 | 10 |
| Interpreter only | 164 | 15 | 149 |
| Compiled candidates | 687 | 115 | 572 |
| Total | 1,302 | 140 | 1,162 |

Detailed includes historical and unresolved outcomes, not completed coverage.
The runtime/interpretation union is still 130 originals. All 42 interpretation
rows, three runtime originals, six original checking rows, and the proof/Nat/
List controls are renewed; 85 runtime rows and four checking rows remain
historical. The [acceptance criterion](cpu-parity-acceptance.md) is unchanged.
F32.read disagreement, competing-error ordering, whole-program resource limits,
foreign implementation support, unavailable oracle hosts and complete lane/API
review remain open. No new semantic exception was introduced.

Next independent bounded category: the nine ordinary no-main modules without
detailed checking receipts; review io/far_types's foreign mode separately. This
CPU work does not require desktop or GPU access. General constructor provenance
through arbitrary evaluator transformations is not claimed by this scoped
substitution/qualification fix.

The documented CLI status boundary also applies to tracked pattern-load errors:
these return status-1 CliOutput. Repository production callers inspect that
status before emission and process exit; external consumers remain unaudited.
Public ParseError and AST shapes are unchanged. The earlier KernelError
private-field construction caveat remains, with no additional public-field break.

## Reproduction and resource controls

After activating the frozen environment:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
cargo test --locked --features extended_observability --test cli_checker_diagnostics --test parser --test parser_templates --test kernel_normalization --quiet
cargo test --locked --features extended_observability --lib depth_tests --quiet
cargo test --locked --features extended_observability --lib diagnostic --quiet
cargo test --locked --features extended_observability --lib origins --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-normalization> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-baseline> -Suite baseline -Repetitions 2
python3 <private-evidence>/compare.py --binary <immutable-snapshot> --output <fresh-name>
python3 <private-evidence>/audit.py
python3 <private-evidence>/proof-regression.py
python3 <private-evidence>/summarize.py
```

Private cpu-constructor-origins evidence retains exact argv, snapshots, input/
binary hashes, raw streams, exits, timing, determinism and disk samples. Scripts
with fixed output targets must be copied to fresh destinations before rerunning.
The [constructor ledger](cpu-constructor-evidence.csv) and other public ledgers
contain sanitized identifiers/hashes; no raw host captures or private paths.

Rust 1.96.0, nightly-2026-05-24 fmt, Node 24.19.0, PS 7.5.4 and Clang 19.1.7
remain frozen. Host debug=0 and CARGO_INCREMENTAL=0 control rebuildable cache;
four build jobs remain. They do not change Bend runtime optimization or budgets.
The C oracle separately uses pinned O3 / -lpthread -lm and --threads 1 --gpu off;
port C retains strict C11 warnings and -pthread -lm -ldl without a new O3 flag.

All build preflights met the 10-GiB reserve. Full-gate minimum workspace
headroom was **9.70 GiB**, above the unchanged 2-GiB workspace / 1-GiB temporary
stop floors. Focused renewals reused gate artifacts. After completion, only
inactive rebuildable test executables were reclaimed, restoring **10.30 GiB**;
protected snapshots, source and historical evidence were verified intact.
Incremental cache remains disabled. No test is left running.

| Evidence | SHA256 |
| --- | --- |
| Final 42-case matrix | eb8d0d512fb347d6b45b77f47b7b1094a6fd8827fd1734d2194c8cb8aeae5ef3 |
| Checking-status audit | bc544afd3b2c8360a8b7f5b68626003660b36796e5b9461df6e6a01f32a63c68 |
| Proof regression receipt | 0466acfddb02da89ae7fa310a6ce7a44c2395ebe636fb357dd38f5fa64c0bc91 |
| Full normalization receipt | 9f779213ef8ecd4913a6ec92d37b2855b47fa41634a5b6b29baa7147325462e6 |
| C/JS/native baseline receipt | c2ae8eb5bb7568e97d9b308608b9a8d123df2008718b3d02c9fef1b2d8815d57 |
| Full gate log | 7f9b997aa75d46cecbc71663acf7f8b3c9a953e3e4e29d075dc158be8f6259e9 |

Cargo.lock remains SHA256
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
The upstream and teamy-docs checkouts remain clean at their recorded pins.
Dependencies, toolchains, references, security/network policy, guards and
cancellation stayed frozen. No install, push, desktop or GPU work occurred.
The later documentation checkpoint records these results without claiming a
second execution of the gate.
