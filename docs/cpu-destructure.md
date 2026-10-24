# Field order, destructuring and local-binder diagnostics

Tested source **`de31e54e8e35f35b1c0fa4c05415d8dd91ca9b10`**, following checkpoint
`ca1f418796677dc4db49bff83181c8be29d59013`, qualifies eighteen unchanged originals
against Bend **`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**. Sixteen function
mains match exact normalized core/text and 399 concrete applications. The IO
original matches all five available execution lanes. The negative original
exposed a diagnostic provenance defect, now fixed without changing rejection.
Full CPU parity remains incomplete.

## Applicability and unchanged fixtures

The [committed originals](../tests/fixtures/cpu-destructure/README.md) preserve the
[pinned flatten sources](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten)
byte-for-byte, including golden records and Apache-2.0 attribution.

| Original family | Count | Evidence |
| --- | ---: | --- |
| field_match_in_order | 1 | IO execution: exact `9\n`, empty stderr, exit 0 |
| field_match_in_order_f and _g | 2 | Function core/text and concrete U32 applications |
| destructure_let_000 and four companions | 5 | Pair fields, reusable locals and helper applications |
| destructure_let_002 and _flip; _003; _004; _005 and _go | 6 | Field swaps, tuple/list reconstruction and selected values |
| nested_destructure_let; destructure_then_match_000 and _001 | 3 | Nested fields and subsequent matching/application |
| destructure_let_001 | 1 | Required parse/load rejection with exact diagnostic and source line |

Fourteen function originals import Base; actual pinned C and JS generation
refuses their function-valued main printer. The two destructure_then_match
originals instead fail the upstream Base prerequisite; the port refuses their
function printer. These original build diagnostics are different and are not
claimed equal. Eight separate authored observations add Base and prefix local
names, confirming the C/JS function-printer refusal twice for both originals.
No upstream source was modified. The pinned
[show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1203)
requires a supported result printer before standalone emission. Function
normalization and direct checked applications do not claim generated execution.

For the IO original, upstream C at O3, upstream JS under Node, port C with strict
warnings, port JS and native Rust each run twice: **ten exact executions**.
Upstream C receives `--threads 1 --gpu off` and links `-lpthread -lm`.
Direct upstream IO remains host-blocked twice with exit 1 and
`TypeError: require is not a function`. The receipt records completed runtimes
matching while full-lane match remains false. Port strict checking separately
rejects `IO`; that proof-mode exclusion does not replace executable evidence.

## Diagnostic repair

Both implementations already rejected destructure_let_001. The port attached a
public parser error at EOF; upstream reports the original local binder at line
14 (`q = p`). The pinned
[body_flatten Local branch](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2772)
rejects the non-lambda tail using the binder's source span. The fix follows that
structured cause rather than matching fixture text or rewriting host strings.

A private flattening error variant carries the existing binder ID. An optional,
loader-owned source index maps that ID to source text/line, and the existing
checked CLI diagnostic writer renders the pinned message and excerpt. The
private load-diagnostic envelope now accommodates both unknown-pattern and
local-binder failures. Existing debug tracing retains the internal parser error.

The public ParseError message, Display text, source API and rejection remain
unchanged. No fields were added to core terms or bodies, no strong term edge or
serialized metadata was introduced, and flattening order and the 64-level
guard remain unchanged. Controls compare ordinary/tracked books, checking work,
fuel/depth/fresh counters and normalized results. Writer failure, flush,
cancellation and unrelated diagnostic controls remain passing. Existing external
CLI exit-code and executable-emitter API caveats from the
[earlier report](cpu-compile-rewrite-diagnostics.md) remain; no claim of an
external consumer audit is added here.

The unchanged original plus five authored direct/nested/imported/template/
multiline cases give **60 observations**: 48 exact raw stream/status observations
and twelve parse/load phase probes. Check, eval and run emit no stdout and
exit 1 with the pinned stderr; unit/integration controls also exercise text,
JSON and CSV invocation modes. No program execution starts for these negatives.

## Application coverage and performance limit

The independent pinned adapter captured **1,087 distinct application vectors**
twice (2,174 values). The initial Rust full-grid candidate ran for over four
minutes and was stopped after verifying the exact task-owned test process.
Its Cargo exit 101 and partial output remain preserved, not relabeled passing.
A separate 30-second timing probe timed out; three targeted maximum-bit-pattern
multiplications completed correctly in about six seconds each. No incorrect
value or safety-budget change was demonstrated.

The committed regression selects **399 vectors** from the captured oracle table:
zero/asymmetric/wrap-sensitive cases for each of five multiplication workers,
three dot-product cases, plus all other selected field/list cases. Inputs include
five U32 values, Nat depths zero through four and lists of lengths zero through
three. The new bounded regression checks exact values and actual core structure;
it passed the focused run and full gate. The remaining **688 exploratory vectors
are not fully port-grid-qualified**. The full oracle table, selection indices,
interrupted candidate and timing evidence remain private. No existing test was
removed or weakened; no resource limit, evaluator or demand rule changed.

## Validation at the tested source

| Check | Result |
| --- | --- |
| Standard format/strict Clippy/build/provider/test gate | **889 passed, 0 failed, 49 ignored**; 98 result-bearing targets; exit 0 |
| Focused normalization/CLI diagnostic candidate | 40 passed |
| Focused library diagnostic/origin candidate | 14 passed |
| Exact normalization suite | 106 cases, 1,200 observations: 842 successes, 352 artifact/printer refusals, six strict F32 exclusions |
| New IO C/JS/native matrix | Ten exact executions; two direct-oracle host blocks retained |
| Baseline upstream C/JS and port native/C/JS | 50 exact executions, two repetitions |
| Local-binder diagnostic/phase matrix | 60 observations pass |
| Earlier rewrite diagnostic/phase matrix | 60 observations pass |
| Earlier false-list/brace and positive proof controls | 144 observations pass |
| No-main checking, compilation and named normal forms | 306 observations pass, including 72 exact build refusals |
| Corpus checking-status audit | 1,302 rows, zero transitions from 062c7ec |

The normalization suite contains 99 originals and seven authored controls.
Every final repeated observation is deterministic with no timeout or abnormal
exit; expected refusals and host blocks retain their own categories. Exploratory
timeout/interruption evidence above is separate. All 439 expected checker
negatives still reject; the other statuses are 366 accepted and 497 rejected.
Status agreement alone is not semantic or diagnostic parity. The 49 hardware/
profiler ignores remain unrun. Other historical negative, Nat/List, demand and
host matrices retain their actual tested commits; full-gate inclusion does not
turn them into renewed differential receipts.

## Remaining acceptance coverage

| Category | Inventory | Detailed originals | Without detailed receipt |
| --- | ---: | ---: | ---: |
| Checker negative | 439 | 9 | 430 |
| Checking only | 12 | 11 | 1 |
| Interpreter only | 164 | 58 | 106 |
| Compiled candidate | 687 | 130 | 557 |
| Total | 1,302 | 208 | 1,094 |

Detailed links include historical and unresolved cells. There are 89 runtime,
99 interpretation and twenty checking rows; runtime/interpretation union is 188.
This source renews all 99 interpretation rows, eleven checking rows and four
runtime originals. Nine checking and 85 runtime rows remain historical. Argument
vectors and authored controls add no inventory rows. The sole unlinked
checking-only module remains io/far_types, requiring foreign applicability review.

The [acceptance criterion](cpu-parity-acceptance.md) remains unmet: complete
per-lane/API review, resource-limit failures on original programs, foreign/host
support, F32.read and competing-error order are still required. Missing coverage
is not converted into a pass, an unsupported host or a GPU requirement.

Next coherent independent batch: the four positive parse/plus_binder originals
(case, lambda_do, nested and row), with the negative term-position companion.
They exercise reusable binders across patterns, lambdas and do/let forms, directly
adjacent to this machinery. Their source/golden review is selection evidence
only; actual printer, strict checking and executable applicability remain to be
qualified. The negative must be checked at its actual phase and diagnostic.

## Reproduction and resources

After activation of the existing frozen environment, exact command families are:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_normalization --test cli_checker_diagnostics --quiet
cargo test --locked --features extended_observability --lib diagnostic --quiet
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -Suite destructure -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -Suite baseline -Repetitions 2
python3 <private-evidence>/qualify.py --output <fresh-name> --snapshot final-snapshot
python3 <private-evidence>/local-binding-matrix.py
python3 <private-evidence>/diagnostics.py
python3 <private-evidence>/proof-regression.py
python3 <private-evidence>/audit.py
python3 <private-evidence>/verify.py
```

