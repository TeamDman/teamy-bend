# Constructor values, continuations and reviewed sharing

Tested source: `7209617167158e9fd6338e59f7ac73f505e4b88b`; starting checkpoint:
`0c949f277b20db7a4a4ade79feba674bf33055bd`. Target:
[pinned Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
No production code, dependency, public API or resource policy changes in this batch.
The new coverage uses owned Facet for JSON observations.

## Independent sharing review and retained regression

The independent read-only review inspected production range
`2ad725bc07ede2320074199f94434700bcf29c67..20fec64bebb656c3fea84896e177f6cda8a9e75d`.
It found **no demonstrated blocking correctness defect**. Successful App-to-Ctor
weak heads retain raw fields, Weak keys preserve allocation identity without
owning the input, and normalization-scoped cleanup prevents stale alias results.
Steps and depth are checked before cache hits. No error, blocked call or
function head is cached. This agrees with the pinned
[raw-field weak-head contract](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2807)
and [shared term cells](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L686).

The review recommended two explicit controls, now in
[composition_work_tests.rs](../src/kernel/composition_work_tests.rs): a real
cache hit returning a constructor containing a captured continuation, and no
increase in the source application's strong count while that memo is live.
Two occurrences of the shared application normalize to distinct fresh binders;
each retains `Captured{}` when separately applied to `Left{}` and `Right{}`.
The input graph is unchanged, depth unwinds and the table is cleared. A separate
review addendum inspected this test on the exact tested source and found the
recommendations addressed. Neither review claims new execution by the reviewer.

The unchanged exact 27-result aggregate regression remains in the full gate:
23-result prefix **634,793** steps; 24-result prefix **861,655**; full aggregate
**1,338,629**, below the unchanged 2,000,000-step/128-depth guards. Lazy fields,
first-error order, insufficient work, depth refusal, alias cleanup and writer/
cancellation controls also remain. Cancellation is still checked before and
after pure normalization, before writing; there is no mid-normalization
interruption guarantee. The prior external aggregate timing/72-observation
receipt stays tied to source `20fec64`; it is not relabelled as a fresh external
matrix here. General memory or wall-clock parity is not established.

## Original, adapter and executable applicability

All five [originals](../tests/fixtures/cpu-constructor-continuations) are byte-for-byte
copies of pinned `tests/flatten/` sources. Their normalized graphs and exact text
agree in both implementations:

| Original | Exact normalized text |
| --- | --- |
| `ctor_construct_000` | `a => b => A{a, b}` |
| `ctor_construct_001` | `a => b => a` |
| `ctor_continuation` | `Print{S{Z{}}, x => Tick{t => Done{S{x}}}}` |
| `ctor_continuation_go` | `f => f(S{Z{}})` |
| `ctor_default` | `\{Z: A{}; (a => B{a})}` |

The originals do not import Base. Actual upstream C/JS and port C/JS/native
builds exit 1 with empty stdout, exactly `Error: a build needs import Base\n`,
and no output artifact. Separate adapters add Base and import the unchanged
original. Four reach function-printer refusal; `ctor_continuation` reaches
unprintable-Type refusal for its local `Io`. All adapted diagnostics, statuses
and absent artifacts match upstream. The local `Io is Type` is an ordinary
constructor tree, not Base IO and not evidence of performing host effects.

[Kernel tests](../tests/kernel_constructor_continuations.rs) separately compare:

- Five normalized graphs and text, with bound-variable identities and outer
  capture inspected explicitly.
- **25 full-value direct applications:** nine constructor pairs, nine ignored
  second arguments, four default-match inputs, and three continuation callbacks.
- **Six nested constructor-only continuation applications:** outer depths 0, 1,
  4 crossed with inner depths 0, 2. The inner input is unused; the outer capture
  determines the final `Done` payload. These normalized bodies contain only
  constructors, lambdas and variables, so beta substitution needs no IO driver.
- **36 closed projections** in five wrapper mains: 34 calls of wrapper `apply`
  plus two direct callback observations. At most three constructor layers are
  observed. This is bounded application coverage, not a recursive IO interpreter.

[Runtime tests](../tests/compiler_constructor_continuations.rs) run the five
closed wrappers with the native Pure entry driver, generated C and Node JS;
strict proof evaluation is tested separately. Scalar/list projections avoid
conflating an imported-constructor spelling issue with computation. They do not
close that existing printer discrepancy.

## Results and commands

The full standard gate on clean tested source passes **997 / 0 / 49** across
126 result-bearing targets in **472.060 seconds**. Format, strict Clippy,
all-feature build and native Node socket-provider checks pass with no warnings.
The 49 ignores are 47 GPU hardware and two optional profiler tests; they remain
unrun. Compile-fail doctest diagnostics are expected passing rejection tests.

The preliminary focused run at `d66b8bfda8fa1e654c335153803593e0f34a3d78` passes
**302 / 0 / 1** in 219.038 seconds. It emitted unused helper warnings from a
standalone CLI integration target. The final source combines those tests with
the compiler target, without suppression, and strengthens the review regression
with two distinct inputs. All are renewed by the final full gate. No failed
Bend comparison or production defect was found in this batch.

The [execution ledger](cpu-constructor-continuations-evidence.csv) records:

| Collection | Observations | Result |
| --- | ---: | --- |
| Original graphs, two implementations, twice | 20 | Exact structural agreement |
| Original text/checking/type, applications and applicability | 150 | Complete; includes 100 exact build/printer refusals |
| Closed wrappers, six execution lanes, twice | 60 | Exact status/stdout/stderr; all exit 0 |
| Closed wrapper generation/build | 30 | All exit 0 |
| Checking audit | 1,302 | Status and streams unchanged; 439 expected negatives reject |
| Additional positive controls | 14 | Pass |
| Three existing hole/TODO disagreements | 18 | Preserved, unresolved |

Every open/closed external invocation uses the unchanged 30-second bound; no
case timed out. Repeated results are deterministic. The remaining 863 checking
rows split 366 accepted / 497 rejected; these controls do not establish exact
upstream diagnostic parity for all rows. The graph probe uses the library whose
Cargo fingerprint matches the captured binary's feature/profile selection.

Temporary host build settings (not Bend C optimization flags) and actual final
standard-gate command:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The exact historical focused command, before combining the standalone target:

```sh
cargo test --locked --features extended_observability --lib --test kernel_constructor_continuations --test compiler_constructor_continuations --test cli_constructor_continuations --test kernel_open_closures --test compiler_open_closures --test kernel_normalization --test executable_demand --test runtime_effects --quiet
```

External matrices use the captured final binary. Representative actual command
forms, with `fixture` selecting the original, adapter or closed wrapper separately:

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

The compiled external C profile is O3, matching the pinned upstream CLI; strict
warnings are applied to port-generated C. Rust 1.96.0, nightly-2026-05-24 fmt,
Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4 stay frozen. Cargo.lock SHA-256:
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured binary SHA-256:
`0fe8e92bb1a1f028c830271b817f249c78ffa5aa389b8fc6c278c581f53a68f4`.

A private runner review found a missing `os` import in the unused timeout branch.
The executed script is preserved; all recorded invocations had completed without
timeout. The corrected runner then killed and reaped its own deliberately slow
child at the unchanged 30-second deadline. This runner control does not alter
any Bend result or runtime budget.

## Storage and remaining scope

Build preflight requires 11 GiB total free, a 1 GiB margin above the 10 GiB
reserve. The final gate starts with 11.017 GiB and reaches a minimum of 10.115
GiB. Completed first-party test and inactive alternate-feature library caches
were reclaimed with ownership/hash/activity checks; dependencies, source and
all unique historical failure evidence remain. The final checking audit starts
with 11.008 GiB. New evidence is about 11 MiB, including a verified lossless
binary archive; large generated artifacts are not copied into the repository.
No disk exhaustion or unrelated process termination occurred.

Tested source `7209617167158e9fd6338e59f7ac73f505e4b88b` adds five unchanged
constructor/continuation originals and seven regression tests, with no production
change. An independent source review found no demonstrated blocking defect in
the preceding sharing repair; the two recommended cache-hit/captured-binder and
live weak-key ownership controls now pass. The exact 27-result aggregate stays
under its unchanged budget. [Scope, commands and receipts](cpu-constructor-continuations.md).

The full gate passes **997 tests, zero failures, 49 ignored**. Five originals
match normalized graphs and text; 25 direct original applications, six nested
continuation applications and 36 closed projected observations are distinct
coverage. All 20 graph observations, 150 open/applicability observations and
90 closed-runtime observations complete. The latter contain **60 exact
status/stdout/stderr matches and 30 successful generation/build steps**, using
upstream and port C O3 and generated JS separately. Original no-Base refusals
and adapted function/Type printer refusals are exact; they are not executable
passes. Local `Io` constructors do not execute Base IO. All 1,302 checking
statuses and streams are unchanged; 439 expected negatives reject. Fourteen
positive controls pass; 18 observations preserve three hole/TODO disagreements.

Detailed evidence now links **344 originals**, including five new interpretation
rows; **958** remain without detailed receipts: **357 checker negatives, one
checking-only module, 63 interpreter cases and 537 compiled candidates**. The
156 interpretation, 95 runtime and 93 checking links include historical,
partial and blocked cells. They are not 344 fully qualified programs and do not
establish CPU parity. Existing imported-constructor printing/JS timeout,
erased-hole/TODO, resource/host/foreign gaps, F32.read, competing failure order,
downstream API caveats and the incomplete 688-vector exploration remain unwaived.

Next bounded work: renew the minimal imported-constructor printer reproducer
from [ragged rows](cpu-ragged-rows.md#open-imported-constructor-printer-discrepancy).
Historically the interpreter, upstream C and port pure CLI print `helper.Box{7}`,
while port generated C/JS print `Box{7}` and upstream JS times out. This batch
uses scalar projections and does not qualify that constructor spelling or
waive its timeout. Preserve those controls before any targeted correction.
Frozen dependencies, toolchains, pins, budgets, guards and cancellation remain;
no installations, pushes, merges, desktop or GPU work.

## Receipt identities

Private receipts retain exact commands, status/stream bytes, source hashes,
timings, artifact outcomes and build profiles. Historical preliminary/failure
receipts retain their own source identities. Review files are read-only analyses,
not test runs.

| Receipt | SHA-256 |
| --- | --- |
| `sharing-review.md` | `5633599bf10921c11e2788c8324c8b116be3a214890e98febd47f185392a3b94` |
| `sharing-review-regressions.md` | `987d10f977c84f09f307ff242f594557f36d64b7f0b945d5a4e4fed4178defdb` |
| `focused-final.json` | `97a17307df2e3e055d9c3d2218281bb0228f269d9e3339eb7277a1ef4663efe2` |
| `full-gate-final.json` | `55c911369fa8d0a22dfe3d559057c92488a1e4b251cae18a5d8df7965e4a65c6` |
| `full-gate-final.log` | `b472aa7f72fb449f70a84c5c4159bc516ca8f5291c1128fe98cbdb7707aaf242` |
| `snapshot.json` | `b3e3abe455a5b6d8a5ec007ded0268173fd07efdbe03262fd891127ae708359d` |
| `open-matrix/receipt.json` | `ef7bf73cf6122935098c6c721f60753b9cd004c0041e673c5d55599d42a5c333` |
| `graphs-matching/receipt.json` | `8b9969cfff9002fe89ee93a028b7975b40addb929f9b0ad57d69da74d5b94069` |
| `closed-runtime/receipt.json` | `9efc46603978ad0dd6ea85a383e2d88caa170bc0fca0676705f36993ba12534d` |
| `audit/receipt.json` | `ddc954ce6f78b7507b2ff1307f0d2b48f553e0466b4fd201ffe06128f914e459` |
| `audit-delta.json` | `2ca4f063a069da76e902d1161470cccbd895ed62e242aebaa180420d26146be0` |
| `controls.json` | `ac385f3c94bca5870c9239c8f1d76b73f7ad564a91607a731684bb22f2895193` |
| `coverage-summary.json` | `40de154353aa9fac09fd673ab2bb074dd6784d40fa65145d598c5831c678ec68` |
| `runner-review.json` | `d7ac9c19f0e11a3b22617d72eeeec3a194ab5b16a2503a7df4af42c2a6ae92ea` |
| `runner-timeout-control.json` | `420fbb364d33d899edad711008c69a21f0c855f5e0a957289fd38eb11e5bd9a5` |
| `inactive-test-cache-reclaimed.json` | `092e3d84bbc29067fcfd2dd97d1b0d16537a765ecebd3bc88b97496405d1d7e2` |
| `alternate-library-reclaimed.json` | `b7ec8422bb4d89d9536a35befdfea8779004ab2255b90246c84a82cec4be891b` |
