# Missing-import diagnostic checkpoint

Tested source: `4d62ceac607c4050c1d53ded536feb837118bf89`; baseline checkpoint
`300aefe146862d3043cd260cdf20504b75c9e9c1` (tested source
`ec91a1634d410781623961607b9cf85aab2e5f87`). Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; teamy-docs remains
`50e89d6897ce2a65167ef65b17c5fa83fe849158`.

## Correction and applicability

Five unchanged originals now produce the pinned missing-file diagnostic and
immediate importing source window: `import/cross_file_io`, `cross_file_proof`,
`cycle_terminates`, `diamond_dedup` and `path_canonical`. All already rejected at
load with exit 1. Their absolute dependencies are absent in the pinned checkout;
fixture comments do not establish successful IO, proof chaining, cycle handling
or diamond execution. These originals remain execution-unrun.
[Pinned loader](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1007).

The existing recursive loader now borrows an immediate import site. Only its
existing canonicalization failure, with `ErrorKind::NotFound`, can attach optional
private diagnostic metadata. The non-inline formatter retains the authored
lexical target and raw importer window, including a nested importing module or
dangling symlink. It does not register raw text in the first-write-wins parser
source map or add another filesystem lookup. Outer unwinding does not replace
inner attribution. Canonicalization still precedes the 64-level import guard,
cycle and seen checks. Missing-root, non-directory and read failures preserve
the public loader error and previous failure order.

Eligible POSIX spellings exclude dot/empty components, backslashes and colons;
ambiguous hashes, Unicode whitespace and U+FEFF keep the existing fallback.
Ordinary relative and nested-relative paths, dangling-file links and lexical
parent-directory links have exact Linux oracle evidence. Synthetic Windows/drive
spellings exercise fallback only; Windows execution is unrun. No normalization
policy or semantic exception is introduced. Terms, sharing, evaluation, guards,
charging, runtime demand and cancellation boundaries are unchanged. This does
not add mid-normalization cancellation.

Public `ParseError` fields/message/Display and public loader signatures stay
unchanged. Private optional metadata/Debug and user-visible CLI formatting change.
These newly formatted refusals use existing `Ok(CliOutput)` with `exit_code = 1`
instead of the host-error `Err` fallback. Production repository dispatch forwards
that output, and `src/lib.rs` captures its status before writing. External invoke
consumers remain unaudited: `Ok` is not evidence of exit success.

## Validation

Standard gate: **1291 passed / 0 failed / 49 ignored**, 1052.681 seconds, at the exact clean source above.
It renews prior host-stack, shared-reference, aggregate-work, partial-application,
constructor/import, guard, writer/flush and cancellation regressions. Focused runs
before the source checkpoint passed 66 integration tests; the final expanded
missing-import target passed three tests. Six new unit controls, four earlier
header controls and one status/writer boundary control also passed. The full gate
renews their final source. An initial targeted Clippy failure for passing the
borrowed import site by value was corrected; final strict targeted Clippy passed
without a warning suppression.

[Compact complete observations](cpu-missing-import-diagnostics-results.json.gz):

- 272 main-matrix commands, 130 deterministic within-lane pairs, zero unexpected
  outcomes. Ten negatives have 180 exact CLI stream observations and 40 separate
  strict/executable load-phase probes. No refused-source artifact was emitted.
- Two valid import programs return `9` and `18`. The latter actually loads a
  shared dependency through a diamond. There are 16 generated C/JS executions,
  eight pure-normalization observations, 16 checking/phase commands and 12
  generation/build commands. Four port artifacts are byte-identical to baseline.
  Native pure run is normalization, not IO engine execution. Integration controls
  compare complete normalized structures separately from printed text.
- 150 adjacent observations, 75 deterministic pairs: four additional path/link
  cases exactly match upstream; nine fallback controls preserve baseline bytes.
  Dot/parent/double-slash, hash/BOM, actual cycle, missing root and non-directory
  diagnostics remain non-exact. Reading a directory remains an upstream host
  failure. Matching refusal phases do not waive those differences.
- 30 direct observations at unchanged upstream original paths, including ten
  phase probes. The original copies and source hashes are verified. All 15
  direct within-lane repetition pairs agree.

