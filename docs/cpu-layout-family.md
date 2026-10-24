# Parser layout family and remaining CPU coverage

Tested source `f29284fdc0649b11478d344d6a0c688582ce4c76` qualifies seven unchanged
layout originals against pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. It expands the requested pair to
adjacent case-body, tab, branch, sibling-case and do-bind layout rules. No new
acceptance, evaluation or presentation mismatch was demonstrated in this family.
The change adds three regression tests, attributed fixtures/derivatives and
bounded comparison-suite coverage. Production source and approved semantics are
unchanged. Full CPU parity remains incomplete.

## Applicability before comparison

| Original | Result | Required observation |
| --- | --- | --- |
| parse/match_indent_column.bend | `Ff7{}` | Pure normalization; shallower top-level case rows |
| parse/ctor_row_layout.bend | `Ss{Ss{Ss{Zz{}}}}` | Pure normalization; inline constructor rows |
| parse/case_body_newline.bend | `Ff{}` | Pure normalization; inline and multiline case bodies |
| parse/tab_character.bend | `Qv1{}` | Pure normalization; actual tab bytes preserved |
| parse/if_branch_newline.bend | `7\n` | Generated C/JS and native IO; indented branch body |
| parse/elif_alignment.bend | `56\n` | Generated C/JS and native IO; uneven sibling case indentation |
| parse/do_line_depth.bend | `1n\n` | Generated C/JS and native IO; shallow do-bind line |

For the four no-Base originals, upstream direct interpretation succeeds while
both emitters refuse with `Error: a build needs import Base`. Fresh repeated
checks verify status 1, empty stdout, exact stderr and no generated artifact.
These are build applicability refusals, not failed pure evaluation and not
function-printer refusals. No Base import is added to the originals.

The three IO originals actually generate C and JS. All four generated-program
lanes plus native IO preserve exact stdout, empty stderr and status 0 twice:
**30 exact executions**. Six direct-upstream-IO observations remain blocked by
Node's missing import.meta.require behavior. The runner retains matched=false
for those original rows even though supported lanes complete with exit 0.
No Bun installation or host substitution is used. Original strict eval remains
outside proof mode, with six repeated `main: undefined name IO` refusals.

Three attributed `_pure` derivatives replace only each IO main with its unchanged
main.out value. They are normalization controls and add no inventory rows; their
generated execution is not claimed. The full normalization suite expands from
42 to **49 cases**. The seven new cases provide **56 exact observations** across
pinned interpretation, port pure run, strict eval reports and actual-core checks,
plus **16 verified no-Base build refusals**. The three named main.out definitions
add **18 matching normal-form/report/core observations**. Pure CLI run dispatches
to canonical normalization; it is not counted as native IO execution.

The [pinned parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2312)
uses the enclosing case-row column, not a general Python-like indentation rule.
A nested match's first case must be farther right than its enclosing row, and
subsequent rows stay at or to the right of that first case column. The qualified
constructor rows, case bodies and do binds accept the demonstrated layouts. This
does not establish that every newline is interchangeable in application spines.
The port's existing parser follows the tested rules. Existing nested-match and template controls are
renewed by the focused tests; parser/engine limits stay unchanged.

All seven original byte hashes match the clean reference. The initial
35-observation probe used the saved 8218011 binary with documentation-only
f3b185e checked out; it retains that identity. Final receipts identify clean
f29284f. Every repeated final comparison is deterministic and completes within
its 60-second child deadline. There is no new oracle exception.

## Validation and current receipt scope

| Check | Result |
| --- | --- |
| Standard Linux gate | **861 passed, 0 failed, 49 ignored**, exit 0 |
| Focused parser/templates/layout tests | **48 passed** |
| Internal depth/work guard tests | **5 passed** |
| Strict all-target/all-feature Clippy and pinned formatter | Passed in the standard gate |
| Three IO originals, twice | 30 exact executions, six direct-IO host blocks |
| Full 49-case normalization suite, twice | 386 successes, 124 verified printer/build refusals, six existing F32 strict-proof exclusions; exact structure/text pass |
| Supplemental named definitions / original strict IO | 18 exact observations / six proof exclusions |
| Five-case baseline, twice | 50 exact observations |

The full gate retains 47 CUDA and two optional-profiler ignores. No warning,
assertion or safety guard was suppressed. The broad checking-status audit,
runtime-negative/demand matrices, Nat/List diagnostic controls and other effects/
comptime/error-order/F32.read/platform receipts are not renewed by this test-only
change. They keep their actual commits and unresolved cells. GPU, desktop,
fresh Windows/macOS and newer upstream remain unrun.

