# Reusable Data, affine functions and quantity diagnostics

Tested source: `a73825e91bdc8b5184b7bcc04597773c2757c503`. Starting published checkpoint:
`25b47aa5e3635b71089a85b9886f6fd9f3fc9768`. The final evidence checkpoint changes
only documentation. Matrix/diagnostic engine source is `96f4d4759a91c8ba81c04550a238aa476269fcd4`; the later
source changes only an equivalent assertion in the new test. Production is
byte-identical across those commits. The final build embeds a different revision
and timestamp, so it has a separate binary snapshot. The 728-observation matrix
and 1,302-entry audit remain attributed to their actual engine binary; renewed
positive/refusal controls and final gates cover the later build. The [upstream target](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599)
remains frozen. Full CPU parity remains incomplete.

## Scope and demonstrated corrections

The unchanged [`grade/accept_data`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/grade/accept_data.bend)
checks reuse of Nat, Data lists, pairs and equality evidence; phantom function
types; affine closure storage and consumption; and Data-to-Type weakening.
Ten original definitions, 26 bounded direct calls, eight consuming observations
and two complete aggregates retain exact normalized graphs and text. Equality
results retain both Reflexivity nodes and function results retain lambdas.
Empty elimination is checked/normalized without fabricating an inhabitant.

Two new negative controls exposed diagnostic defects, not accepted invalid
programs. Reusable affine types rejected without a Bend source location; repeated
use of an affine binder rejected with internal quantity-accounting wording.
The [pinned `quant_used`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L611)
reports the declared binder and measured excess use in the outer context.
The port now preserves that structured diagnostic while retaining the internal
cause. Weak external provenance anchors generated lambdas/lets at their binder
occurrence. Grown datatype/arrow expressions inherit their head/domain's span,
including multiline parenthesized heads. No core fields, sharing edges,
serialized terms, rejection predicates or evaluator policies change.

Eleven appended negatives and one independent imported helper cover invalid
quantity/kind, repeated affine use, erased live use, completed-law/nested/local
binders, false evidence, impossible Empty arguments and multiline/imported
locations. Tracked and ordinary checking have identical terms, decisions,
remaining work/depth and fresh-ID accounting. Existing cancellation, structural,
source-hole, named-call and aggregate-sharing controls remain in the gates.
There is still no mid-normalization cancellation claim.

The inventory audit also reproduced an older stale source-address defect in the
unchanged no-main negative `proof/goal_ladder_state_027`: the actual `Type` at
line 234 had been attributed to a later definition. Merely adding new anchors
moved the wrong line from 237 to 236. Source identities now retain weak origin
roots independently of transformed-node sweeping; parsed Type terms have real
spans. This holds distinct captured roots (including generated surface roots),
not every substitution result, and adds no strong ownership of terms or children.
Tests cover capture before transformation, sweep before first source query,
allocator identity reservation, bounded transformed-node cleanup, release on
tracker drop, exact original rejection and unchanged checking work. This second
original is checking-only negative evidence, with no runtime claim.


## Applicability and results

The unchanged original normalizes to `(1n, 1n)`. Upstream executable generation
requires Base. Both import orders with Base reject before generation in both
implementations: root list-sugar constructors are not the imported module's
constructors. Those controls preserve the exact failures. No source rewrite or
namespace workaround is used to manufacture upstream executable coverage.

The port's existing strict pure backends separately execute original main and
both aggregates. They cover every child of the 20 constructor-valued direct
results and eight consuming observations. Native, C and JS constructor trees
match pinned interpreter graphs in two runs. This is **not** upstream executable
C/JS parity; those executions remain unrun after generation refusal.

| Check | Current result |
| --- | --- |
| Standard gate | **1104 passed, 0 failed, 49 ignored**; formatter, production strict Clippy, all-feature build and real Node provider pass |
| Focused family/aggregate/demand/effect controls | **46 / 0 / 0** |
| Complete graph comparisons | 184 observations: 46 sources/definitions × two interpreters × two repetitions |
| Normalized text | 184 exact observations over the same definitions/programs |
| Original direct calls | 52 pinned-interpreter observations, plus typed port API regressions |
| Exact rejection controls | 280 status/stdout/stderr matches; 160 generation refusals also leave no artifact |
| Strict pure runtime | 18 exact native/C/JS constructor observations; nine generation/build controls |
| Matrix total | 728 observations including one probe build; zero timeouts |
| Inventory status audit | 1,302 statuses unchanged from the prior port checkpoint; no new acceptance, timeout or abnormal exit |
| Supplementary all-targets Clippy at `96f4d4759a91c8ba81c04550a238aa476269fcd4` | **Failed**, seven observed pre-existing test-only lints; fail-fast, not exhaustive |

