# Native radix sort and list/map execution

Tested source: `d224e79d33b4a0ab26f701b57f448056b63b790a`; the following checkpoint changes reports only.
**The unchanged radix-sort original now completes within the unchanged native
work budget. The list/map-fold original already passed and is newly qualified.**
Full CPU parity remains incomplete. Upstream is frozen at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

## Diagnosis and correction

The pinned [radix original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/base/list_sort.bend)
implements 32 binary passes locally; it is not bundled List.sort. Every pass
shifts each U32 by a Nat count, masks the low bit and preserves list order within
each partition. Native execution previously exhausted its two-million-step budget
with no stdout, while upstream C O3/JS and port C/JS printed `6\n` (exit 0,
empty stderr). Even the reduced two-element sort exhausted native work.

The only production change adds sealed, checked `U32.shr` and `U32.and` to the
existing packed-word optimization path in [numeric.rs](../src/runtime/numeric.rs).
[The pinned library](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1293)
defines shr as a logical one-bit shift and and as bitwise conjunction of Word
fields. The extracted ordinary shr/and/shrn bodies are byte-identical between
the port and upstream; their SHA-256 identities are retained in the compact receipt.
Shr alone lets each reduced sort complete but leaves the original
aggregate over budget. Adding And lets the original finish. No incorrect step/GC
accounting was established; this closes missing packed operation support rather
than changing resource policy. No charges, budgets, stack settings or cancellation
boundaries changed. In particular, shrn keeps ordinary Nat recursion and saturates
to zero after 32 shifts; it is not replaced by a wrapping host shift count.

Only already packed values take the shortcut. Ordinary constructor arguments use
the original checked bodies and thunk identities. No zero-operand shortcut or
forced Word decoding was added. Pure fallback tests show that shr can discard a
failing low bit, and `and(0, ordinary-failing-bit-word)` can ignore that bit;
reversed operands expose the failure. This internal lazy evaluator control is
separate from executable by-value policy, which still forces live fields. Strict
IO tests retain first-error/output ordering, work/depth refusal and cancellation.
Origin/signature tests deny same-named user definitions the optimization; mixed
representations, partial application and GC fallback remain covered.

The clean-source instrumented IO run records 1,926,203 cancellation polls for the
full radix original, and 85,294 for the list/map original. Poll counts include
entry/boundary checks, so are not presented as exact charged-transition totals.
This radix workload has limited remaining budget headroom; larger inputs are not
qualified by it. No stronger scalability or mid-normalization cancellation claim
is made. Independent static review found no blocking findings. Its only wording
correction (completion rather than halt) was applied before the source commit.

## Results and limitations

| Program | Before native | Current native / port C / port JS / upstream C / upstream JS |
| --- | --- | --- |
| unchanged base/list_sort | exit 1, work-budget error, empty stdout | exit 0, `6\n`, empty stderr in all five lanes twice |
| unchanged run/list_map_fold_ops | exit 0, `1111111\n` | same exact result in all five lanes twice |
| authored word boundaries | not previously qualified | 12 exact output lines in all five lanes twice |

The [map/fold original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/list_map_fold_ops.bend)
covers list mapping/filtering/folding/predicates and structural Map folding with
string-key lengths. The boundary companion covers zero/high/max words, disjoint
masks, 31/32/33 shifts, copied constructors and partial application.
**30/30 runtime observations match**, within 66 total matrix observations.
All deterministic repeats agree; no process timeout. Six direct upstream Node
attempts still fail with `TypeError: require is not a function`; those attempted
host failures remain unqualified. Generated JS succeeds here with the installed
Node host. Twelve strict check/eval attempts still reject IO. Separate 18 phase
probes show upstream/executable checking accepts all three inputs and port strict
checking rejects at the check phase, after loading. These are deliberate proof
scope exclusions, not production failures and not new census waivers. No pure
normalization/graph claim is made for these IO entries.

Ten corrected reductions are compared before/after (20 observations). Before:
empty/singleton, one-pass, shift and mask controls pass; two/sorted/unsorted/highbit
full sorts exhaust work. After: all ten pass with their exact expected values.
Two early one-pass probes omitted partition accumulator arguments and were
checker-rejected; their original failed preparation receipts remain local. The
corrected probes explicitly supply Nil accumulators. A phase harness initially
expected load refusal instead of check refusal; it was corrected without changing
production. A snapshot helper refused an already existing historical directory
before writing; it was corrected to a fresh directory. No old snapshot was
replaced and no failed attempt is counted as a pass.

| Validation | Result |
| --- | --- |
| Clean-source standard gate | **1,162 passed / 0 failed / 49 ignored** |
| Format, production strict Clippy, all-feature build, Node provider | Pass |
| Focused numeric unit controls | 16 passed |
| Focused original/runtime/template/aggregate controls | 59 passed |
| Strict Clippy for changed library and integration tests | Pass |
| Differential matrix / phase probes | 66 / 18 observations; no timeouts |
| Reduced before/after executions | 20; original historical failures retained |
| Full checking census | 1,302 port statuses/phases/raw streams unchanged |

