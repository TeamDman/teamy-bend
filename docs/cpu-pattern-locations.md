# Pattern binder diagnostic locations

Tested source: `3b144f16f1a6dd5344a668dc0bd31a488c602aeb`. The following evidence checkpoint changes only documentation.
Upstream remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
**The three retained pattern-location gaps are closed; full CPU parity remains incomplete.**
Dependencies, toolchains, evaluation/checking semantics and safety limits are frozen.

## Correction and pinned contract

The unchanged originals `check/negative_data_field_000`, `check/negative_data_reuse`
and `halt/contraction_growth` already rejected duplicated affine fields. The port
lost the field binder's source and rendered its internal usage error with a host
location. It now reports the exact Bend diagnostic and pattern location.

Pinned [parse_bind](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1681)
retains the pattern token span, including both IDs of a reusable binding.
[match_flatten](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2720)
gives a synthesized nested-field binder the nested constructor pattern's span.
[body_flatten](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2779)
carries it into the generated lambda;
[quant_used](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3435)
uses that lambda's span when its usage fails.

The production change adds only existing binder-to-source mappings in the parser
and pattern compiler. It reuses weak source identities. It does not copy shared
references, allocate new semantic binders, alter allocation order or template
cache keys, or change core fields/serialization. No checker or evaluator production
rule changes. Variable-only fallback substitution and first-constructor-row priority
remain unchanged. Independent read-only review found no source blocker on the clean
tested commit; review itself did not execute tests.

Eleven exact negative cases cover the three originals plus multiline/imported
fields, nested constructors, a synthesized field duplicated by a fallback row,
selected-row priority, variable-only fallback, shadowing and closing-binder order.
All load successfully and reject while checking. Exact status, stdout, stderr and
absence of output artifacts are checked twice at upstream/port checking and
compiler boundaries. Emission/build/execution after rejection are **unrun**, never
runtime passes.

Two positive controls retain complete normalized graphs/text and constructor values
for nested matches and reusable Data fields. Native strict evaluation and the pure
C/JS emitters agree. Their executable emitters consistently require Base; that
refusal is distinct from pure-emitter execution.

## Validation

| Check on the tested source | Result |
| --- | --- |
| Standard gate | **1,138 passed / 0 failed / 49 ignored** |
| Formatting, strict production Clippy, all-feature build, Node provider | Pass |
| Focused integration Clippy | Pass |
| Core/normalization/parser and two affected integration targets | 110 passed |
| Occurrence units | 8 passed |
| Expanded tracked/plain checker work control | 1 passed |
| New integration target | 3 passed, included above and in full gate |
| Target matrix | 270 observations: 198 exact rejection observations, 18 unresolved-location observations, 54 positive/applicability observations |
| Renewed prior runtime/diagnostic matrix | 246 observations; existing failed lanes retained |
| Combined matrices | **516 observations**, zero timeouts; repeated status/stdout/stderr groups deterministic |
| Phase probes | All 12 negative controls load, then reject during checking |
| Inventory census | All 1,302 statuses unchanged; exactly three changed stderr streams, all now exact upstream matches |
| Prior 489-case subset | 32 exact / 385 strict IO proof exclusions / 72 unresolved, unchanged |
| Prior ten retained cells | Nine now exact; `unsafe_many` remains unresolved |
| Positive and unfinished-hole controls | 14 positive observations; 18 retain three unwaived differences |

The work control compares tracked/plain Book structure, internal errors, remaining
fuel, depth and fresh IDs for all twelve new negative controls and two positives,
in addition to its retained cases. Positive normalization runs after provenance
owners are dropped. The earlier shared-reference ownership/cache tests and default
host-stack regressions run in both focused and full checks, including the unchanged
90-application checker and deep-array parser controls. No stack override, raised
Bend guard, optimization workaround or new cancellation mechanism is used.

## Unresolved and excluded evidence

The adjacent reusable function-field refusal remains a real diagnostic difference.
Both reject the same Data-versus-Type obligation, but upstream points to `case L{+f}`
and the port points to the constructor field type. Pinned
[check-let](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3452)
conditionally relocates errors whose expected object is its newly created kind.
That separate expected-kind error-origin rule is not changed here. Both exact
streams are retained; this case has no parity credit and is no policy exception.

The old 72 unresolved checking cases still comprise 64 upstream-accept/port-reject
outcomes and eight differing rejections/preemptions. The strict IO proof exclusions
are separate from executable success. These and other ledgers overlap; do not sum
them as independent defects.

The runtime renewal preserves two repeated upstream C-generation failures and two
Node direct-IO host failures. Failed generation leaves build/execution unrun.
Imported-JS printer timeouts, dependent-pair/minimal-IO C failures, F32.read,
competing-error order, host/foreign/resource/API gaps and the incomplete 688-vector
exploration remain unwaived. Supplementary all-target Clippy was not rerun; its
historical failures are not replaced by passing standard/focused lint checks.
GPU hardware and desktop checks were not run.

No new public API changes. Existing `CliOutput::exit_code()`, emitter and
`DefDecl::completion_parameters` caveats remain; external consumers were not audited.
No mid-normalization cancellation is claimed.

Coverage now links 410 originals, with 892 unlinked: 341 checker negatives, one
checking-only module, 13 interpreter-profile cases and 537 compiled candidates.
Only three checking links are added. Runtime/interpretation links remain unchanged.
Linked receipts include partial, blocked and historical outcomes, not completed
parity. The [acceptance criterion](cpu-parity-acceptance.md) is unchanged.

Next bounded diagnostic work is the reusable-let expected-kind location. A separate
substantive applicability-first candidate remains `parse/datatype_declaration_form`,
`empty_datatype` and `empty_param_group`. Neither receives qualification credit here.

