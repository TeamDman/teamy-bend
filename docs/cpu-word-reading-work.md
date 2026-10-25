# Packed subtraction closes the combined word-reader work gap

Clean tested source: `7bf8cec77c94866ff5ad467335526f19b7ccaa86`. The unchanged 16-result word-reader
aggregate now prints all 16 expected lines in native Rust, port C/JS and pinned
upstream C/JS, twice per lane. Native execution previously stopped after five
lines at the unchanged **2,000,000-step** budget. No resource exception was added.

The production change adds `U32.sub` to the existing checked, packed-only ordinary
arithmetic optimization. It uses wrapping 32-bit subtraction when both operands
are already sealed packed words; either ordinary constructor retains the existing
stored-body fallback. Origin/signature checks, strict executable demand, lazy
constructor APIs, matching order, work charging, GC, depth limits and cancellation
implementations are unchanged. There is no new public API or dependency change.

## Evidence for the correction

Pinned [U32.read.go](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L2070)
subtracts 48 from each character before adding it to the accumulator. Pinned
[U32.sub](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1278)
wraps [Word.sub](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1170),
whose fixed-width complement-and-carry implementation establishes wrapping subtraction.
The ordinary body constructs a structural Word. In the native runtime this made
subsequent operations fall back despite the digit subtraction starting with packed
operands. The measured correction preserves that representation through the chain.

| Same source observation | Before charges | After charges | Result |
| --- | ---: | ---: | --- |
| Leading-zero maximum alone | 960,394 | 212,143 | Both exact |
| Five-result prefix | 1,807,765 | 526,091 | Both exact |
| Six-result prefix | 2,000,000 | 598,041 | Before refuses; after exact |
| Unchanged sixteen-result aggregate | 2,000,000 | 739,206 | Before refuses; after exact |

Before the full aggregate refused, it entered 109 `Add` and 212 `Shl` ordinary
fallbacks. After, it enters zero ordinary numeric fallbacks, with 85 available
packed `Sub` results, 146 packed `Add` results, and 146 each packed/available `Shl`
results. Successful GC charges decrease from 454,936 before refusal to 150,169
for the completed run. Subtraction was not an optimized operation before, so the
old operation counters do not directly count its ordinary body invocations.

This was real cumulative work, not established double charging. Full-run totals
have different completion boundaries and are not equal-work timing benchmarks.
Temporary counters ran after unchanged successful cancellation/decrement checks;
all 40 runs across 20 cases (16 individual inputs and prefixes 4/5/6/16, twice) were deterministic
at every output boundary. The counters and `track_caller` annotations were removed
before clean validation. [Exact counters/source hashes](cpu-word-reading-work-measurements.json)
and [reproduction instrumentation](../scripts/word-reading-work/README.md) are retained.
The initial after-source line map incorrectly copied numeric tick line 516;
correct line 520 and the prior metadata hash are preserved. No observed charge
used that site and no count changed.

## Original executions and controls

[Per-case evidence](cpu-word-reading-work-evidence.csv) covers nine programs:
the unchanged upstream `base/read_bounds` and `base/show_read` originals, the
unchanged word aggregate, three earlier boundary/helper/file companions, a new
subtraction companion, and two new negative companions. Originals retain their
Apache-2.0 attribution. New controls are MPL-2.0.

* The aggregate, show_read, numeric/helper controls and subtraction edges match
  all five executable lanes. Subtraction includes zero/MAX/high-bit wraparound,
  ordinary reconstructed operands on either side, and partial application.
* Nat overflow and explicit private-request refusal both print only `before\n`,
  exit 1, and report the exact pinned diagnostic in all five lanes. The private
  request uses explicit constructors, avoiding the separate C wildcard inconsistency.
* Original read_bounds and the file-error companion still fail upstream JS at
  unavailable `bun:ffi`; successful corresponding native/C/port-JS lanes do not
  turn those blocked cells into matches.
* All 18 direct-upstream Node invocations fail in the known `require` host path.
  Generated execution is independently measured. No Bun installation was attempted.
* Strict check/eval is separately refused: 32 observations for unavailable IO/File,
  four private-request observations at load because `Emit` is unavailable. The 54
  phase probes distinguish this load refusal from the other strict checking refusals
  and accepted executable checking. These are not runtime passes or new exceptions.

The final matrix contains **198 observations: 66 exact successful executions,
20 exact intentional-failure executions, 4 upstream-JS host failures, 18 direct-Node
host failures, 36 strict proof refusals, 36 generations and 18 C builds**. Runtime
results are repeated twice under a 30-second process timeout; no timeouts occurred.
Native isolation independently passes all 40 observations. The first matrix and
phase attempts stopped on an overly narrow harness assumption that every strict
refusal occurs during checking; those incomplete receipts remain separate.
[Compact streams, phases and isolation evidence](cpu-word-reading-work-results.json).

## Regression validation and remaining failures

| Check at clean source above | Result |
| --- | --- |
| Standard repository gate | **1,173 passed, 0 failed, 49 ignored** |
| Format, production strict Clippy, all-feature build, Node provider | Pass |
| Strict Clippy for changed integration targets | Pass |
| Word-reader matrix / native isolation / phase probes | 198 / 40 / 54 observations, completed |
| Six original runtime negatives | 44 exact observations; 12 required observations still open |
| Ten demand controls | 92 exact; 6 approved C wildcard exceptions; 2 unresolved C diagnostic-order observations |