The private cpu-destructure evidence directory preserves argv, source/binary/
adapter hashes, raw streams, statuses, timings, snapshots and cleanup receipts.
Fixed-output tools require fresh destinations. Child limits remain 30/60 seconds,
outer limit twenty minutes; workspace/tmp stop floors remain 2/1 GiB. Four Cargo
jobs, debug=0 and disabled incremental compilation are host build controls,
separate from Bend runtime optimization and budgets.

Every build began above the 10-GiB reserve. Full-gate minimum was
**9.13 GiB**. Post-validation test/root/check-metadata
cleanup alone could not restore that reserve. Inactive compiled dependency
archives/metadata were also reclaimed, with protected binary hashes checked;
locked offline sources and all source/evidence snapshots remain. The next build
will regenerate those caches under frozen inputs. Final reserve is
**10.04 GiB**. This task's evidence occupies about 198 MiB; retained
tool/evidence storage is about 20 GiB, while the remaining build cache is about
395 MiB. Evidence growth now dominates disk use. Continued milestones need a
fresh headroom check; once rebuildable caches cannot meet the reserve, additional
storage is required rather than deletion of historical evidence or source.

Rust 1.96.0, nightly-2026-05-24 formatting, Node 24.19.0, PowerShell 7.5.4 and
Clang 19.1.7 remain frozen. Port C retains strict C11 warnings and
`-pthread -lm -ldl`. Cargo.lock SHA256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Upstream and teamy-docs `50e89d6897ce2a65167ef65b17c5fa83fe849158` remain clean.
No installation, dependency adoption, push, desktop/GPU work, security-policy
change or new semantic exception occurred.