## Reproduction and preservation

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --features extended_observability --test kernel_core --test kernel_normalization --test parser --test parser_numeric --test parser_templates --test cpu_template_occurrences --test cpu_pattern_locations
cargo test --locked --features extended_observability --lib occurrence
cargo test --locked --features extended_observability --lib source_origins_preserve_checker_work_terms_and_successful_evaluation
cargo clippy --locked --features extended_observability --test cpu_pattern_locations -- -D warnings
node "$UPSTREAM/bend2/main.ts" "$FIXTURE"
node "$UPSTREAM/bend2/main.ts" "$FIXTURE" -o "$ARTIFACT"
"$BEND" check "$FIXTURE"
"$BEND" compile "$FIXTURE" --executable --target c --output "$ARTIFACT"
```

Roots are caller-selected. The committed manifests, exact expected streams and
complete positive graphs make the bounded checks reproducible. The
[case ledger](cpu-pattern-locations-evidence.csv) identifies private process receipts
by hash. Matrix child deadlines are 30 seconds, integration children 20 seconds,
and the gate 1,200 seconds. Upstream C renewal uses the CLI's C11/O3 profile;
pure port C retains O3 and `-Wall -Wextra -Werror -pedantic`.

Rust/Cargo 1.96.0, fmt nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and
PowerShell 7.5.4 remain frozen. The listed host debug/cache overrides are distinct
from Bend runtime optimization profiles. Work/depth budgets remain 2,000,000/128;
the parser guard remains 64. Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

The gate began with 11.39 GiB free and reached
10.35 GiB minimum, above the 10 GiB stop floor; preflight requires
11 GiB. Temporary storage retains a separate 1 GiB floor. Rebuilt test binaries
explain the transient growth. Only completed inactive rebuildable caches were
reclaimed, restoring 11.45 GiB. Source and unique historical evidence remain.

The compact [pre-fix receipt](../tests/fixtures/cpu-pattern-locations/before-receipt.json)
preserves all six original failure streams/statuses in Git. Command roots use
placeholders; stream bytes are unchanged, including ANSI and a repository-relative
host path. Original fixtures, expected diagnostic JSON, and the separate unresolved
kind-location streams are also committed. No private machine paths, credentials or
large binary artifacts are included.

Bulk raw logs, full receipts and captured binaries remain **outside Git preservation**.
Historical failed gates are retained with their actual source commits. This turn's
initial test Clippy failure, corrected import prototype and missing copied runner
helper are retained separately; none is counted as successful validation. Reviewed
feature-branch publication is not a main merge or release. Captured current CLI SHA-256:
`8926d2ef4a0ced9bde13df863939a69f52a0e932b311985c7f05fa708002c908`.

| Private receipt | SHA-256 |
| --- | --- |
| `full-gate.json` | `a19cb98a86e013ca98265a7e1d1d65a17bcf3ec5657ae10f1e03348f1ada01f0` |
| `focused-structure.json` | `052913c61cc6dd79aae61a559433ac7d683b6aa60e580c83ef453bf5a64e4beb` |
| `focused-occurrence.json` | `4fcf37c123b3218472d11e2d00ca562d778858f681ec83b63e1f1229ba4e169b` |
| `focused-work.json` | `2547f006cad595fdceb47fff21b773750df3a78563f464ae814a2083a13b4eb6` |
| `focused-clippy.json` | `41b86e9fb0674ec5023cc1fa90314621ca6b42028424d721285bf81dc32e540d` |
| `stack-environment.json` | `3587055f20289c5538403abd3f6e57dc859d40deabe46ce26210789071c961b6` |
| `snapshot.json` | `add1348756ff32637607148ce3f79e82bf8eb7fb2a01cb6e4b906d8caa2aac37` |
| `before.json` | `55ad1452efbfbf8533d521f1619f136c5bc3c8f028182fbe55bdb0db25836e6e` |
| `matrix/receipt.json` | `79cbaa5b8b0771064f32796e01851449b59517fc7e814d35fc46170fc33c970b` |
| `baseline-matrix/receipt.json` | `6794d27e1a2b6fe2e147e8cdcfd03bdd4a30deb1bf7a6fb7bfe0d3d596bd4c2c` |
| `baseline-matrix-initial/receipt.json` | `044a7ab334f1ed92407c4cf71358b6ba32a10097a68994de73b8cfa3574db536` |
| `phase-controls.json` | `eea73b1ef7e1eb499c69de136f539cd342bc5bea6bec55b206d5f9ce5e786c8a` |
| `audit/receipt.json` | `8224aa218177f741bad38b7e9ddbeda0335ccfa02b4aca4c92a2d3209c580a29` |
| `audit-delta.json` | `1e31d91365b362c5f56a8f7c092c689d10c37fb604e33c74cc84b607147f45e9` |
| `audit-upstream-delta.json` | `ac45be8129954785f96068905e4e36861570e3d968147037a5c414ce6b26f8e0` |
| `renewed-checking.json` | `4072e52cd03a358b263544099602600d5d03465348ac92b9bbe24cffa318b0f2` |
| `checking-renewal-summary.json` | `3f582fd6d744f1ed0cf53516ba2b78bc5dc763c3a9215f6878abb342bd18b761` |
| `retained-controls.json` | `71cdf647c4395f7e94f867e41dd30e2dd60a7d7760f974164c77b3d0c2613d29` |
| `controls.json` | `4807c64e6d016aa1e639686db92d1643f351b470207b2ada28267b7d2ddb9574` |
| `independent-review.json` | `4ee440b2e38b82f7f351f8c6c6f087b98a8794098792b9813653304f00498efb` |
| `frozen-inputs.json` | `e27263ddc01eee7611e37b1fc101600027edce4b8be5271e1429d5eaf61fb59b` |
