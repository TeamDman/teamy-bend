<!-- SPDX-License-Identifier: MPL-2.0 -->
# Bare family-head diagnostics

Tested source `c7519a0d1a5efeb872ccc6b1ab011e1b85f04b2e` starts from published
`f3bca9a93d0c2385a55df5bee843b8443e0ef64f`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, docs remain
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Frozen dependencies, toolchains,
semantic types, termination checks, work/depth guards and cancellation boundaries
are unchanged. Full CPU parity remains incomplete.

## Contract and correction

The unchanged `check/family_head_bare.bend` rejects in both implementations.
Pinned upstream reports `a family instance (write List<..>)`, observes `List` and
points to its use in `bare`. The port previously exposed its internal
`a parameterized datatype needs <...> arguments` through host error formatting.

[The pinned reference inference rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3260)
requires an explicit family instance; it also rejects a bare parameterized family
in an erased context. The port retains its exact rejection branch and adds the
existing structured diagnostic with the rejected reference, current definition
and context. No parser, term, type, reference sharing or evaluator change is needed.
Parameterized family references already remain inert during normalization.
Diagnostic rendering uses only the remaining allowance on a clone; failure to
render retains the original rejection. The change does not inspect later arguments.

**Public compatibility:** `KernelError.message`, `Display`, equality and shape are
unchanged. The crate-private `checker_diagnostic()` now supplies metadata for this error, and
derived `Debug` includes that metadata and can therefore differ. CLI stderr
intentionally changes to the pinned Bend message/context/source window. Such
formatted failures use the existing status-bearing `Ok` output path: programmatic
`invoke` consumers must inspect `exit_code()`. This batch neither changes that API
shape nor completes the outstanding external invoke-caller audit. No
mid-normalization cancellation facility is claimed.

## Bounded evidence

Eight negatives cover the unchanged original, an argument's type, a nested
annotation with context, an imported definition, template substitution, a local
family, an erased argument and a family application with an invalid later argument.
All eight already rejected before this fix. [Compact before/after evidence](../tests/fixtures/cpu-family-head/family-head-before.json)
retains their source hashes and exact diagnostics from the prior clean `9a260404`
binary, production-identical to starting checkpoint `f3bca9a`.

**144 exact stream commands plus 48 separate phase probes** verify check-phase
refusals twice per lane. Pinned checking and C/JS generation match port
check/eval/run and executable C/JS/native compilation: status 1, empty stdout,
full exact stderr and no emitted artifact. Negative execution remains unrun.
These comparisons do not normalize diagnostic text or infer a phase from comments.

Two successful controls use an explicit `List<&2, U32>` instance and a bare
nonparameterized datatype. Their **52 commands** cover eight pure normalizations,
16 checking/phase observations, 12 generation/build operations and **16 actual
port/upstream C-O3/JS executions**. Every normalization and runtime execution returns
`7\n`, with status 0 and empty stderr.
Native API `run_main` for these pure programs performs normalization, not native IO.
The four generated port C/JS artifacts are **byte-identical to baseline**; this is
not an equality claim for every emitted program or native binary. C uses pinned
upstream's actual O3 profile; port C retains strict warnings. All matrix children
are bounded at 30 seconds. No timeout, output discrepancy or nondeterminism occurs.
[Exact commands, streams and hashes](cpu-family-head-results.json).

Unit controls preserve the original error, Rc ownership, term Debug structure, rejected
pointer identity, fresh IDs and one checking step across erased/affine/reusable
demand. An unvisited later hole cannot replace the first error; exhausted display
fuel/depth cannot replace the rejection. All ten fixtures join existing tracked/plain
work and semantic comparisons. The marked-reference unit verifies existing `!`
spelling only; no GPU execution is claimed.

The initial template draft rejected earlier for using an erased parameter live;
its 17-command baseline was incomplete. An exploratory caret spelling was not
accepted source syntax. A first unit assertion mistakenly omitted the existing
marked-reference `!`; only its expectation was corrected. These failed attempts
are retained separately in the compact before receipt and are not final passes.

The clean standard gate passes **1,208 tests / 0 failed / 49 ignored** in 867.71 seconds. Formatting, strict production Clippy, all-feature build and real Node socket-provider checks pass. The 47 CUDA hardware and two optional profiler ignores are unchanged.