The extended lint command was `cargo clippy --all-targets --all-features --locked
--offline -- -D warnings`. That run stops on
two ignored `map_err` causes and five `assert!(Result::is_err())` diagnostics in
unchanged nested-computed/dependent-proof tests. The earlier run stopped on four
different pre-existing test-only lints. Parallel fail-fast results are not an
exhaustive lint inventory. Focused strict Clippy on the two new test targets passes.
No warning suppression or unrelated style migration was applied.

The final checker audit changed 58 diagnostic streams;
48 match pinned upstream stderr exactly.
Ten changed-stream cells remain unresolved. Six have matching normalized content
and source spans but label the enclosing datatype instead of its constructor;
three lack the pattern-field binder anchor despite matching measured usage and
rejection. `check/unsafe_many` is a pre-existing acceptance disagreement
(upstream accepts, the strict port checker rejects), not a diagnostic-only match.
No new exception is approved. The exact cells are: `check/cop_field_formation.bend`, `check/data_self_ref.bend`, `check/negative_data_field_000.bend`, `check/negative_data_reuse.bend`, `check/nested_field_bound.bend`, `check/unsafe_many.bend`, `grade/reject_fn_field.bend`, `grade/reject_negative.bend`, `grade/reject_plus_field.bend`, `halt/contraction_growth.bend`.
Status agreement alone is not exact diagnostic parity. Fourteen positive
controls pass; 18 observations retain the three erased-hole/TODO disagreements.
Historical unmodified failures remain unwaived: imported-constructor JS printer
timeouts, the dependent-pair undefined-CID C build with two unrun executions,
minimal-IO C failures, F32.read, competing failure order, foreign/host/resource
and API coverage, and the incomplete 688-vector exploration.

## Compatibility and reproduction

The preceding public `DefDecl::completion_parameters` addition remains a Rust
source-compatibility break for external struct literals. Repository constructors,
raw-emitter arity consumers, checker canonicalization and checked runtime callers
were inspected; no additional call-site update is required here. External
consumers were **not audited**. Existing status-bearing CLI failures require
`CliOutput::exit_code()` handling, and prior executable-emitter Err compatibility
caveats still apply. This batch adds no public AST field.

Commands use existing Rust/Cargo 1.96.0, formatter nightly-2026-05-24, Node24.19.0,
Clang19.1.7 and PowerShell7.5.4. The existing activation script supplies PATH and
`TEAMY_BEND_CC`. Host build overrides are `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=4`,
`CARGO_NET_OFFLINE=true`, `BEND_GPU=off`, and `RUST_TEST_NOCAPTURE=1` for the gate.
These host debug settings are not Bend optimization profiles.

```sh
pwsh -NoLogo -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --offline --features extended_observability \
  --test kernel_data_grades --test compiler_data_grades \
  --test kernel_match_contracts --test compiler_match_contracts \
  --test kernel_telescope_capture --test compiler_telescope_capture \
  --test compiler_imported_printer --test compiler_dead_residue \
  --test executable_demand --test runtime_effects -- --nocapture
cargo clippy --test kernel_data_grades --test compiler_data_grades \
  --features extended_observability --locked --offline -- -D warnings
```

Fixture assembly and ownership rules are recorded in the
[fixture README](../tests/fixtures/cpu-data-grades/README.md). Strict pure source
uses `teamy-bend compile SOURCE --entry ENTRY --target c|javascript --output FILE`
without `--executable`; C builds use the configured Clang with
`-std=c11 -O3 -Wall -Wextra -Werror -pedantic SOURCE -lm -o FILE`.
Pinned checking/generation uses `node bend2/main.ts SOURCE` and
`node bend2/main.ts SOURCE -o FILE.c|FILE.js`. All matrix children have 30-second
bounds; gate/focused runners have 1,200-second bounds. Full commands, statuses,
streams, source hashes and durations are retained privately in the data-grades
receipts. No upstream/docs file, dependency, toolchain, guard or budget changed.

The earlier successful gate at `47deb3bac65978a3f89e405dd6baa757bb1315a9`
(1,102/0/49 and 710 matrix observations) is retained as historical evidence.
The source-root correction at `96f4d4759a91c8ba81c04550a238aa476269fcd4` also passed 1,104/0/49 and 46/0/0.
Final gate counts above apply to the reviewed test cleanup at `a73825e91bdc8b5184b7bcc04597773c2757c503`.
One superseded cleanup gate was canceled and is not qualification evidence; both
initial test-lint failures are retained. No historical result is relabeled. The initial
Data-graded aggregate harness was correctly rejected and corrected to Type;
that setup failure and both failed Base import experiments remain preserved.

