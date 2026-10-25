<!-- SPDX-License-Identifier: MPL-2.0 -->
# Refined datatype diagnostic formatting

Tested source `9a260404fbb5819870e48f14a78bb80b7afc9181` starts from published
`9f218a786bacccfadab9f1a81e6f024524bf9e58`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, docs remain
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Dependencies, toolchains, lockfile,
semantic types, termination checks, work/depth guards and cancellation boundaries
are unchanged. Full CPU parity remains incomplete.

## Pinned representation and correction

The unchanged `halt/non_structural_descent.bend` already rejected in both
implementations with matching self-call message and source location. Its context
still differed: port `Nat - [Zero]`, pinned upstream `Nat<> - Zero{}`.
[Upstream term_show's ADT case](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1363)
prints an explicit angle-bracket argument list whenever exclusions exist, followed
by each excluded constructor in order. It groups the result only above precedence
1. [err_show](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1446)
uses that same renderer for normalized context types.

The shared renderer now follows this rule, including arrow/infix operands,
application heads and open-list heads at precedence 2. Lower-precedence positions
remain ungrouped. A syntax-only helper recognizes a nonempty exclusion list;
printing never evaluates arguments, rebuilds terms, reorders exclusions or alters
source origins. No checker, normalizer, runtime or compiler logic changed.

This intentionally changes **public Display text for refined datatypes**, including
nested renderings and diagnostics that contain them. It is not a claim that all
public strings are unchanged. Public function signatures and semantic term fields
are unchanged; existing consumers that rely on the old bracket spelling will see
the corrected representation. The external invoke-caller audit remains outstanding.
No new status path or mid-normalization cancellation facility is introduced.

## Evidence

Five negative cases cover the unchanged original, multiple exclusions,
parameterized lists, imported definitions and a false proof after constructor
elimination. Baseline [before/after receipts](../tests/fixtures/cpu-refined-type/refined-type-before.json)
retain actual source hashes and diagnostics from the preceding clean `b8a07c2`
binary, production-identical to the starting documentation checkpoint.
All five already rejected; only the refined-type text changes.

**90 exact stream commands and 30 separate phase probes** match pinned checking
and C/JS generation refusal. Port check/eval/run and executable C/JS/native
compilation refuse with status 1, empty stdout and no artifact. Full stderr,
including context ordering and source windows, is compared without normalization.
Each lane repeats twice deterministically. No negative runtime execution is claimed.

Two positive constructor-elimination controls each return `7`. Their **52 commands**
comprise eight pure normalizations, 16 checking/phase observations, 12 generations/
builds and **16 actual C/JS executions**, across port and upstream twice each.
Pure native API run_main is normalization, not native IO. Upstream C uses its
actual O3 profile; port C retains strict warnings. All four generated **port C/JS
artifacts are byte-identical to the saved baseline artifacts**. This byte-equality
claim covers those controls, not every possible emitted program or native binary.
Matrices use 30-second child bounds and complete without timeout or mismatch.
See [matrix commands and hashes](cpu-refined-type-results.json).

[Sixteen direct printer observations](cpu-refined-type-printer.json) come from the
pinned term_show function. Tests compare exact text for empty/nonempty parameters,
one/multiple ordered exclusions, high/low precedence positions and shadowed binder
IDs. They assert unchanged Debug structure and Rc ownership. These are programmatic
printer terms; no parsing, type acceptance or execution claim follows from them.
The five real source diagnostics supply separate checking/provenance evidence.

The clean standard gate passes **1,205 tests / 0 failed / 49 ignored** in 855.00 seconds. Formatting, strict production Clippy, all-feature build and the real Node socket-provider checks pass. Ignores remain 47 CUDA hardware tests and two optional profilers.

Focused tests pass **10 / 0 / 0**, including two renderer regressions, two new CLI/runtime tests and existing descent/lexical-scope controls. Targeted strict Clippy passes. Focused receipts identify the dirty implementation candidate; the clean gate rechecks the committed source, including prior self-call, stack, sharing, work/depth, cancellation and diagnostic controls. No test assertion, ignore or budget was weakened. Source and evidence received independent review.

## Current census and remaining gaps

The renewed 1,302-fixture census measures **366 both accepted, 355 exact rejections, 494 acceptance disagreements, 81 different rejections and 6 upstream host/oracle failures**.

All port statuses and phases are unchanged. Exactly one port stderr stream changes: `halt/non_structural_descent`, which now matches upstream byte-for-byte. All other 1,301 port streams are unchanged. All 385 historical IO members renew byte-for-byte. **190 measured differences remain outside that exact cohort: 109 acceptance and 81 rejection differences**. IO-symptom candidates are not additional exclusions.

The preceding census had 191 residual differences. Only one fix is attributable to this port change; upstream transitions versus that run are none. Current host/oracle failures: `check/bystander_hypothesis_cost.bend`, `check/deep_hyp_rewrite.bend`, `page/deep_list_walk.bend`, `reg/borrow_fork_hold.bend`, `reg/dead_param_fork.bend`, `rfc/share_loop.bend`. Historical instability remains unwaived even when a current run accepts an earlier failing input. These checking-only oracle observations qualify no runtime lane.

[Reconciliation](cpu-refined-type-census.json) records exact changes and raw identities. [Changed status rows](cpu-refined-type-census-delta.csv) extend the previous [self-call delta](cpu-self-call-census-delta.csv), whose report identifies its full CSV base. Reconstruct the previous CSV first, set all engine_commit fields to this tested source, then apply the new delta. Unchanged rows are not duplicated. Both-accept means checking-status agreement, not semantic/runtime parity; historical receipts keep their actual commit identities.

Applicable original linkage rises **457→458 of 1,302**. The **844 unlinked** entries comprise 521 interpreter/compiled candidates, 309 checker negatives, 13 interpreter-only cases and 1 checking-only module. Linkage is not universal parity; authored controls do not inflate the pinned inventory count.

Nat-first C ordering remains pending/unwaived, and the approved wildcard cases
remain a separate historical scope. Compact Nat/string lowering is unadopted.
Existing source-hole/TODO, resource, F32.read, foreign/host, oracle, external-API
and supplementary all-target lint differences remain explicit. No semantic
exception was added and old matrices are not relabelled as current runs.

The next independent diagnostic gap is `check/family_head_bare`: both checkers
reject, but the port reports its internal parameterized-datatype message instead
of the pinned family-instance expectation and source context. Nearby
`data_tuple_family` and `data_family_alias` actually accept upstream and belong to
the historical strict-IO cohort; their comments do not establish a family-head
rejection. [Current observations](cpu-refined-type-next.json) keep these distinct.

## Reproduction and preservation

Activate the installed tools and use the recorded host build profile:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability \
  --test cpu_refined_types --test kernel_scope_printer --test kernel_descent_alias
cargo clippy --locked --features extended_observability \
  --test cpu_refined_types --test kernel_scope_printer -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$UPSTREAM/bend2/main.ts" tests/fixtures/cpu-refined-type/halt/non_structural_descent.bend
"$BEND" check tests/fixtures/cpu-refined-type/halt/non_structural_descent.bend
python3 scripts/checking-census/run.py --reference "$UPSTREAM" \
  --binary "$BEND" --phase-probe "$PROBE" --output "$FRESH_CENSUS" \
  --build-receipt "$BUILD_RECEIPT"
```

Use the exact binary/probe identities in [validation](cpu-refined-type-validation.json)
and the [census build instructions](../scripts/checking-census/README.md). All matrix
commands and artifact identities are recorded with portable placeholders. Repeated
rows are compacted only after exact stream equality checks. To replay the printer
observations, load the pinned module and pass each saved `term` to `term_show`:

```sh
node --input-type=module - "$UPSTREAM/bend2/bend.ts" docs/cpu-refined-type-printer.json <<'JS'
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
const { term_show } = await import(pathToFileURL(process.argv[2]).href);
const { cases } = JSON.parse(readFileSync(process.argv[3], 'utf8'));
for (const row of cases) {
  if (term_show(row.term) !== row.text) throw new Error(row.name);
}
JS
```

The census retains four workers, 30-second upstream and 60-second port/probe bounds.
Host Rust debug/incremental overrides are distinct from Bend optimization profiles.

Minimum recorded free space was **11.179 GiB workspace / 4.538 GiB temporary**. Existing 11 GiB build preflight and 10 GiB workspace / 1 GiB temporary stop floors remain enforced. Existing cache placement and disabled host debug/incremental data remain in use. No cache or unique historical evidence was deleted.

Compact unique failures, attributed sources, goldens and reproducible receipts are
committed. Bulk logs, full raw census streams, generated artifacts and lossless
binary snapshots remain local and are not preserved by Git pushing. No installs,
desktop/GPU work, main merge or release occurred.
