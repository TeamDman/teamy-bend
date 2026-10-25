<!-- SPDX-License-Identifier: MPL-2.0 -->
# Unknown datatype diagnostics and constructor provenance

Tested source `5013562f98204d5e4694486d0a3f77f366345b7d` starts from published
`b7c18b0de45fa292fab9d07f01a86c2c209ae557`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, docs remain
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Dependencies, toolchains, types,
checking, evaluation, termination, work/depth guards and cancellation boundaries
are unchanged. Full CPU parity remains incomplete.

## Contract and correction

The unchanged `check/forward_data_table.bend` already rejects in both checkers.
Its constructor field names a datatype declared later. Pinned upstream reports
`a declared datatype (unknown: D3)` at `MkTwo`; the port previously exposed
`Two: undefined datatype D3` through host error formatting.

[book_adt](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L971)
rejects absent names and names whose declarations are not datatypes, using a
message-only diagnostic and the original ADT span.
[Datatype inference](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3368)
performs this lookup before checking arguments.
[Constructor checking](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3684)
uses the constructor's label for its field telescope.

The port now attaches the existing structured diagnostic only to an unknown-name
failure at that inference call. The existing constructor-label propagation then
provides the pinned label and source window. No new source metadata, copied terms,
lookup behavior or argument evaluation is introduced. The shared datatype helper,
its other callers, wrong-arity failures and unknown-exclusion failures are unchanged.
Forward declarations remain unavailable; the original's later duplication code is
not reached and is not being qualified by this rejection.

**Public compatibility:** `KernelError.message`, `Display`, equality and shape are
unchanged. Crate-private diagnostic metadata is now present for this rejection;
derived `Debug` includes it and can differ. Public CLI stderr intentionally changes
to the pinned message/context/source window. These formatted failures use the
existing status-bearing `Ok` output path where the previous unstructured error
propagated as `Err`; programmatic `invoke` consumers must inspect `exit_code()`.
The CLI process still exits 1. No public signature/status-path redesign is included, and
the external invoke-caller audit remains outstanding. Missing source/presentation
or cancellation/output-write failure still returns `Err`. No mid-normalization
cancellation capability is claimed.

## Evidence and controls

Eight negatives cover the original, a later local declaration, nested list field,
dependent constructor context, imported helper, template substitution, an ordinary
definition used as a datatype, and an unknown outer datatype before an invalid
argument. The [compact before/after receipt](../tests/fixtures/cpu-forward-data/forward-data-before.json)
preserves source hashes and exact failures from the prior clean `c7519a0` binary,
production-identical to starting checkpoint `b7c18b0`. All eight already rejected.

**144 exact stream commands and 48 separate phase probes** verify each case twice
across pinned checking/C/JS generation and port check/eval/run/executable C/JS/native
compilation. They agree on check-phase status 1, empty stdout, full exact stderr and
no emitted artifact. No diagnostic normalization is used. Negative execution is
unrun, not a runtime pass.

Two positive controls place a datatype before its use and refer recursively to the
datatype currently being declared. Their **52 commands** cover eight pure
normalizations, 16 checking/phase observations, 12 generation/build operations and
**16 actual port/upstream C-O3/JS executions**. Every normalization and runtime
execution returns `7\n`, status 0 and empty stderr. Native API `run_main` for these
pure programs performs normalization, not native IO. Four generated port C/JS files
are **byte-identical to baseline**; that claim is limited to these controls. C uses
the pinned upstream O3 profile; port C retains strict warnings. All matrix children
are bounded at 30 seconds, with no timeout, discrepancy or nondeterminism.
[Exact commands and hashes](cpu-forward-data-results.json).

Unit controls retain internal errors, term Debug/Rc identity, rejected pointers,
fresh IDs, demand behavior and one checking step. Unknown names do not inspect
later invalid arguments. Existing known-family arity/exclusion failures retain
their original errors. Message-only formatting with an empty context requires no
normalization; it can succeed with no remaining normalization allowance. A context
that needs normalization correctly falls back to the original rejection when
fuel/depth is exhausted. No synthetic budget charge or increased limit is added.
All ten fixtures also enter the existing tracked/plain source/work/evaluation checks.

One initial authored positive omitted the required U32 annotation on `+`, which
defaults to Nat. Both upstream and the old port reject it. The initial emission run
and focused command (**286 passed / 1 failed / 1 ignored**, exit 101) remain failed
evidence; they are not a passing gate. Only the fixture expression was corrected
to `(x + total(xs) : U32)`. The compact receipt preserves its original source and
both diagnostics. Earlier successful artifact emissions were reused by identity;
no historical failure was overwritten.

The clean standard gate passes **1,212 tests / 0 failed / 49 ignored** in 879.95 seconds. Formatting, strict production Clippy, all-feature build and real Node socket-provider checks pass. The 47 CUDA hardware and two optional profiler ignores are unchanged.

