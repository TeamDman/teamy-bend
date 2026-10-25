# Invalid-token diagnostics retain Bend source locations

Clean tested source: `d8122bf54fb0879a3307ee1873f5ec607a824e40`. While the separate
[competing-error decision](cpu-failure-order-renewal.md) remains unresolved, this
checkpoint closes an independent, measured source-diagnostic gap. Invalid
identifier starts and EOF already failed loading; they now report the pinned
expected syntax category and Bend source excerpt rather than host context.

Five unchanged originals are qualified: `check/array_default_distinct`,
`check/array_element_data`, `check/array_get_neutral`, `check/axiom_runtime_capture`
and `check/comp_splice_tuple_goal`. Their actual pinned phase is **source load
rejection**, irrespective of older comments about arrays/checking. Four authored
controls cover term/name positions, EOF without a final newline and an imported
failure. [Originals, goldens, compact pre-fix evidence and attribution](../tests/fixtures/cpu-token-expectations/README.md).

Pinned [parse_lexeme](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1564)
expects a name where an identifier must start, while [term parsing](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1961)
expects a term at its fallback. The port reused its low-level name parser for
both but did not attach this source diagnostic metadata. The correction supplies
the appropriate category to the existing optional loader-owned diagnostic boundary.

Public `ParseError` values, token consumption, acceptance, rejection, terms,
sharing and resource accounting remain unchanged. Keyword-specific errors retain
their separate current behavior. The change does not add old array syntax or
weaken any grammar, source-hole or structural guard. A unit regression verifies
tracked and untracked loader errors are identical, including imported errors.
The private-request and runtime demand implementation are untouched.

## Results and boundaries

The clean-source differential matrix passes **162 exact diagnostic observations**:
9 cases × 2 repetitions × (upstream CLI + upstream C/JS generation + six port
entry points). Every observation has empty stdout, exit 1 and exact upstream stderr;
no output artifact is emitted. The six port entries are check/eval/run and
executable C/JS/native compilation. All **54 phase probes** report source-load
rejection separately for upstream, strict port and executable port checking.
There is no execution, successful checking or normalization claim for these invalid
sources. [Current compact evidence](cpu-token-expectations-results.json).

Standard gate: **1,175 passed, 0 failed, 49 ignored** at the clean source above.
Formatting, production strict Clippy, build and Node provider pass. The changed
integration target's strict Clippy passes. Existing failure-order, laziness,
cancellation, stack, sharing and budget regressions run in the gate. The full
checking census and supplementary all-target Clippy were not rerun; their prior
open results are not cleared. Hardware/profiler ignores remain unrun.

An initial imported control used an invalid import spelling and was corrected
before qualification; the initial receipt remains local. The standalone first
integration target emitted unused compilation-helper warnings, so its test was
moved to the existing target that uses those helpers. No warnings were suppressed.
Focused checks passed after relocation. The first full gate then caught a stale
prefix-control golden. Auditing matching legacy goldens found two (`malformed` and
`quantity-declaration`); both newly formatted errors match pinned upstream twice.
Only their expected port strings were renewed, with old bytes retained in
`*.before-port-stderr.json`. The first gate stopped at 745 passed, 1 failed and
48 ignored; remaining tests were unrun in that attempt. Six prefix/reusable tests
and targeted Clippy passed before the final gate renewal. Initial receipts remain
separate. Production code remains the implementation at `197149a`; final gate and
matrices use the golden-update commit above without relabelling earlier results.

Structured CLI refusals return `Ok(CliOutput)` with status 1. Repository production
dispatch reaches `src/lib.rs`, which reads `exit_code()` before emitting output.
The integration tests exercise actual process statuses for all six entry points.
External consumers were not audited: they must inspect `exit_code()` rather than
assuming every `Ok` is success. This extends an existing compatibility caveat to
newly formatted errors; it changes no public signature or plain loader error.

## Reproduction and remaining work

Use the already installed frozen tools and the existing cache. Host overrides:
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_BUILD_JOBS=4`,
`CARGO_INCREMENTAL=0`, `CARGO_NET_OFFLINE=true`, `BEND_GPU=off`; `RUST_MIN_STACK`
was unset. These settings do not change Bend runtime optimization or limits.

```sh
cargo test --locked --features extended_observability --lib syntax::diagnostics::tests
cargo test --locked --features extended_observability --test cpu_pattern_locations \
  invalid_tokens_keep_exact_source_diagnostics_and_failure_status
cargo clippy -p teamy-bend --locked --all-features --test cpu_pattern_locations -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

For each listed case, run `node "$REFERENCE/bend2/main.ts" "$FIXTURE"` and its
`-o <artifact.c>` / `-o <artifact.js>` generation forms, then the six port commands
above; compare the `.stderr.json` string, empty stdout, exit 1 and absent artifact,
twice. The external matrix has a 30-second child timeout; the integration regression
uses 20 seconds. Exact commands, source hashes, guard/disk observations and all gate
receipts are indexed by [validation](cpu-failure-order-renewal-validation.json).

Five checking receipts add scoped coverage, not runtime passes or universal parity.
The historical full census at `d224e79` still records 300 differences outside 385
historical IO exclusions and six upstream stack failures. Do not subtract five
from that historical count or redate it: only this bounded subset was renewed.
All existing semantics/host/resource/foreign, API and supplementary-lint gaps
remain explicit. The competing-order choice is pending; the compact string-pattern
representation remains unadopted and its original still fails the current guard.

Preflight retains an 11 GiB workspace requirement; wrappers stop below 10 GiB
workspace or 1 GiB temporary space. Source and unique historical evidence were
preserved; no caches were deleted this turn. Small before/current failure records
are in Git. Raw logs, complete receipts, generated binaries and compressed snapshots
remain local and are not preserved by Git publication. No installation, desktop,
GPU, dependency/toolchain change, new semantic exception, merge or release occurred.
