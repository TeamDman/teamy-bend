# Composition work accounting and demanded sharing

Tested source: `20fec64bebb656c3fea84896e177f6cda8a9e75d`. The preceding failure receipt remains at
`cbfe84fb6f76c17fcb3022fc86d3e62f01a3cafa`; instrumentation starts from the clean
checkpoint `2ad725bc07ede2320074199f94434700bcf29c67`. Reference:
[pinned Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Both [27-result wrappers](../tests/fixtures/cpu-open-closures/wrappers) and the
three attributed upstream originals remain byte-for-byte unchanged.

## Diagnosis and correction

The boundary was real accumulated reduction work, not a special charge for an
aggregate. The first 23 isolated calls consumed 1,644,880 steps; constructing and
normalizing their list added 48. Call 24 alone consumed 731,385 more. All 27
isolated calls totaled 3,863,368 steps, each receiving its own existing budget.
The original aggregate actually stopped at two million; it was not observed to
finish with that larger total.

There was avoidable work within those calls. Unrelated substitutions rebuilt
application nodes. Preserving those nodes, without yet caching any result,
exposed **465 repeated demands of the same `Word.shl` allocations** among 496
shift demands for `double(4294967295)`; the step count remained 725,483. The
pinned [multiplication](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L1239)
reuses shifted operands. Its [weak-head reducer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2810)
and [term cells](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L686)
explicitly share demanded work while retaining raw fields.

The scoped repair is in [term.rs](../src/kernel/term.rs),
[reduce.rs](../src/kernel/reduce.rs) and [check.rs](../src/kernel/check.rs):

- An application whose children are unchanged by substitution retains its Rc.
- During one normalization, successful application-to-constructor heads are
  memoized by allocation identity. Weak keys prevent address reuse. Input terms
  stay immutable; fields remain raw. No failures, blocked calls or functions
  are cached, and no new public term variant or serialization format is added.
- Aliases stay constant within that scope. The table is dropped on success or
  failure; separate calls cannot inherit stale alias results. Every demand still
  charges its existing step and checks the depth guard before a cache lookup.

This removes repeated reductions rather than discounting performed work. The
2,000,000-step and 128-level limits, input/structural/source-hole guards,
executable demand rules, IO behavior and cancellation boundaries are unchanged.
Pure-driver cancellation remains checked before normalization and before writing
its result; this change does not add mid-reduction cancellation.

| Observation | Previous charged work / outcome | Final charged work / outcome |
| --- | --- | --- |
| One `double(4294967295)` | 725,483 / pass | 220,958 / pass |
| 23-result prefix | 1,644,928 / pass | 634,793 / pass |
| 24-result prefix | 2,000,000 / exhausted | 861,655 / pass |
| Exact 27-result main | 2,000,000 / exhausted | 1,338,629 / pass |

The final isolated-call sum is 1,338,573; the aggregate adds 56 list-traversal
steps. These counters are captured from the final full-gate test log. This is
not a wall-clock benchmark: the four final full-aggregate CLI observations take
26.768–27.437 seconds, within the unchanged 30-second per-process bound. Timing
headroom is finite; no general performance or resource-parity claim follows.

## Validation and reproduction

The final standard gate passes **990 / 0 / 49** across 124 result-bearing targets,
in 463.44 seconds. Ignored and unrun: 47 GPU hardware tests and two
optional profiler tests. Format, strict Clippy, all-feature build and the native
Node socket-provider checks pass. Expected compile-fail doctest diagnostics in
the log are successful rejection tests.

The preliminary focused source `f9cf7a3bd59fbcee212646a8829ade51e26dd177` passes
**306 / 0 / 1**. Its first standard-gate attempt stops at Clippy's 101/100
function-length check; no later gate stage runs in that failed attempt. The
final source extracts the application mapper, adds an insufficient-total-budget
regression, and passes the full gate. The failed receipt is retained.

[Kernel regressions](../src/kernel/composition_work_tests.rs) cover individual,
23/24-prefix and unchanged 27-result normalization; actual substitution versus
unaffected sharing; lazy cached fields; charged cache hits; zero-work and depth
refusal; total work exhaustion; first-error order; cache cleanup and changed
aliases. [Execution regressions](../tests/compiler_open_closures.rs) now run both
unchanged aggregates natively as well as generated C/JS, and verify cancellation
leaves both writers empty. Existing named-call, source-hole, structural, demand,
writer and cancellation controls remain in the final full gate.

The [execution ledger](cpu-composition-work-evidence.csv) records:

- **72 runtime observations:** 46 exact status/stdout/stderr matches, 24 successful
  generation/build steps, two direct upstream IO host blocks. All available
  required observations pass; the host-blocked lane does not become a pass.
- **12 prefix observations:** counts 23, 24 and the byte-identical 27-result
  wrapper, two repetitions each in upstream interpretation and the port pure CLI.
- **12 graph observations:** two pure originals plus `partial_app_snf`, both
  implementations, twice. Binding structure and normalized graph shape are separate from closed output.
- **1,302 checking controls:** unchanged statuses and streams; 439 expected
  negatives reject, and the other rows split 366 accepted / 497 rejected. These
  are checking controls, not universal exact upstream diagnostic qualification.
- **14 positive controls** and **18 observations of three unresolved hole/TODO
  disagreements**. The latter remain unwaived.

The direct upstream IO lane still returns exit 1, empty stdout and exactly
`TypeError: require is not a function\n` under installed Node. The pinned
[host runner](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1274)
requires unavailable host behavior. Native IO and generated C/JS print `12\n`.
Strict proof IO exclusions remain separate. No Bun or other dependency was added.

After activating the frozen toolchains, the actual Cargo/standard-gate commands
use these temporary host build settings (not Bend runtime optimization flags):

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test kernel_open_closures --test compiler_open_closures --test cli_compile_executable --test kernel_wildcard_values --test kernel_normalization --test executable_demand --test runtime_effects --quiet
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The external matrices use the captured final binary, 30-second invocation
bounds, exact outputs and two execution repetitions. Representative commands:

```sh
node "$BEND_REFERENCE/bend2/main.ts" "$fixture"
teamy-bend run "$fixture"
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

C uses the pinned upstream CLI's O3 profile; no O0 result is substituted.
Rust 1.96.0, nightly-2026-05-24 formatting, Node 24.19.0, Clang 19.1.7 and
PowerShell 7.5.4 remain frozen. Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Binary SHA-256: `899618fdd20d3e49980cd6e43bcb5f7ec3ef75c2557a59b7a0d6ad34865b2564`.

## Storage and remaining scope

Build preflight requires 11 GiB total free, providing 1 GiB of headroom above
the 10 GiB reserve. The full gate's minimum
was 10.146 GiB. Completed integration/unit executables
and verified inactive alternate-feature first-party library caches were reclaimed;
all hard links were inventoried before the library removal. Two old zombie rustc
entries initially tripped the conservative activity check; their states were
verified and no unrelated process was killed. A comparison preflight refused
before building when free space was below 11 GiB, then passed after safe cache
recovery. No disk exhaustion occurred. Sources, dependency caches, failed probes,
old snapshots and unique evidence were preserved. The current binary is retained
as a verified lossless gzip; evidence is not repeatedly copied into the repository.

No public method signature or dependency changes. The earlier external consumer
API caveats remain open. Coverage is unchanged at 339 linked / 963 unlinked
originals; see [acceptance criteria](cpu-parity-acceptance.md). This closes the
demonstrated aggregate work failure only. No new exception, push, desktop use or
GPU work. Next bounded independent work is the five constructor/continuation
fixtures listed in the acceptance checklist.

## Receipt identities

Raw receipts retain commands, exact statuses/streams, source hashes and timings
privately. Historical failures are retained with their original tested sources.

| Receipt | SHA-256 |
| --- | --- |
| `baseline-work.json` | `1645fcfef737df184de71a329e8aa99a0cd67e4c6c99f08fa8d03aa47965601c` |
| `baseline-work.log` | `7af76b18b370a3d1ba5e623ceb01945151e1aa9d53b6ae872c3e9b76f46594fb` |
| `repeated-work.log` | `f0a243f22c3716e32cd2f9a7c9da42f5fe96e015d9e8777bc042b2650ab213d8` |
| `preserved-work.log` | `99e45fb1fc8173ad9e0a972d76f2ec1ca4b1e972c0e7c1f26310c18feb8efcd8` |
| `focused-final.json` | `b207f5e56e9d95c526a759c8555ebc6721202c140b45b7ae836e58cef7113b53` |
| `full-gate-initial.json` | `8f1daab853f0da492168c4377df837d143f3ff1cc40bab07d5e65884d39f7235` |
| `full-gate-final.json` | `6b6110d33a630824bf038c1b65cced1b24d352e9c60bfb6824a29041606daf12` |
| `full-gate-final.log` | `d4aff25154dbf9cbc09c0b5568f6ef123cf183a317afc87f5837cdf82d003097` |
| `snapshot.json` | `118548b49186bd17ddf7674997f1000b86e9933a2c6fbc96955942bb5b342430` |
| `closed-runtime/receipt.json` | `e62d782129d486ddb3a20c8b6f0353e3b395643962dcb814b4145da37e29ae9f` |
| `prefixes.json` | `979574617a7a01cf8e9fa22dee4c97ac181857172ca9270ae882352c45a68688` |
| `graphs-matching/receipt.json` | `fe772eff17c6d78e95e898299a6501d1c1b757501b33486164bdb2b344f4f970` |
| `audit/receipt.json` | `3fb813c56e190abfbac3b23cf2c75e659d48511af1241f689a052255533f9779` |
| `audit-delta.json` | `6349ba9a5105e1c7a1dc9cd512279cc0e31f73061fbe82f72ea5cf5dec523055` |
| `controls.json` | `554a721eb0eda66c0f8af6ec2199db046b280d3245ff7b92322083b2ec63a686` |
| `coverage-summary.json` | `a198e576d939795c4eda90011caae580a1338a1ec3ac945827c680e1ff7e756c` |
| `alternate-library-reclaimed.json` | `45146302d81f9a67d2990267c7cd5a20cf6fee93f369253980f832aaea240d5a` |