Six runtime rows (three new originals plus three baseline originals) and all 42
interpretation rows are current. The other 82 runtime rows remain historical.
The runtime ledger has 88 originals; runtime/interpretation union is 130. The
five separately recorded original checking rows bring the detailed-evidence
union to 135. A linked row can contain a required failure or host block; neither
union is a completion count. The [acceptance criterion](cpu-parity-acceptance.md)
still requires complete mode/API review and closure of required missing/failed
cells on a named candidate/profile.

## Remaining coverage by category

The table combines the runtime, interpretation and original-checking ledgers.
A detailed row means a recorded comparison, including known mismatches; it does
not mean the fixture or its category is complete. The historical status-only
1,302-case audit is separate. Counts are derived from the frozen inventory and
source-hash-verified applicability data, with no new execution of unlinked cases.

| Required mode under pinned gate | Inventory | Detailed original rows | Without detailed receipt | What remains |
| --- | ---: | ---: | ---: | --- |
| Checker-negative | 439 | 3 | 436 | Three originals have phase/construct agreement but raw diagnostic differences. All 439 were rejected in the historical status audit; status alone does not qualify any exact diagnostic contract. |
| Checking without main | 12 | 2 | 10 | Two exact checking rows; nine remaining ordinary proof/type modules and the foreign helper io/far_types need their own appropriate-mode comparison. No main execution is invented. |
| Interpreter-only gate selection | 164 | 15 | 149 | Exact normalization and applicability for the remainder; 120 unlinked entries are in flatten. This category also contains reg/framed_cell_dep with IO despite its compiled-selection exclusion. |
| Interpretation plus compiled candidates | 687 | 115 | 572 | 88 runtime rows plus 27 actual function-printer-refusal/normalization rows. Remaining lanes need actual applicability, execution, streams/status and host review. |
| **Total** | **1,302** | **135** | **1,167** | Linked rows still contain failures, host blocks and historical evidence. |

The separate runtime/interpretation-only count remains 130 linked / 1,172
unlinked. Adding the five original-checking rows explains the five-row difference;
authored controls never increase either inventory union. The
[category CSV](cpu-coverage-summary.csv) and retained per-original coverage map
make these denominators reproducible. There are no optional examples hidden
outside these four gate categories in the 1,302-file inventory.

The largest unlinked compiled-candidate directories are reg (85), compile (83),
check (64), io (64), flatten (51) and run (45). These directory names are not
semantic families or permission to run all their contents blindly. Actual
function/data printer applicability and strict-versus-executable modes still
need review. There are also 131 unlinked checker negatives in check, 60 in parse,
54 in flatten and 33 in proof; batching common error causes offers more useful
coverage than unrelated single-file passes.

Across the 88 historically linked runtime rows, required non-passing cells still
include four resource-limit cells (one native, two port C, one port JS), one
native foreign gap and six upstream-JS Bun-FFI blocks. Two upstream-JS passes
need their recorded stack4096 profile. Direct upstream CLI has 34 host-blocked,
48 unverified and six passing rows. These aggregates retain mixed historical
commits; only six runtime rows are newly qualified here. The two C lanes that
are not required for a JS-only companion are applicability decisions, not passes.
F32.read, competing-error ordering, constructor diagnostics, large library/resource
work and complete loader/API/platform review remain separate open requirements.

Among originals without detailed receipts, source-profile hints overlap:
19 foreign companions, fourteen network, nine filesystem, eight clock/timer,
three environment, one window and one audio. These are review flags, not proven
unavailable hosts. Eleven unlinked offload-annotated candidates still need CPU
fallback qualification; their annotations do not make GPU hardware a CPU blocker.

**Recommended next bounded batch:** qualify the five original constructor/import
negatives `check/nat_literal_import`, `check/tuple_import_required`,
`import/list_sugar`, `parse/qualified_constructor` and
`page/ctor_list_mismatch_000`. Their original expected diagnostics describe the
same undeclared/wrong-family constructor boundary exposed by the existing Nat/
List controls. Investigate phase, normalized offending term and Bend source
location together, then correct demonstrated presentation defects without
changing rejection or demand semantics. One comment in import/list_sugar claims
a parse failure, while its expected diagnostic has checking context; use actual
pinned phase probes rather than that comment. These five sources were inspected
only in this checkpoint, not newly executed or qualified.

This recommendation addresses a known shared gap in the largest under-qualified
category. A subsequent small category-closure batch can compare the nine remaining
ordinary no-main modules while keeping io/far_types's foreign mode distinct.
The broad interpreter gap is mostly flattening/normalization; group its next
batch by shared pattern/closure machinery instead of filename order. No additional
semantic exception or dependency adoption is proposed.