All **452 observations / 220 within-lane pairs** are retained with commands,
status, streams, timing and raw-stream hashes. Only machine roots are replaced
in published text. Dynamic symlink and directory topology is retained for the
adjacent controls. No subprocess timed out; each differential child was bounded
at 30 seconds. C uses the upstream CLI profile `-std=c11 -O3 -lpthread -lm`;
port C additionally uses `-Wall -Wextra -Werror -pedantic`.
Independent review verified source, binary/probe/library, oracle identities,
all comparisons and artifact equality. [Validation receipt](cpu-missing-import-diagnostics-validation.json).

Reproduce repository controls after existing tool activation, from this checkout:

```sh
export PYTHONDONTWRITEBYTECODE=1 CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_BUILD_JOBS=4 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --offline --features extended_observability --lib missing_import -- --nocapture
cargo test --locked --offline --features extended_observability --lib import_header -- --nocapture
cargo test --locked --offline --features extended_observability --test cpu_missing_import_diagnostics --test cpu_import_header_diagnostics --test cpu_template_occurrences --test kernel_core --test kernel_normalization --test kernel_open_closures
cargo clippy --locked --offline --features extended_observability --lib --test cpu_missing_import_diagnostics -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Per-oracle commands are in the compact result receipt; the full checking census
uses [the existing adapter](../scripts/checking-census/README.md). Rust 1.96.0,
formatter nightly-2026-05-24, Node24.19.0, Clang19.1.7 and PS7.5.4 are unchanged.
The host debug/cache overrides above are explicit; they do not change Bend runtime
optimization or resource budgets. Dependencies, lockfiles and pins remain frozen.

## Census and remaining requirements

The complete 1,302-fixture census measured 366 both accept, 401 exact rejections,
494 acceptance disagreements, 35 different rejections and six oracle failures.
All 385 historical IO-cohort observations renewed exactly; they remain explicitly
classified strict-proof differences. Outside that cohort, 144 differences remain:
109 acceptance + 35 rejection. Only the five targeted port stderr streams changed;
every port status and phase is unchanged. The upstream dead_param_fork host-failure
phase changed without changing its oracle-failure classification. It is not a port
improvement. No discrepancy is waived.
[Reconciliation and compact changes](cpu-missing-import-diagnostics-census.json)
and [six-row delta](cpu-missing-import-diagnostics-census-delta.csv) retain actual
source identities. Both-accept counts are checking status agreement only.

Coverage linkage: 509 linked / 793 unlinked. Remaining unlinked applicability: 260 checker rejection, 520 interpretation and compiled candidates, 1 checking without main, 12 interpretation only.
A linked load-diagnostic receipt is not runtime qualification or universal parity.
CPU parity remains incomplete. The previously approved demand contract remains
intact; nat_first ordering is pending and unwaived. Compact Nat/string lowering
is unadopted. Base coordinates, external invoke audit, historical supplementary
all-target Clippy failures, and recorded source-hole, F32, resource, foreign and
oracle limitations remain open. Supplementary all-target Clippy, invalid-source
execution, desktop/GPU and newer upstream adoption are unrun in this batch.

Next measured independent batch: malformed-literal load diagnostics in
parse/hex_no_digits, invalid_string_escape, unclosed_char_code and
word_overflow_literal. Current original invocations reject at load with exit 1
in both implementations, but wording and/or source coordinates differ. Their
correction and adjacent valid-literal execution are not completed by this receipt.

## Preservation

No additional cleanup was needed. All 21 recorded storage-preservation files
retain their start-of-batch hashes, including archives and restoration manifests.
The previous verified 1,287,000,064-byte workspace recovery remains historical
evidence; this batch claims no further reclaimed bytes. The unchanged 11 GiB
preflight, 10 GiB active workspace floor and 1 GiB temporary floor are enforced.
Gate minima: 12.178688 GiB workspace and 2.476 GiB temporary storage.

Compact sanitized receipts and source are preserved on the feature branch.
Bulk raw logs, binary snapshots, private backups and storage report/extension CSV
remain local and must accompany an environment transfer. Historical archived
binary paths require selective restoration using the preserved manifest/tool;
full restoration requires enough capacity. The two Library upload attempts failed
before creation; no Library IDs exist. No upload was retried in this batch.