The [compact ledger](cpu-data-grades-evidence.csv) hashes these receipts; raw
captures, CLI snapshots and unique historical failure artifacts remain outside
Git and require separate preservation. Disk preflight requires 11 GiB and stops
below 10 GiB (temporary filesystem floor 1 GiB). Completed inactive test binaries
are rebuildable caches; source and unique evidence are retained.

## Remaining acceptance work

Detailed links now cover **392 originals**;
**910 remain unlinked**: 356 checker negatives,
one checking-only module, 16 interpreter cases and 537 compiled candidates.
These are applicability categories, not counts of missing runtime tests.
Historical/partial/blocked links do not imply full parity; the explicit
[acceptance criterion](cpu-parity-acceptance.md) still requires applicable
contracts and unresolved cells to be closed or reviewed explicitly.

Next independent candidate: `reg/framed_cell_dep`, with actual proof-function/IO
applicability first, four bounded dependent-worker inputs (0, 1, 3, 8), the
unchanged original IO observation, and a false-evidence control. This is a
proposal only; no execution credit is assigned.

## Receipt identities

Matrix/audit CLI SHA-256: `eecad52ec06a893069ac11254070315826ec4ef1b18cecdf416ac2bb99744e86`.
Final gate/control CLI SHA-256: `2567508d3703040cbff4ba434c8048817e722010a6ca1e8eebdad49371bfc62f`.
Frozen Cargo.lock SHA-256:
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

| Receipt or runner | SHA-256 |
| --- | --- |
| `full-gate-reviewed.json` | `e7755465b380e6f49074c6dea7f4582fe41734292b0569b7b4733c64bb9f4120` |
| `focused-reviewed.json` | `69e316631fbaadda1e1268e4a6edfae0ae3b5718e83add7b5c10bac1730a6db9` |
| `full-gate-final.json` | `3dc609beeb250772112e84d28f46890cf5db26185a2fc54fb14496e1c9fa03fd` |
| `focused-final.json` | `ac70d4c778025540fdb8122385ec92a8987a542017c95b4ffa7b851abc1af864` |
| `matrix/receipt.json` | `4efb9d4e554b37dd1a25388c51ec414c65713bfa553f91f7b0e991136f87a704` |
| `audit/receipt.json` | `00aeb15cab3f41f3f76687c4fdce9f7f890bf25205262ca51f72e82e05ec3a5b` |
| `audit-delta.json` | `8de6410dfbe1f2cb447e4ad17833402db451eae81fc9ac33b545e260e93f83c4` |
| `audit-upstream-delta.json` | `e63da45598218872ac0c76e932a364b033d29ccfcc94bcbbe233647710c1008e` |
| `remaining-diagnostics.json` | `a679691fefab539620d8fc30679e3f45a736b2b0877baff65c6b412704df6ca1` |
| `controls.json` | `7eeda9d0202dbee2f70db0597848e482c8c632b28a583109a3a8597cc78f30ce` |
| `controls-reviewed.json` | `7992f29ea9e2fffb80b95dabc235e1ed5f11dcc557ee33b57af0c2f3487c6a0d` |
| `independent-review.json` | `5410864b22cca84c0e33fe4d450bb8f55e4582e66539f0de9d708c65dc5c4f89` |
| `test-cleanup-review.json` | `5c00ca67fb6f329ede19467830933720e13d11a36a8bfeab3da5036ca612fbe6` |
| `api-callsite-audit.json` | `2a94e31f4f862c63247b6edacbff357506944e1afad99ea0c8e4519fde9176b7` |
| `extended-clippy-classification.json` | `88a93438ee47f4200c06c214a013bd5b74de772d8618eba81059c9cda0d0ea0a` |
| `focused-clippy-checkpoint.json` | `944c67ae5f51f8b23dc2a32ee963757a4485b504bef93e1a4405981017bb8fc0` |
| `snapshot.json` | `6a2a3083943b4e426f549349685a8765a2b31f1d13ebdd2e0c134060b94036d0` |
| `snapshot-reviewed.json` | `b4a09a6104e48299b878d2dbf7bef63ae38b3068aef7a22f4c58bde1a4811ed8` |
| `frozen-inputs.json` | `2b304a76f01ffde98aec2f93ed47e01e14168f9b51a0f7862c790af8530d8c14` |
| `coverage-summary.json` | `8f85253fc3511284ecb07a3378bf8db024667a25fc4f086d185e94a9b288e980` |
| `matrix.py` | `d4dd5c5d22c01dcc04a40d389acaed2156de119eb2ff6c8ea61c68f27f6abbc9` |