Focused validation passes **291 / 0 / 1**, including the new family diagnostics and existing stack, sharing, work/depth, cancellation, provenance, termination and aggregate composition controls. Targeted strict Clippy passes. Focused receipts identify the dirty candidate; the full gate rechecks committed source. Independent source and evidence reviews found no blocker. No assertion, ignore or budget was weakened.

## Census and remaining acceptance work

The renewed 1,302-fixture census measures **366 both accepted, 356 exact rejections, 494 acceptance disagreements, 80 different rejections and 6 upstream host/oracle failures**.

All port statuses and phases are unchanged. Exactly one port stderr stream changes: `check/family_head_bare`, now matching upstream byte-for-byte. The other 1,301 port streams remain unchanged. All 385 historical IO members renew byte-for-byte. **189 measured differences remain outside that exact cohort: 109 acceptance and 80 rejection differences**. IO symptoms are not additional exclusions.

Previous residual: 190. One diagnostic fix is attributable to this port change. Upstream stream/phase transitions versus the preceding run: none. Current host/oracle failures: `check/bystander_hypothesis_cost.bend`, `check/deep_hyp_rewrite.bend`, `page/deep_list_walk.bend`, `reg/borrow_fork_hold.bend`, `reg/dead_param_fork.bend`, `rfc/share_loop.bend`. Historical oracle instability remains unwaived even when an individual current run completes; these are checking-only observations.

[Reconciliation](cpu-family-head-census.json) preserves exact census identities.
[Changed rows](cpu-family-head-census-delta.csv) extend the previous
[refined-type delta](cpu-refined-type-census-delta.csv). First reconstruct that
previous complete CSV through its documented chain; set all engine_commit fields
to this source and apply the current delta. Historical receipts retain their
actual tested identities. Checking acceptance does not establish evaluation or
runtime parity; transient upstream host outcomes are separate from port fixes.

Original fixture linkage rises **458→459 of 1,302**. The **843 unlinked** entries comprise 521 interpreter/compiled candidates, 308 checker negatives, 13 interpreter-only cases and 1 checking-only module. Linkage is not universal parity; authored controls do not inflate inventory coverage.

Nat-first C ordering remains pending/unwaived; approved wildcard observations
remain separate. Compact Nat/string lowering is unadopted. Existing source-hole,
TODO, resource, F32.read, foreign/host, oracle, external-API and supplementary
all-target lint differences remain open. No exception or guard change is added.

The next independent diagnostic gap is `check/forward_data_table`: both reject at checking, but the port reports its internal unknown-datatype error with a type-level label instead of the pinned constructor declaration source/message. Duplicate-constructor negatives (`check/book_prefix_arity`, `check/ctor_shared_tag`) also retain load-phase formatting gaps. These are separately recorded [current census observations](cpu-family-head-next.json), not fixes or new qualifications in this checkpoint.

## Reproduction and preservation

Activate the installed tools and retain the recorded host profile:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib \
  --test cpu_family_heads --test cpu_refined_types --test cpu_self_calls \
  --test kernel_descent_alias
cargo clippy --locked --features extended_observability --test cpu_family_heads -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$UPSTREAM/bend2/main.ts" tests/fixtures/cpu-family-head/check/family_head_bare.bend
"$BEND" check tests/fixtures/cpu-family-head/check/family_head_bare.bend
python3 scripts/checking-census/run.py --reference "$UPSTREAM" \
  --binary "$BEND" --phase-probe "$PROBE" --output "$FRESH_CENSUS" \
  --build-receipt "$BUILD_RECEIPT"
```

Use exact binary/probe identities in [validation](cpu-family-head-validation.json)
and the [census build instructions](../scripts/checking-census/README.md). Matrix
commands use portable placeholders; repeated rows are compacted only after exact
stream equality. The census retains four workers, 30-second upstream and
60-second port/probe bounds. Host Rust debug/incremental overrides are distinct
from Bend runtime optimization profiles.

Minimum recorded free space: **11.155 GiB workspace / 4.499 GiB temporary**. The 11 GiB build preflight and 10 GiB workspace / 1 GiB temporary stop floors remain enforced. Existing cache placement and disabled host debug/incremental data remain in use. No cache or unique historical evidence was deleted.

Compact unique failures, attributed original, controls, goldens and reproducible
receipts are committed. Bulk logs, raw census streams, generated artifacts and
lossless binary snapshots remain local; pushing Git does not preserve them.
No installs, desktop/GPU work, main merge or release occurred.