Corrected focused checks pass **10 / 0 / 0** (nine integration tests and the tracked/plain provenance test), and targeted strict Clippy passes. The initial failed command's successful library tests are not relabelled as a clean gate. The full gate rechecks final committed source and earlier stack, sharing, work/depth, cancellation, provenance, termination and aggregate composition controls. Independent source/evidence review found no blocker. No assertion, ignore or budget was weakened.

## Census and remaining requirements

The renewed 1,302-fixture census measures **366 both accepted, 357 exact rejections, 495 acceptance disagreements, 79 different rejections and 5 upstream host/oracle failures**.

All port statuses and phases are unchanged. Exactly one port stderr stream changes: `check/forward_data_table`, now byte-for-byte equal to upstream. The other 1,301 port streams remain unchanged. All 385 historical IO members renew byte-for-byte. **189 measured differences remain outside that exact cohort: 110 acceptance and 79 rejection differences**. IO symptoms are not additional exclusions.

Previous residual: 189. One diagnostic fix is attributable to this change. Separately, `reg/borrow_fork_hold` moved from an upstream load-phase stack failure to upstream acceptance on this run; the port still refuses its Nat literal above the unchanged 128 parser guard. Thus 189 minus one port diagnostic fix plus one newly measurable acceptance disagreement remains 189. This is retained oracle instability, not a new port regression, completed runtime lane or approved resource exception. Upstream stream/phase transitions versus the preceding run: `reg/borrow_fork_hold.bend`. Current host/oracle failures: `check/bystander_hypothesis_cost.bend`, `check/deep_hyp_rewrite.bend`, `page/deep_list_walk.bend`, `reg/dead_param_fork.bend`, `rfc/share_loop.bend`. Historical instability remains unwaived even when a particular run completes; these are checking-only observations.

[Reconciliation](cpu-forward-data-census.json) preserves raw census identities.
[Changed rows](cpu-forward-data-census-delta.csv) extend the prior
[family-head delta](cpu-family-head-census-delta.csv). Reconstruct that previous
complete CSV through its documented chain, set all engine_commit fields to this
source, then apply this delta. Historical receipts retain their actual tested
commits. Checking agreement is not evaluation/runtime parity; oracle instability
is separate from port improvement.

Original fixture linkage rises **459→460 of 1,302**. The **842 unlinked** entries comprise 521 interpreter/compiled candidates, 307 checker negatives, 13 interpreter-only cases and 1 checking-only module. Linkage is not universal parity; authored controls do not inflate inventory coverage.

Nat-first C ordering remains pending/unwaived. Approved wildcard observations keep
their separate scope. Compact Nat/string lowering is unadopted. Earlier source-hole,
TODO, resource, F32.read, foreign/host, oracle, external-API and supplementary
all-target lint differences remain open. No new semantic exception is introduced.

Next independent gaps share duplicate-constructor load diagnostics: `check/book_prefix_arity`, `check/ctor_shared_tag`, `compile/ctor_tag_dispatch_000` and `_001`. All reject during loading in both implementations; their full diagnostics differ. Compile-folder placement does not make them executable positives. [Current observations](cpu-forward-data-next.json) preserve their actual phases and original hashes; no correction or runtime qualification is claimed here.

## Reproduction and preservation

Activate installed tools and retain the recorded host build profile:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib \
  source_origins_preserve_checker_work_terms_and_successful_evaluation
cargo test --locked --features extended_observability \
  --test cpu_forward_data --test cpu_family_heads --test cpu_refined_types \
  --test kernel_descent_alias
cargo clippy --locked --features extended_observability --test cpu_forward_data -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$UPSTREAM/bend2/main.ts" tests/fixtures/cpu-forward-data/check/forward_data_table.bend
"$BEND" check tests/fixtures/cpu-forward-data/check/forward_data_table.bend
python3 scripts/checking-census/run.py --reference "$UPSTREAM" \
  --binary "$BEND" --phase-probe "$PROBE" --output "$FRESH_CENSUS" \
  --build-receipt "$BUILD_RECEIPT"
```

Use exact binary/probe identities in [validation](cpu-forward-data-validation.json)
and the [census instructions](../scripts/checking-census/README.md). Matrix commands
use portable placeholders; repetitions are compacted only after exact stream
comparisons. The census retains four workers, 30-second upstream and 60-second
port/probe bounds. Rust host debug/incremental overrides are distinct from Bend
optimization profiles.

Minimum recorded free space: **11.131 GiB workspace / 4.458 GiB temporary**. The 11 GiB build preflight and 10 GiB workspace / 1 GiB temporary stop floors remain enforced. Existing cache placement and disabled host debug/incremental data remain in use. No cache or unique historical evidence was deleted.

Compact unique failures, attributed original, controls, goldens and reproducible
receipts are committed. Raw logs/census streams, generated artifacts and lossless
binary snapshots remain local and are not preserved by Git pushing. No installs,
desktop/GPU work, main merge or release occurred.
