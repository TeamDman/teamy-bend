# Binder shadowing and catch-all priority: bounded CPU checkpoint

Tested source: `e46434c5417252287478158e45260b3cbe6603eb`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
All three [originals](../tests/fixtures/cpu-binder-priority/README.md) retain exact
Apache-2.0 upstream bytes. The new tests and observation wrappers use MPL-2.0.
No production implementation change was needed. CPU parity remains incomplete.

## Applicability and binder contracts

| Original | Complete checked main type | Bounded input grid |
| --- | --- | --- |
| `shared_param_computed_000` | `@n:Nat -> Nat` | 4 depths, 0..3 |
| `cross_binder_promotion` | `@x:Pr -> @y:Nat -> Nat` | 27 triples: pair fields and discriminator each 0..2 |
| `var_row_column_rebind` | `@a:W -> R` | 5 depths, 0..4 |

Pinned type introspection and the Rust regression assert all ordinary `Lone`
binders, domains and result structure. No reusable `+` binder occurs in this
batch. The shared-parameter fixture's historical comment mentions syntax absent
from its actual source; the observed program is ordinary predecessor matching.
It supplies no evidence for `&`/`Own` behavior.

Actual printability was established before the grid. The unchanged originals
lack Base, so upstream C/JS and port C/JS/native emitters return exit 1, empty
stdout, exact `Error: a build needs import Base\n` stderr and no artifact.
Separate Base-importing function aliases pass source checking, then all five
emitters refuse the function-valued main with identical complete diagnostics.
The pinned [printer boundary](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
explains both refusals. Neither outcome gives an original an executable pass.

## Application results and runtime observations

The [36 exact application rows](../tests/fixtures/cpu-binder-priority/applications.tsv)
are checked twice against the pinned interpreter, yielding 72 oracle values.
The Rust checked evaluator compares complete constructor results and structural
shapes; original normalized functions also match golden text and structure.
CLI pure `run` text and JSON `eval` values match separately. No separate observation
of the CLI writer's internal core is claimed.

- Predecessor input depths `[0, 1, 2, 3]` produce `[0, 0, 1, 2]`.
- For `P{a,b}` and discriminator `y`, cross-binder matching returns `b` when
  `y` is zero and `a` otherwise. All three depths vary independently. The pattern
  binder named `y` shadows the outer name without replacing its discriminator.
- Catch-all inputs 0..4 produce `KN{Z{}}`, `K10{}`, then `KN{input}` for
  depths 2..4. In particular, depth 2 returns `KN{S{S{Z{}}}}`, never `K99{}`.
  This preserves the earlier catch-all's priority and the entire matched value.

These results are grounded in the unchanged
[cross-binder](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten/cross_binder_promotion.bend)
and [catch-all](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten/var_row_column_rebind.bend)
sources and pinned interpreter observations, not comments alone.

Typed pure wrappers return `List<List<U32>>`. Nat values project to `[depth]`;
catch-all results retain tags and payloads: K10 becomes `[10,0]`, KN becomes
`[0,depth]`, K99 becomes `[99,0]`. Thus observations cannot confuse the result
constructors. IO adapters print the same lists. `ExecutableEntry::Io` assertions
ensure those tests use the native executable driver; pure `run_main` normalizes.
Upstream/port generated C and JS, native IO and pure observations match exact
stdout, empty stderr and exit 0. These projections do not close the independent
imported-constructor spelling discrepancy.

Direct upstream IO under Node returns exit 1, empty stdout and
`TypeError: require is not a function\n`; `import.meta.require` is unavailable.
The six direct calls remain blocked. Three strict IO proof checks are deliberate
proof-mode exclusions; executable checking is separately successful. No host was
installed or substituted, and neither category is counted as a runtime pass.

## Validation

| Check at tested source | Result |
| --- | --- |
| Full standard gate | 948 passed, 0 failed, 49 ignored; 110 result-bearing targets |
| Format / strict Clippy / all-feature build / TCP-UDP provider | Passed |
| New regression tests within gate | 3 passed: type/core/applications, pure C/JS, native IO C/JS |
| Original normalization / type introspection | 18 / 3 observations |
| Direct oracle applications | 6 invocations; 72 values, 36 distinct fixture/input pairs |
| Original missing-Base / function-printer refusals | 30 / 30 exact observations; no artifacts |
| Accepted-source phase probes | 12; separate from emission and execution |
| Pure output / strict eval | 36 / 6 exact observations |
| Native IO and generated IO outputs | 30 exact observations |
| Direct IO oracle / strict IO proof | 6 host blocks / 3 exclusions |
| Generation / C build | 36 successes |
| Final matrix | 216 process observations; repeated result lanes, no timeout or nondeterminism |
| Checking-status audit | 1,302 unchanged: 439 expected negatives rejected; others 366 accepted / 497 rejected |
| Positive / hole controls | 14 pass; 18 observations preserve three unresolved cases |

The [kernel](../tests/kernel_binder_priority.rs) and
[compiler](../tests/compiler_binder_priority.rs) tests ran in the full gate with
existing proof, guard, cancellation and CLI boundary controls. The gate took
308.04 seconds from clean source. Its 49 ignores remain unrun:
47 GPU hardware tests and two optional profilers. The captured binary's
`default,extended_observability` feature identity was verified by matching the
actual Cargo artifact hash, not fingerprint timestamps.

Initial applicability, normalization and application probes used the prior
`d48db566d46d0408bd2f172aca69ddf3c3d3cc23` binary. They retain that identity and
are not relabeled as new-source qualification. The final matrix repeated these
contracts with the current captured binary.

## Commands, storage and evidence

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$BEND_REFERENCE/bend2/main.ts" "$ORIGINAL"
"$BEND_BINARY" run "$ORIGINAL"
"$BEND_BINARY" --output-format json eval "$ORIGINAL"
node "$BEND_REFERENCE/bend2/main.ts" "$WRAPPER" -o "$WORK/up.c"
clang-19 -std=c11 -O3 "$WORK/up.c" -lpthread -lm -o "$WORK/up"
"$WORK/up" --threads 1 --gpu off
"$BEND_BINARY" compile "$WRAPPER" --executable --target c --output "$WORK/port.c"
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$WORK/port.c" -pthread -lm -ldl -o "$WORK/port"
"$WORK/port" --threads 1 --gpu off
```

Generated JS runs under Node; port emission uses `--target javascript`. Pure
and IO wrappers are each tested. Matrix C uses the pinned Linux O3 profile;
the separate Rust test compiler uses strict flags with no explicit optimization
switch. Host Rust debug overrides do not change Bend budgets or optimization.
Private receipts retain exact argv, exit codes, streams, fixture/artifact hashes
and durations. The [public CSV](cpu-binder-priority-evidence.csv) contains compact
contract identities. Child bounds remain 30 seconds for the matrix, 60 for the
audit, 20 for compiled regression children and 1,200 for the gate.

Build preflight remains 11 GiB, one GiB above the 10 GiB reserve. The gate started
with 11.086 GiB free and reached 10.407 GiB minimum. After binary capture,
106 inactive rebuildable integration-test executables were retired,
restoring headroom before the external C matrix. Unique source/evidence,
dependencies and toolchains were preserved. Current free space is
11.071 GiB; recheck before any further build. Emergency stop floors
remain 2 GiB workspace / 1 GiB temporary storage.

Detailed receipts link **313 originals**; **989** lack detailed receipts:
**360 checker negatives, one checking-only module, 78 interpreter cases and 550
compiled candidates**. Three interpretation rows are new; the earlier 127
interpretation, 93 runtime and 90 checking rows retain historical identities.
Linked receipts include partial/blocked cells and do not establish CPU parity.
Imported-constructor printing/upstream JS timeouts, erased-hole/TODO,
host/resource/foreign/F32.read/runtime-error-order/API requirements and the
unfinished 688-vector exploration remain unwaived. Frozen dependencies,
references, toolchains, guards, budgets, cancellation and approved demand policy;
no installations, pushes, desktop or GPU work.

Next independent batch: the five `wide_scrutinee_000..004` originals. Source
review proposes 96 vectors using every combination of depths 0 and 2 across
four or five columns. This would exercise nonadjacent discriminators, ignored columns,
default reconstruction and complete payload forwarding. Actual printability
must be probed first; only source/inventory review is complete, with no execution
credit assigned. The grid is a bounded branch/payload sample, not arbitrary-depth
or exhaustive semantic coverage.

Imported-printer [discrepancy and timeout evidence](cpu-ragged-rows.md#open-imported-constructor-printer-discrepancy)
remain at their earlier source `623c6bacfc82993e731c4c124306924c1d68d8eb`, not rerun
or relabeled here. The [acceptance criterion](cpu-parity-acceptance.md) requires
all applicable lanes and API/failure/resource contracts; linked receipts and
passing counts are not universal parity.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `8a2e8ad507bebe42aaafdda2ff535cea288199f32fd30ebd02ca5a5871f37572` |
| snapshot.json | `8525e3368177247d805bb9dea1b0644596b03daf22c69b7f376398c140f04ea9` |
| snapshot/teamy-bend.gz | `c83b50d0ecd68ddd668b76653c4afedcb736970d529271f5f0cb9a39bbfb21a8` |
| full-gate.json | `3cce52963b99c72fd263c7352177c5cfddec742ce256654c8bb3c444cf1896d6` |
| full-gate.log | `7326828af68d356e58f724d182529afd1061fd6879df699c684e630412027ddf` |
| qualified/receipt.json | `92f2b8fe3ec1cd7827efbd091657835ace2b660a742c8dcf6289d743d6df9a0a` |
| preflight-applicability/receipt.json | `4378bb9caf194ddd8ad25b9057a4bec88251e91e8e2de8c46113ad9c147096c9` |
| preflight-normalization.json | `6c57030714f8406baf08ec06d321d304c10c53929ba81e259bc95187c9000c44` |
| preflight-applications.json | `82645b627a05316e28b9c89b729138fed0f977ea7603d0e530daa0909c4f55ea` |
| audit/receipt.json | `4198cc7ca9a583caa1d9922a0c66e3e84444132841c26267fae4c94ac16c9e91` |
| controls.json | `61b2e9bc52d5ec786ac7b5a33bf271a3e999f4cb57e4fae5e68f2038fa28d74e` |
| coverage-summary.json | `e44d9a173564a06a1a9dc3356fa4f94aa7f12f2079848cbb461f8f9cad3447a8` |
| inactive-test-cache-reclaimed.json | `fe04cbef800dbe04d374622c336894b886283962ce5c0f5b554b331cba61530f` |
| qualify.py | `f4c4e41414d2125d5b24f6836ff53448bf6fce9bff1012d7e0c4be5d0fe9e406` |
| types.mjs | `d2319d1a519234fbc7525a97f07d8ce456c3e3ef6aa14399e071265c54679709` |
