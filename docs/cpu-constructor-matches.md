# Simple, single and nested constructor qualification

Tested source: `0b041df0fa8de2a50787026082ee59afa253070b`. Starting checkpoint:
`ace3c51c0f350fde8fb8d8808b82b7f3cc1d3661`. Target:
[pinned Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Production code, manifests, lockfile and toolchains are unchanged in this batch.

## Applicability before execution

The five [originals](../tests/fixtures/cpu-constructor-matches) are byte-for-byte
Apache-2.0 copies of pinned `tests/flatten/`: `simple_ctor_000`,
`simple_ctor_001`, `single_ctor_type_001`, `single_ctor_type_001_mk` and
`nested_literal`. New adapters/wrappers/tests use MPL-2.0; derived oracle data
retains pinned-source attribution.

All five pure mains normalize to functions. Neither fixture comments nor the
absence of a runtime artifact was used to guess applicability. The initial
65-observation preflight explicitly checked normalization and actual compilation:
originals lack Base, so upstream C/JS and port C/JS/native all exit 1 with
empty stdout, exactly `Error: a build needs import Base\n`, and no artifact.
Separate Base-importing adapters then reach exact function-printer refusal.
Only after those observations was the application grid executed.

The final original collection repeats raw normalization, strict-eval report
text, actual graph comparisons and all refusals separately. This does not claim
that original functions were run as printable compiled values. Source types and
checking phases are recorded as observations, not blanket exact diagnostic parity.

## Bounded behavior and separate observation paths

| Original | Inputs | Behavior exercised |
| --- | ---: | --- |
| `simple_ctor_000` | 4 | Z versus S; retain the predecessor field in B |
| `simple_ctor_001` | 5 | None and Some at depths 0, 1, 2, 4; preserve the optional value |
| `single_ctor_type_001` | 4 | Eliminate Only and return one successor of its field |
| `single_ctor_type_001_mk` | 4 | Construct Only while retaining its complete field |
| `nested_literal` | 16 | Env cross-product of Stop/Num(0)/Num(1)/Num(3); literal priority, later Num arm and both Stop fallbacks |

[Kernel controls](../tests/kernel_constructor_matches.rs) compare all **33 direct
applications** of the original mains with full constructor text and structure.
The original graphs use alpha-canonical binder indices and explicit ordered
children, match arms/fallbacks and constructor names. This is structural/binding
comparison, not a claim of physical Rc identity; retained sharing controls test
allocation sharing separately.

Two authored wrappers per original reuse those same inputs:

- `values/` returns all full imported constructor values, preserving payloads and
  qualified names. Native pure execution and generated C/JS compare exact output.
- `wrappers/` projects each result to a small scalar list. These additional
  observations permit generated upstream JS to qualify branch behavior without
  pretending its broken imported-constructor printer succeeded.

Both sets have independently captured pinned normalized graphs and exact text.
The [compiler regressions](../tests/compiler_constructor_matches.rs) run all ten
mains through native Pure, strict C and Node JS, and preserve refusal status,
stderr and absent artifacts. Strict proof normalization, pure CLI output and
compiled execution remain separately recorded.

## Harness failure and independent review

The first focused run at `5a210c34a47e8ec522d3e8d7726d818e6c2b6514` exits 101:
a new wrapper graph test aborts with stack overflow. A preserved instrumented
probe loads and checks the first wrapper, reaches the graph-decoding marker,
then aborts before its evaluation marker. The failing operation is recursive
`facet_json::from_str::<Shape>` over the approximately 15 KiB Word/List oracle.
The separate native/C/JS wrapper tests had already passed. This localizes the
observed failure to the test-oracle decoder, not Bend reduction or a Bend guard.
It does not establish a defect in every Facet decoding use.

The test-only repair serializes the complete existing bounded `Shape` and
compares it exactly with canonical upstream JSON, removing one terminal newline
only. It drops no node, field, constructor name, order or binding relationship;
all typed shallow-original and direct-application structural checks remain.
The serialized comparison can reject differently formatted equivalent JSON,
but cannot silently accept a different represented graph. Shape's 128-depth,
unbound-variable and unsupported-variant checks are unchanged. No host stack
size, Bend budget, evaluator, dependency or cancellation policy was changed.

An independent read-only review of the exact tested source found no blocking
concern and confirmed preserved structural coverage. The reviewer did not run
tests. The failed focused run, diagnostic probe and dirty three-test candidate
probe retain their own identities; none is relabelled as the clean final gate.

## Results and reproducibility

| Check | Result on tested source |
| --- | --- |
| Standard gate | **1,003 / 0 / 49**, 129 result-bearing targets, 483.162 seconds |
| Final focused checks | **23 / 0 / 0**, 15.626 seconds |
| Original/applicability collection | **150 observations**, including 100 exact refusals with no artifacts |
| Graph collection | **60 exact structural observations**: 20 original and 40 wrapper/value, two implementations twice |
| Runtime collection | **180 observations**: 110 exact executions, 60 generation/build successes, 10 unresolved upstream-JS timeouts |
| Checking audit | **1,302 unchanged statuses and streams**; 439 expected negatives reject; other rows 366 accept / 497 reject |
| Additional controls | 14 positive passes; 18 observations preserve three hole/TODO disagreements |

The projected-output runtime submatrix has 60 exact executions and 30 successful
build/generation steps. The full-constructor submatrix has 50 exact executions,
30 successful build/generation steps and ten upstream-JS timeouts. All port
values match pinned interpretation and upstream C O3. The five generated
upstream-JS full-value programs have the previously diagnosed local runtime tag
versus qualified descriptor-name mismatch; each is killed and reaped at the
separate three-second diagnostic bound twice, with empty streams. Their generated
JS is retained. A kill is not a language failure status or exact oracle agreement.
The overall runtime runner exits 1 deliberately; no exception is adopted.

The prior [imported-printer correction](cpu-imported-printer.md) and its
12-timeout/72-aggregate external receipts remain tied to source `936d299`.
Those complete external matrices are not renewed here. This batch adds fresh
full-value observations and renews the imported-printer integration tests and
aggregate sharing/work/cancellation tests in the standard gate. The unchanged
27-result aggregate uses 1,338,629 steps; prefixes 23/24 use 634,793/861,655.
Limits remain 2,000,000 steps and depth 128. Cancellation still has only its
existing before/after-normalization/output boundaries, not mid-normalization
interruption. No general resource-equivalence claim follows.

Formatting, strict Clippy, all-feature build and native Node socket-provider
checks pass. Ignored/unrun: 47 GPU hardware tests and two optional profiler
tests. Expected compile-fail doctest diagnostics are passing rejection evidence.

Actual final commands after activating the frozen tools:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export RUST_TEST_NOCAPTURE=1
cargo test --locked --features extended_observability --test kernel_constructor_matches --test compiler_constructor_matches --test compiler_imported_printer --test executable_demand --test runtime_effects --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug/profile overrides do not alter Bend runtime optimization. External
comparisons use the captured final binary and two execution repetitions:

```sh
node "$BEND_REFERENCE/bend2/main.ts" "$fixture"
teamy-bend run "$fixture"
teamy-bend --output-format json eval "$fixture"
node "$BEND_REFERENCE/bend2/main.ts" "$fixture" -o upstream.c
teamy-bend compile "$fixture" --executable --target c --output port.c
clang-19 -std=c11 -O3 upstream.c -lpthread -lm -o upstream
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic port.c -pthread -lm -ldl -o port
./upstream --threads 1 --gpu off
./port --threads 1 --gpu off
node "$BEND_REFERENCE/bend2/main.ts" "$fixture" -o upstream.js
teamy-bend compile "$fixture" --executable --target javascript --output port.js
node upstream.js
node port.js
teamy-bend --output-format json check "$fixture"
```

C uses the pinned upstream CLI's O3 profile. Ordinary matrix invocations retain
30-second bounds; the known imported-JS printer loop has a separately recorded
three-second bound. Checking children retain 60-second limits and outer task
wrappers 1,200 seconds. Only task-owned timed-out processes are killed/reaped.

Rust 1.96.0, nightly-2026-05-24 fmt, Node 24.19.0, Clang 19.1.7 and PowerShell
7.5.4 remain frozen. Cargo.lock SHA-256:
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Binary SHA-256: `00a447464c33e45434787247e4d791dd308b115ccd6ee4cba129609ab2947ac4`.

## Storage and remaining scope

Build preflight requires 11 GiB total free, a 1 GiB margin above the 10 GiB
reserve. Full-gate minimum was **10.062 GiB**. Only verified
inactive first-party build caches are reclaimed, including completed failed-test
executables; their source, logs and unique failure evidence remain. The current
binary is retained in a verified working copy and lossless archive; the matching
library is retained until the graph probe completes. No dependency cache,
upstream/docs source or unique historical failure is deleted. An intermediate
post-gate cleanup stage asserted the comparison threshold before the remaining
test-cache cleanup; completing that planned cleanup restored 11.021 GiB before
comparisons. No threshold changed and no disk exhaustion occurred. The original
16.3 MB pretty graph receipt is preserved in a verified lossless archive; an
equivalent 0.50 MB compact JSON retains every observation and graph.

Tested source `0b041df0fa8de2a50787026082ee59afa253070b` adds five unchanged simple,
single-constructor and nested-constructor originals with a bounded **33-input**
grid. Original graphs/text, complete application results, scalar projections
and executable output are compared separately. No production change or new
port semantic mismatch was found. [Scope and receipts](cpu-constructor-matches.md).

The full gate passes **1,003 tests, zero failures, 49 ignored**; final focused
controls pass **23 / 0 / 0**. The **60 graph observations** and **150 original/
applicability observations** complete. Ten wrapper mains have **110 exact
executions and 60 successful generation/build steps**. The **ten upstream-JS
full-constructor printer timeouts remain unresolved**, so the complete runtime
matrix exits 1. All projected-output lanes agree. Port native/C/JS full values
match the pinned interpreter and C O3; this is not upstream-JS output parity.
Original Base-required and adapted function-printer refusals are exact and do
not become executable passes.

The initial new test aborted while recursively decoding its large graph oracle,
before Bend evaluation. The independently reviewed test-only repair compares
the entire canonical serialized graph, with no dropped nodes or structural
relaxation. Failed receipts remain tied to `5a210c3`; the clean final gate uses
this tested source. No host stack, Bend work/depth guard, evaluator or policy
change occurred. Aggregate sharing/work/demand/cancellation regressions pass in
the full gate; no mid-normalization cancellation is claimed. All 1,302 checking
statuses and streams remain unchanged; 439 expected negatives reject. Fourteen
positive controls pass; 18 observations retain three hole/TODO disagreements.

Detailed evidence now links **349 originals**, with **953** without detailed
receipts: **357 checker negatives, one checking-only module, 58 interpreter
cases and 537 compiled candidates**. Five interpretation rows are new; prior
rows retain their historical identities. Links include partial/blocked evidence,
not completed parity. Imported-JS printing, erased-hole/TODO, resource/host/
foreign gaps, F32.read, competing failure order, downstream API caveats and the
unfinished 688-vector exploration remain unwaived.

Next independent family: `flatten/literal_default_000..004`, covering literal
ladders, three-column defaults and reconstruction of the whole matched default
value. Establish applicability before a bounded **45-input** grid. These remain
unqualified. Frozen dependencies, toolchains, pins, budgets, guards and
cancellation; local checkpoints only, no pushes/merges/installations/desktop/GPU.
No new user decision was needed for this bounded qualification.

## Receipt identities

Private receipts retain exact commands/status/streams, source and binary hashes,
bounds, build profiles, generated-artifact hashes and timings. The
[public execution ledger](cpu-constructor-matches-evidence.csv) keeps observations
and classifications separate without machine paths.

| Receipt | SHA-256 |
| --- | --- |
| `applicability/receipt.json` | `0cc5cafb061aca9aedec8570b91c9e141b40f91471e9d3073e8abbacfa6dcfe2` |
| `application-preflight.json` | `9262951b2a064ea6f0a94f7e5adc93175e25a6ac162c702be6de1e380557a8ce` |
| `focused.json` | `71d70a77da0c6bad9b985253908954078008bba66caede8965cfac0e06357d6f` |
| `focused.log` | `7f9d03813addbb282d024e7206c325a3e1f327540f6059c434444611ad38aaa9` |
| `stack-probe.json` | `e0a7746705eaf1ac77ddc55cc75d289dd71bc88850996bbb2cfd3cd32ab20e4a` |
| `stack-probe.log` | `7ce6844458c7ba3c7f133b0b3aa2b2f24e6f867e7fb7686ef33526dd7ffa7b36` |
| `graph-encoding-probe.json` | `39ec2f35a8c768dcea7f1d4c0a56ea45074c6d80d7c05720e711bf4f0065a6fa` |
| `comparison-review.md` | `569025ba5f7d506e857bf1eed5b03c9768f3079cb3c83c7d5c55589a3b947a33` |
| `focused-final.json` | `4496f7daad7c50fc53197b7085177cba4dab2623f8d7f3f36491a7a15c928578` |
| `full-gate-final.json` | `cd2e2c54a142bc448c3a2dff41830df7c0d7bf77fe846dcf77b7d2a419c3af39` |
| `full-gate-final.log` | `6198651e20bfeb6cc759679ad11705806d442ce96470e90e7c25f9acc10c805a` |
| `snapshot.json` | `ddf14048d3a8bedde8c1187b76fd8e9d35e9037aa5834e14ae613fa36a32080d` |
| `open-matrix/receipt.json` | `176670a20f6a498d03f2ad4b19f3561b8e5bd5fe46c54a00d51c8674085be1ed` |
| `graphs-matching/receipt.json` | `69b2e1ec2193bfc1c29fdd5bcc5e5d207fe4479b2e856b2af36bb99a12f91675` |
| `matrix/receipt.json` | `62a24b002bf3af6d8af782d6d609e5437196405656a7e50c5cc2a6a47b1e5f02` |
| `matrix-run.json` | `83a22787c1c6b3d880dbc40a0b4d47523223e3a50b6156b6e8540e99042b77af` |
| `audit/receipt.json` | `c0c65b6925afee346ed857fe6a245e84bb6c797650543fddffb8b4cc64b40346` |
| `audit-delta.json` | `4f9801ed8f5035a3a9b617ea2218775497fee2b76507f523e86dc601ef04e7ed` |
| `controls.json` | `f0f123d1d1bda09fad7608f62adbdd2d09b7e7ce6b22adc2b0444ef51e8da612` |
| `coverage-summary.json` | `517d9e7ee1b3b6b6edee104a0b1069372053e53c2d92b85be5404e608f8e64d2` |
| `comparison-preflight.json` | `81de6f0f0515402cf130d3cb04844e5bedbef1721a7fd1fbf0c51e6115ca010a` |
| `graph-receipt-compaction.json` | `fa637d5ce78f9e5b46c069145848a9548a5ad3eed527a9b5442f0356be1a37c7` |
| `tool-versions.json` | `32e8c352aa70402d09982384c921cfb09d9400ed99956a24478b5952d4860a71` |
| `upstream-js-evidence.json` | `dda0f6659612e6290f23771f7159ba945371a36e1f3c784f5f10d5070635101a` |
