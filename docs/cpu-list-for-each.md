# List.for_each execution and proof boundaries

Tested source: `2dacf522eba2665652036b7b6ed369a2ba2c16d3`; the following checkpoint changes reports only.
**List.for_each already executes correctly. This tests-only checkpoint renews and
links coverage; it does not claim a new runtime repair or full CPU parity.**
Upstream stays `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

## Diagnosis and scope

The original `base/list_fold.bend` accepts in upstream checking. The port's strict
loader rejects its `List.for_each(~...)` call because its explicitly limited pure
Base never registers that execution-only template. The [loader](../src/syntax/parser.rs)
loads `executable-base.bend` only for executable checking. Its List.for_each body
is byte-identical to [pinned upstream](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1010),
using IO.pure/IO.bind; extracted body SHA-256:
`578ee0b0da11ae99817326da3ed48377ebfdf178f06901879b350786a2424e4a`.

The captured predecessor binary at `ead74b8db6e720be3047a434685cbaf55dc69dac`
already executes the unchanged original with the full expected output. Earlier
milestone 5.21 and the [historical checklist](cpu-parity-checklist.md) already record
one five-runtime probe. That evidence lacked a current per-fixture ledger link.
Existing tests cover generated-JS sequential/empty/halt behavior and JS lowering;
the new tests extend current evidence to native/C, nested calls, invocation
isolation and negative boundaries. Independent review agreed that moving effects
into proof Base would exceed scope. **No production, dependency or policy change
was made.** The strict acceptance disagreement stays measured and unwaived.

## Results

The unchanged original matches all five executable lanes twice. Across it and
seven authored positives, **76 of 80 runtime observations match exactly**; four
upstream-JS attempts for sleep-bearing nested/sequential callbacks fail because
Node lacks `bun:ffi`. Those lanes remain unqualified, with no installation or
runtime substitution. The pinned [sleep effect](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/effs/sleep.js#L4)
advertises a timer requirement, and the generated
[system provider](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L6070)
loads Bun's FFI when used. This is evidence for these two Node runs, not a claim
that all generated JS requires Bun. The original covers filtering, left/right folds, predicates, searches, sort and IO
iteration; its source SHA-256 is `99d0d1ee86621a581040e188ad1e7d7aea37a26a546e35b1fbf3e94f73d6dfa3`.
Two pure companions additionally pass strict checking and complete canonical
value-graph comparisons, with results `[1n, 2n, 3n]` and
`(16n, [1n, 2n, 3n, 9n])`. Graph capture and port graph assertions remain distinct.

The IO cases cover empty, sequential, direct and nested callbacks. The halt case
prints `first\n`, then exits **7** with `stop\n`; the third iteration's
`unreachable` output never appears. This nonzero status is a matching expected
contract, not a failed comparison. Reusing each checked book twice tests invocation
isolation; it does not establish allocator reclamation or mid-normalization
cancellation. Existing host-boundary cancellation/work/sharing/stack controls pass.
No exact elapsed-time scheduling assertion was introduced.

Six executable negatives reject false proofs, holes, callback element/result
types, missing markers and missing arguments, without output artifacts. False
proof and hole diagnostics match upstream exactly. Four diagnostics remain
unqualified: missing-argument/result embedded-Base offsets, missing-marker syntax,
and wrong-element loss of structured source context. The strict loader refuses
these effect-template calls earlier; that is separately recorded, not an exact
executable diagnostic comparison.

The direct upstream Node interpreter runs both pure cases and the empty IO case.
Five other IO cases hit `TypeError: require is not a function` (ten repeated host
failures). Generated upstream C matches all expected outcomes; generated upstream
JS matches the original and all companions except the two sleep-bearing host-limited
cases. Port native/C/JS matches every positive's expected output/status. The first matrix stopped on the newly
observed Bun failure; its incomplete receipt and script are retained. Resumption
reused completed observations only after checking identical commands/source hashes,
recorded the host failure explicitly and ran the remaining observations, including
the second failure repeat. No failed lane became a match. Port strict check/eval
retain load refusals for all six IO-positive cases; no new IO exclusion is subtracted
from the census.

| Validation | Result |
| --- | --- |
| Standard gate, clean tested source | **1,159 passed / 0 failed / 49 ignored** |
| Format, strict production Clippy, all-feature build, Node provider | Pass |
| Focused tests including prior sharing/IO/cancellation/template/proof controls | 43 passed |
| Focused new-test strict Clippy | Pass |
| Differential matrix | 288 observations; deterministic repeats; zero timeouts |
| Separate proof/executable phase probes | 84 observations; deterministic repeats |
| Full checking census | 1,302 classifications/statuses/streams unchanged; one oracle-stage variation |

Observation categories: `{"build_only": 16, "exact_executable_rejection": 28, "exact_execution": 76, "exact_interpreter": 6, "full_graph_oracle_capture_port_graph_asserted_in_tests": 4, "generation_only": 32, "strict_check_or_value": 8, "strict_effect_template_unavailable": 48, "unresolved_Node_host_failure": 10, "unresolved_executable_diagnostic_difference": 56, "unresolved_upstream_JS_Bun_host_failure": 4}`.
Six rejected inputs have no generated runnable artifact; execution is unrun for
those lanes. The two generated-JS host-limited programs were attempted and failed;
they are not unrun commands or qualified executions. All 49 standard ignores remain unrun. Historical supplementary
all-target Clippy failures were not rerun or cleared. No desktop/GPU checks ran.
The focused runs used the candidate tests; the clean-source full gate renews them.
Initial authored syntax/quantity/name mistakes and a test-only i32/u32 compile
error are retained privately as failed preparation attempts, not Bend defects or
passing qualification. No limits or stack settings were raised to address them.

## Census and acceptance

Counts: `{"acceptance_disagreement": 494, "both_accept": 366, "different_rejection": 191, "exact_rejection": 245, "oracle_or_infrastructure_failure": 6}`. All 1,302 fixture hashes,
port phases, statuses and raw output streams match the previous measured source.
Upstream statuses/streams and classifications are unchanged, but the known stack
failure `reg/dead_param_fork.bend` moved from the load to check stage. This is an
observed oracle-stage variation, not a port regression or newly qualified case.
The public census retains both stages; provenance/timing metadata belongs to this
new receipt. There are still **300
differences outside 385 historical IO exclusions, plus six upstream stack
failures**. This includes base/list_fold's deliberate strict availability boundary;
execution agreement does not waive it. Historical baseline 302 classifications
remain 151 diagnostic candidates / 109 acceptance / 42 other, with current
residual categories 149 / 109 / 42. Prior contracts, resource policies, unavailable
Bun/foreign hosts, printer failures, F32.read/error-order/TODO differences and
external API caveats remain unchanged.

The new original-fixture runtime link yields **412 linked / 890 unlinked**:
341 checking-negative, one checking-only, 13 interpreter-profile and 535 compiled
candidates. Links include historical, partial and blocked receipts; they are not
universal parity. The [acceptance criterion](cpu-parity-acceptance.md) is unchanged.
Next independent substantive family: `base/list_sort.bend` high-bit/empty radix
sorting and `run/list_map_fold_ops.bend` list/Map recursion, after actual runtime
applicability checks. Neither is qualified by this report.

## Reproduction and preservation

Frozen Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and
PS 7.5.4 remain. Temporary Rust host profiles are separate from Bend's C O3 runtime
profile and unchanged evaluator budgets:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --offline --features extended_observability --test cpu_list_for_each --test runtime_effects --test executable_check --test parser_templates --test kernel_open_closures
cargo clippy --locked --offline --features extended_observability --test cpu_list_for_each -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

With `REF` pointing to the unchanged upstream checkout and `OUT` to fresh scratch
storage, use the original or any authored positive as `CASE`:

```sh
CASE=tests/fixtures/cpu-list-for-each/original/list_fold.bend
node "$REF/bend2/main.ts" "$CASE"
target/debug/teamy-bend check "$CASE"
target/debug/teamy-bend run "$CASE"
node "$REF/bend2/main.ts" "$CASE" -o "$OUT/upstream.c"
node "$REF/bend2/main.ts" "$CASE" -o "$OUT/upstream.js"
target/debug/teamy-bend compile "$CASE" --executable --target c --output "$OUT/port.c"
target/debug/teamy-bend compile "$CASE" --executable --target javascript --output "$OUT/port.js"
"$TEAMY_BEND_CC" -std=c11 -O3 "$OUT/upstream.c" -lpthread -lm -o "$OUT/upstream"
"$TEAMY_BEND_CC" -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$OUT/port.c" -pthread -lm -ldl -o "$OUT/port"
"$OUT/upstream" --threads 1 --gpu off
"$OUT/port" --threads 1 --gpu off
node "$OUT/upstream.js"
node "$OUT/port.js"
```

Each matrix generation/build/execution is bounded at 30 seconds. Runtime and
checking observations repeat twice; unoptimized strict integration C is separate.
The census uses the [committed adapter](../scripts/checking-census/README.md) and
30/60-second process limits; the additional executable phase probe calls
load_executable/check_executable without running main. No dependency was installed.

Binary SHA-256 `8320c64076730199d255047d9d5d1dd4ac99bfcd72dc0186ce42621ef5d6fd4f`.
Matrix receipt `8212258224bff1e0fbb90bded7fbd1e83dfb200a61c09c390ed53118370d6dad`.
Census receipt `51767cee2ac0421d72e976b246ed3afd3447df2716b278a9e35beb96a6b65a89`.
Cargo.lock `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` and docs pin
`50e89d6897ce2a65167ef65b17c5fa83fe849158` are unchanged; reference trees stay clean.

The gate started with 11.32 GiB free and stayed above
10.18 GiB (11 GiB preflight/10 GiB stop floor; 1 GiB temporary
floor). Rebuildable test executables caused the growth. After completion, active
process/inode/hash checks retired only inactive caches, restoring
11.33 GiB. Source and unique evidence remain intact.

The reviewed feature checkpoint preserves source fixtures, expected values,
[baseline observations](../tests/fixtures/cpu-list-for-each/baseline.json),
[compact nonzero outcomes](cpu-list-for-each-failures.json),
[per-case records](cpu-list-for-each-evidence.csv) and
[current census](cpu-list-for-each-census.json). Bulk raw logs, complete observations,
generated programs and binary captures remain outside Git in the task evidence
store; the push does not back up those unique artifacts. No merge, force push,
package publication or release occurred.