New tests cover lazy fallback (including packed-first/ordinary-second), left/outer
error ordering, strict live costly tails, zero-budget checks, cancellation-before-
charge, a million-iteration subtraction loop that still exhausts the work budget,
GC-at-every-safe-point fallback, checked Base origin/signatures and strict proof
exclusion. Existing aggregate sharing, source stack, depth, work and cancellation
regressions ran in the full gate. This does not imply mid-normalization cancellation
exists. Initial new-test compilation used `unwrap_err` on a non-Debug value; it was
corrected without changing production traits. That preliminary failed log remains.

Both renewed negative/demand runners return exit 1 intentionally to preserve open
parity requirements. The six-negative suite retains port-C array-depth safety
refusal, three stack-fault safety refusals (native/port C/port JS), native foreign
scalar support unavailable, and upstream-JS stderr/file host failure, twice each.
The JS-only scalar companion's two C lanes are inapplicable and unrun. The demand
suite retains three approved C wildcard cases separately from the unwaived C
`nat_first` diagnostic-order disagreement. All children exited, with no timeouts.
[Exact residual streams and classifications](cpu-word-reading-work-residuals.json).

## Reproduction and preservation

Use the already provisioned frozen tools. Host Rust build overrides were
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`,
`CARGO_BUILD_JOBS=4`, `CARGO_NET_OFFLINE=true`, `BEND_GPU=off`; `RUST_MIN_STACK` was
unset. These are Rust host build settings, not a Bend optimization contract.

```sh
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy -p teamy-bend --locked --all-features \
  --test cpu_read_bounds --test runtime_packed_words -- -D warnings
cargo test --locked --features extended_observability --test cpu_read_bounds
cargo test --locked --features extended_observability --lib runtime::numeric::tests
cargo test --locked --features extended_observability --test runtime_packed_words
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 \
  -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/negatives" -OracleOptimization O3 -Suite negatives
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 \
  -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/demand" -OracleOptimization O3 -Suite demand
```

For each fixture, the matrix invokes `teamy-bend run`, strict `check`/`eval`,
`node "$REFERENCE/bend2/main.ts" "$FIXTURE"`, and both executable generators.
C oracle compilation uses the actual pinned CLI profile:

```sh
node "$REFERENCE/bend2/main.ts" "$FIXTURE" -o "$ARTIFACTS/upstream.c"
"$TEAMY_BEND_CC" -std=c11 -O3 "$ARTIFACTS/upstream.c" -lpthread -lm -o "$ARTIFACTS/upstream"
"$ARTIFACTS/upstream" --threads 1 --gpu off
teamy-bend compile "$FIXTURE" --executable --target c --output "$ARTIFACTS/port.c"
"$TEAMY_BEND_CC" -std=c11 -O3 -Wall -Wextra -Werror -pedantic \
  "$ARTIFACTS/port.c" -pthread -lm -ldl -o "$ARTIFACTS/port"
"$ARTIFACTS/port" --threads 1 --gpu off
node "$REFERENCE/bend2/main.ts" "$FIXTURE" -o "$ARTIFACTS/upstream.js"
teamy-bend compile "$FIXTURE" --executable --target javascript --output "$ARTIFACTS/port.js"
node "$ARTIFACTS/upstream.js"
node "$ARTIFACTS/port.js"
```

Compare `.stdout`, optional `.stderr` and `.exit` exactly, twice; use the classified
host/proof cells above instead of treating them as matches. The integration test
bounds generated children to 20 seconds; the external matrix uses 30 seconds;
existing negative/demand children use 60 seconds and retain actual cleanup status.
All commands/source and raw-log hashes, tool pins and host profiles are in the
[validation receipt](cpu-word-reading-work-validation.json). Independent read-only
review found no blocker; fresh upstream execution subsequently confirmed its
pending companion comparisons. No reviewer test execution is claimed.

Build preflight requires 11 GiB workspace headroom and stops below 10 GiB workspace
or 1 GiB temporary space. This gate's measured minimum was
11.80 GiB workspace / 4.99 GiB temporary space. Existing inactive
build-cache relocation was retained; no cache or unique evidence was deleted this
turn. Private raw logs, full command receipts, generated executables and compressed
binary snapshots remain outside Git; their hashes are recorded. The small exact
failure receipts, expected streams, measurement records and reproduction code
are committed. Git publication does not preserve the bulky local artifacts.

Coverage remains **424 linked / 878 unlinked**; the latter contains 523 compiled
candidates, 341 checker-negative fixtures, 13 interpreter profiles and one
checking-only module. A linked receipt is not complete parity. The full checking
census was not rerun: the last measurement at `d224e79` retains 300 differences
outside 385 historical IO exclusions and six upstream stack failures. Runtime
production changed here; do not claim whole production-tree equivalence to that
census. Checker/syntax are unchanged, and the 54 current phase probes do not
redate the historical census.

The next substantive unresolved original remains `parse/string_pattern_sugar`:
production load still refuses at depth 64; [compact lowering](cpu-compact-patterns.md)
is unadopted research and does not close it. Other previously unwaived F32.read,
erased-hole/TODO, foreign/host, resource and diagnostic-order gaps remain. Historical
supplementary all-target Clippy failures were not rerun or cleared. Hardware and
profiler ignores remain unrun; no desktop/GPU work, dependency adoption, toolchain
change, guard change, new exception, main merge or release occurred.