## Receipt identities

| Evidence | SHA256 |
| --- | --- |
| no-main-final/receipt.json | 94c8f8d08702491eca9e55eee80ee92beee0de2123a5376b3351aa8e035344ae |
| normalization/receipt.json | 2abe0de69c688a1583e146e3289fb5eb2578688566390a13ce950a17fc9464e7 |
| applications-qualified/receipt.json | a415c3847ed7466611aedc9e2e80af416d940cb468135aac3a22f684bc204b98 |
| printer-probes/receipt.json | 6973274b4cff7e6eba599833b76d3800462eb162e72f8be20c35740b3b55d42d |
| regression-selection.json | ada01bd7ba4f4053b43bb3a3ec0d6640c94eeeb07ade85c2b29c34c50efc413a |
| audit/receipt.json | cda4205165cc10cae8e4a2f8b8b367b7aae66f55420f8f887eac70908b1fde16 |
| baseline/receipt.json | 05138a0366b603c672920af68a49bdff9c391fa4229a238a75074d213af59a51 |
| runtime/receipt.json | 841eec6a7088c24f50cb97b532a59bcfb36323cde393f84a6e2a2d3bb1335aa7 |
| full-gate.log | ca40ecbe6cfcb72ab3accf149c4c0b5f4be02152e457bd932adca38788e4f631 |
| full-gate.json | 0b4d137dc1444ab214a22770bb1d04a4ae5c9afbeaa46d35fb39e286d212a6d0 |
| diagnostics/receipt.json | d7d47256854534a27aaaaa3e71579465c58d3c419ae77167d03d776fd05fc909 |
| local-binding-final/receipt.json | 3d44c86a401992f924c3c4da7dc24185bb5b9c730db3f291ffd170c745e51784 |
| proof-regression/receipt.json | f92dafad33ca91570b0aadf43e8cb6e0ecacf7761d6c6a5edab48ece9a794e8e |
| oracle-provenance.json | 1a727f6f749188036a51a73d223eb020d430186595c994f861b9a7604ef45edc |
| pre-gate-cache.json | 05aebc48eed5478d4c52e860a85ca38cdc527148e9ee38eb8ab56b166eb26ec0 |
| pre-gate-root-cache.json | bf47cb49cff4636520ac6cbd4200e6bcf7e862a037822f6373854b3823f78ce7 |
| focused-candidate.json | 2d6bd23d124ba551ceefd6d1a68fb1528f17c5deeb0bceb3838c91aa7af7f46d |
| candidate-test-stop.json | ee692a99737ca9b6606ec8d62b0430d420452e24d586d726bb309968f523a1ed |
| timing-boundaries-receipt.json | 7a4fa42976b95fd84e6f882093a58442e54d6a6be2f1e99cf329f2fa0e2ddf22 |
| grid-timing-receipt.json | 50c5a800ed499f6970d35addf7b526a64d58b3b23e081ee6e3c4a0c2c4b34b8c |
| final-cache-preflight.json | 3eedfa9f620d514219687ac7f9aade50b26194e4f1cace7f36e2b43b22e9ec1c |
| final-root-cache.json | 9e5b9d6a7b81b2f38141243fd680d39f49e6dd81b2a38c6ca5b346aa613794a1 |
| final-check-metadata.json | 4b2ad243d10756ea23813e0d6a3d9288e06631f93fb0fa920e76eb8bf24b15f5 |
| final-dependency-cache.json | 7c669b420f81b9d52fb01b521df3efb648198c5c4bd5197a4a0a98783c3df8b9 |
| final-cache-complete.json | 3c303d497d54b1ca96aaadcdd8fc939c06ae7aea5d255764f1298fed6d66f8d2 |

Saved all-feature CLI SHA256: `6633e8369d2a965e65279360619c73d6a88016e5d5d38192720f65bd6e688d5c`.
Saved normalization probe SHA256: `9de341a206fd3f4c8ef39027792fdec1638482264903e1d2484b74e3fc5d5ca5`.
Normalization-run CLI SHA256: `7a1ad32bccc44a04b553592799a2508b2e985402dd6c28a5a19a68afeeccac1e`.
Runtime-run CLI SHA256: `7a1ad32bccc44a04b553592799a2508b2e985402dd6c28a5a19a68afeeccac1e`.
The Cargo test-feature binary and all-feature binary are separately identified;
all were built from the same tested source. The later documentation checkpoint
reports this source and does not claim a second full-gate execution.