## Reproduction and cache prevention

Use the existing activated toolchain with these temporary host settings:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test parser --test parser_templates --test cpu_layout --quiet
cargo test --locked --features extended_observability --lib depth_tests --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-layout> -Suite parse-layout -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-baseline> -Suite baseline -Repetitions 2
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-normalization> -Suite all -Repetitions 2 -RequireExactText
python3 <private-evidence>/named-normalization.py
```

`-Suite layout` selects just the seven new normalization cases. Exact argv,
streams, exit statuses, deadlines, source/binary hashes and determinism checks
are retained in private cpu-layout-family receipts. The snapshot and evidence
scripts use fresh output directories; source in the reference is never changed.

Host Rust debug=0 omits symbols. **CARGO_INCREMENTAL=0 is newly explicit for this
checkpoint**: it disables host compiler caching to prevent recurring incremental
cache growth. Neither setting changes Rust optimization/overflow checks or Bend
runtime policies. Four build jobs remain. Separately, the upstream C oracle uses
its pinned CLI O3 profile, -lpthread -lm, --threads 1 --gpu off. Port C uses
-std=c11 -Wall -Wextra -Werror -pedantic, links -pthread -lm -ldl, and receives
no new optimization flag. JS uses the installed Node.

The initial 10-GiB preflight stopped the build after root-only cleanup left a
small shortfall. Inactive Cargo incremental products were identified as additional
growth. After checking for active builds/target processes and tracked target
files, only that rebuildable cache was reclaimed; protected previous snapshot/
receipt hashes stayed unchanged. The retry retained the **10-GiB reserve** and
**2-GiB workspace / 1-GiB temporary stop floors**. No source or historical evidence
was deleted, and no exhaustion occurred. Dependencies, lockfile, toolchains,
upstream/docs pins, security policies, Bend budgets and cancellation remain
frozen. Nothing was installed or pushed.

The full gate completed in 181.98 seconds, with minimum headroom
**10.15 GiB workspace / 4.38 GiB temporary**. After validation,
headroom was 10.01 GiB / 4.39 GiB. The incremental cache did not regrow during these
builds. Preflight remains mandatory before the next build.

## Receipt hashes

| Receipt/input | SHA256 |
| --- | --- |
| initial/receipt.json | `82bd23d6a45efac623eb2b27b25feb49c0f13c2dfcb64d30e4305218ae1799f4` |
| parse-layout/receipt.json | `9d208c72ffaa76fb247d4c6fe250734f8327643b1a6b092b538cfb5a6c2aeb68` |
| normalization/receipt.json | `05f1b46c7c07389313121589c66ddbc08abd831889221545df08c7f4bbb799b5` |
| named-normalization/receipt.json | `777ab38a4663db6721b3bfb7c712428031be9927749fa27d40e705cb2613138b` |
| baseline/receipt.json | `19ab83ab75e42fbe680270a3b74a1c92d0b83559ff46f6f8f941897284246fde` |
| full-gate.json | `dd5eb2af75be643d589822edfb64d1178662915a045dc26d9f543b30d52339d9` |
| focused-clean.json | `624936b515e9ea73f94f4a69d56073c46c131e81a2dadb799fa82713c06915b0` |
| guards-clean.json | `2d1642335b06abd0c169cbd432d7a2a01b8c68e2e919d51a697f9e83cf602133` |
| preflight-focused.json | `bdafa796c3718fe98ba5fd357a0e74cc7af4cd7b8d3fbefd4363323fa9bcbe4a` |
| incremental-clean.json | `7e3e3c8b2690fedec17a05ec782b2f7299b0a5c5123ee3dd23484d2904f3a333` |
| verified-summary.json | `9c33bb49812ddbb44592e364477b423348117f5e5d4adc8210a3ba45db867d0d` |
| coverage-summary.json | `44a6b2a6bebad366ad7371a5e0112d9c388a3bbc3eae7b8a4ea7104f34a3c46d` |
| coverage-rows.csv | `e6ef0c2c9bd023cf1788a0b0706116f5aae93077da8bfe13ecf33701f9fd3ae1` |
| snapshots/teamy-bend | `9a689f76242c0dc1e62fdfd46ec3d64e2ba28fc4878621c0346a071d7b635b9d` |
| snapshots/compare_normalization | `0e81208b3539cc75ec86019f2e6ccb76fefc49c9f0f27b5cef030e4eb34bbb79` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