The full gate renews default-host-stack, source-hole, false-proof, macro/shared
reference, composition-sharing, work/depth and cancellation regressions. All 49
ignored hardware/profiler checks remain unrun. Historical supplementary all-target
Clippy failures were not rerun or cleared; the focused lint result is scoped.
No desktop/GPU, dependency/toolchain, API/CLI/logging or upstream source changes.

## Census and acceptance

Current census counts: `{"acceptance_disagreement": 494, "both_accept": 366, "different_rejection": 191, "exact_rejection": 245, "oracle_or_infrastructure_failure": 6}`.
There remain **300 differences beyond 385 historical IO exclusions**, plus **six
upstream stack failures**. The historical 302-row triage baseline remains
151 diagnostic candidates / 109 acceptance / 42 other; current residual categories remain
149 diagnostic candidates / 109 acceptance / 42 other. No acceptance/rejection or diagnostic policy changes were made.
Upstream phase changes in this renewal: `[]`.
See [census receipt](cpu-list-sort-map-census.json). Runtime qualification does not
waive strict IO disagreements or unavailable oracles.

Two original runtime links yield **414 linked / 888 unlinked**. Remaining unlinked
applicability categories: 533 interpreter-and-compiled candidates, 341 checker
negatives, 13 interpreter-profile cases and one checking-only module. Links are
not universal parity; [the acceptance criterion](cpu-parity-acceptance.md) still
requires lane review, exact positive/negative contracts, API/interface coverage,
resource controls and explicit closure of unresolved differences. F32.read,
error-order, erased-hole/TODO, foreign/host/printer failures, resource-policy cells,
external API compatibility caveats and the unfinished 688-vector exploration
remain unchanged. Next independent substantive batch selected from the ledger:
`run/map_set_ops` and `run/list_string_lookups`; their IO mains are source-inspected,
but their executable lanes are not qualified here.

## Reproduction and preservation

Activate the provisioned environment, then from the repository use:

```bash
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib runtime::numeric::tests
cargo test --locked --features extended_observability --test cpu_list_sort_map --test runtime_packed_words --test kernel_open_closures --test cpu_list_for_each --test parser_templates --test runtime_numeric
cargo clippy -p teamy-bend --locked --all-features --lib --test cpu_list_sort_map --test runtime_packed_words -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The temporary Cargo debug/incremental overrides control host build disk use only;
Bend executable semantics and C O3 qualification remain separate. Default host
stack, Rust 1.96.0, fmt nightly-2026-05-24, Node 24.19 and Clang 19.1.7 are retained.
For each original and companion set `fixture` to its checked-in path, `reference`
to the unchanged upstream checkout, and `artifacts` to a fresh writable directory:

```bash
teamy-bend run "$fixture"
teamy-bend check "$fixture"
teamy-bend eval "$fixture"
node "$reference/bend2/main.ts" "$fixture"
node "$reference/bend2/main.ts" "$fixture" -o "$artifacts/upstream.c"
node "$reference/bend2/main.ts" "$fixture" -o "$artifacts/upstream.js"
"$TEAMY_BEND_CC" -std=c11 -O3 "$artifacts/upstream.c" -lpthread -lm -o "$artifacts/upstream"
teamy-bend compile "$fixture" --executable --target c --output "$artifacts/port.c"
teamy-bend compile "$fixture" --executable --target javascript --output "$artifacts/port.js"
"$TEAMY_BEND_CC" -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$artifacts/port.c" -pthread -lm -ldl -o "$artifacts/port"
"$artifacts/upstream" --threads 1 --gpu off
"$artifacts/port" --threads 1 --gpu off
node "$artifacts/upstream.js"
node "$artifacts/port.js"
```

Each matrix process has a 30-second process-group timeout; executable observations
repeat twice. Rust integration C tests retain their separate strict unoptimized
compiler profile and 20-second process bound. Phase probes perform no execution.
Checking census uses the existing `scripts/checking-census/run.py` with explicit
reference, captured binary, phase-probe, output and build-receipt arguments.
Binary SHA-256: `de7a8ff20b4d7208c57e3f709e8dbbe05a1de4082ff01dac6230d3670da21f0b`. Cargo.lock:
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Mounted docs pin: `50e89d6897ce2a65167ef65b17c5fa83fe849158`.

Disk preflight began at 11.199 GiB; monitored minimum was
10.079 GiB, above the 10 GiB stop floor. Generated test
executables caused the growth; inactive default-only/all-feature CLI caches were
reclaimed mid-gate after confirming no active process used them. Completed-gate
inactive test caches were then reclaimed, leaving 11.254
GiB. No source, snapshot or unique evidence was deleted. Future large builds need
fresh preflight and a larger starting margin if this cache footprint repeats.

[Compact pre-fix/current failures](cpu-list-sort-map-failures.json),
[per-case evidence](cpu-list-sort-map-evidence.csv),
[validation commands and hashes](cpu-list-sort-map-validation.json), fixture originals and tests are
in Git. Private task-folder raw commands/streams, full logs, generated artifacts,
phase/census outputs and binary snapshots are **outside Git**; checkpoint pushing
does not preserve those bulk artifacts. Their hashes are retained in compact
receipts. Publication uses the isolated reviewed feature branch, normal push and
remote-SHA verification; no main merge or release.
