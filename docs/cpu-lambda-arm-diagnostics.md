<!-- SPDX-License-Identifier: MPL-2.0 -->
# Numeric lambda-arm diagnostics and renewed checking census

Clean tested source: `59ac3fdbdd900be23e6516a3da84bafa31e7574a`. The two requested diagnostic gaps and three
adjacent unchanged originals are fixed. This is a source-diagnostic correction;
it does not add numeric lambda matching or change acceptance/evaluation semantics.
Full CPU parity remains incomplete. The `nat_first` C ordering exception remains
pending and unwaived; compact-pattern research remains unadopted.

## Pinned cause and bounded correction

Pinned [lambda parsing](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1813)
consumes a colon only when its preceding expression is a variable or reference.
A number, constructor, lambda, annotation or nonempty application becomes the
fallback term, after which [the parser expects `}`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1822).

For `\{0: 9}`, the port already rejected correctly but consumed `:` first, then
reported its constructor-name error at the following token. It now attaches the
pinned closing-brace diagnostic at that consumed colon, then returns the original
public error. The error message, post-colon cursor, term nodes, sharing, parsing
work, source-hole refusal and safety guards are unchanged. Named arms and fallbacks
are untouched. `GpuRef` retains its existing separate refusal and is unqualified
here because upstream represents that marker differently; no new exception is added.

All five originals remain byte-identical: `check/erased_match_numeric`,
`check/undo_residual_lane`, `flatten/statement_beta`, `flatten/statement_lam_absorb`
and `run/binop_probe_split`. Nine authored negatives cover multiline/imported
colons, constructor/function/application/annotation heads, a bad head after a valid
arm, and missing syntax. Before the fix, twelve diagnostics differed and two
already matched. [Sources, attribution and compact pre-fix evidence](../tests/fixtures/cpu-lambda-arm-diagnostics/README.md).

Independent source review found no blocking issue. Tests preserve tracked/plain
public errors and explicitly distinguish the post-colon public location from the
colon-anchored diagnostic. The shared compile-bound reference control preserves
loading and its existing checking refusal; it is not an execution pass. Initial
authored negatives missing required return annotations were corrected before
qualification; the failed attempts remain separate private evidence.

## Qualified results

| Check | Result |
| --- | --- |
| Full standard gate at the clean source | **1,181 passed / 0 failed / 49 ignored** |
| Focused unit and parser/checker/provenance targets | 9 + 68 passed |
| Strict Clippy for the changed integration target | Passed |
| Fourteen negative cases, twice | **252 exact diagnostic observations; 84 load-phase probes** |
| Four positive controls, outputs 7 / 3 / 4 / 7 | **40 executable runs** across native, port C/JS and upstream C/JS |
| Pinned interpreter / strict evaluation / strict checking | 8 / 8 / 8 successful observations |
| Generation / C compilation | 16 generations / 8 successful builds |
| Positive phase probes | 24 accepted |
| Before/current C/JS artifacts | All eight files byte-identical for these controls |

Negative observations cover upstream CLI/C/JS generation and six port entry points:
check, eval, run and executable C/JS/native compilation. Every refusal has exit 1,
empty stdout, exact pinned stderr and no emitted artifact. Rejected sources never
execute. Positive controls distinguish full normalized/data constructor graphs,
printed output and execution. Named keys, empty calls and parenthesized keys retain
their semantics; numeric and function fallbacks without colons remain valid.
All bounded matrix repetitions were deterministic; no timeout occurred. C uses the
pinned CLI's C11/O3 profile, with strict warnings added for port C. No O0 or hardware
claim is made. [Commands, streams and hashes](cpu-lambda-arm-diagnostics-results.json).

The full gate retains prior packed-word, sharing, stack, cancellation, work/depth,
false-proof and source-hole regressions. This adds no mid-normalization cancellation.
Supplementary all-target Clippy and its prior failures were not rerun. Hardware/
profiler ignores, older host/foreign/resource/F32.read/hole differences and external
API caveats remain explicit.

Formatted CLI refusals retain the existing status-bearing `Ok(CliOutput)` boundary.
Actual process failure is tested at all six entries; production dispatch reads
`exit_code()`. External consumers were not audited and must inspect that status.
No public API signature or plain loader error changed.

## Full census: measured current totals

The complete unchanged **1,302-fixture** inventory was remeasured at this source,
using the existing four-worker checking runner and freshly built matching probes.
It does not execute or normalize `main`.

| Checking observation | Current count |
| --- | ---: |
| Both accept | 366 |
| Exact rejection: phase/status/stdout/stderr | 318 |
| Acceptance disagreement | 494 |
| Differing rejection | 118 |
| Upstream oracle failures | 6 |

All 1,302 port checking statuses are unchanged from `d224e79`. All **385 historical
IO-exclusion observations renewed exactly**, including fixture hashes, phases,
statuses and raw streams; subtraction is guarded by assertions before reporting.
The extra IO diagnostic candidate outside that historical set is not silently waived.
Current non-IO differences are therefore **227: 109 acceptance disagreements and
118 differing rejections**, plus the six separate oracle failures. Historical triage
labels split the 227 into 109 acceptance, 86 diagnostic candidates and 32 other
rejection differences; labels do not establish semantic equivalence.

Compared with the previous full census, **73 differing rejections now match exactly**.
That is cumulative progress since `d224e79`, not 73 fixes from this turn; five are
this batch's originals. One other port diagnostic changed but remains mismatched:
`parse/prefix_operator_dead` now reports a term at `-`, while upstream expects a name
at `5`. No acceptance/status change is hidden by these diagnostic improvements.

The six oracle failures remain unqualified. `reg/dead_param_fork` changed upstream
failure phase from checking to loading, with unchanged exit 1 and stack-failure
stderr. Supplementary probing exposed instability: the first attempt retained only
an `accepted` phase sidecar before an over-specific harness assertion stopped it;
its exit/streams were not saved, so it is **phase-only evidence, not a recorded pass**.
Five subsequent fully recorded probes all failed with exit 1 and identical stack
stderr, once during checking and four times during loading. They do not replace the
census's failed observation or establish an upstream semantic cause.

[Current per-fixture table](cpu-lambda-arm-census.csv) and [counts, deltas and compact
changed-failure receipts](cpu-lambda-arm-census.json) preserve that distinction.
Historical census files retain their original source identities. A census match or
both-accept result does not grant normalization/executable parity or repeated-fixture
qualification. Family links remain a separate measure: **443 linked / 859 unlinked**
(322 checker negatives, 523 compiled candidates, 13 interpreter profiles and one
checking-only module).

## Reproduction, preservation and next decision

Existing host build overrides: `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_BUILD_JOBS=4`, `CARGO_INCREMENTAL=0`,
`CARGO_NET_OFFLINE=true`, `BEND_GPU=off`; `RUST_MIN_STACK` unset. They do not change
Bend runtime optimization or safety budgets. Rust 1.96.0, formatter
nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and the reference pin remain frozen.

```sh
cargo test --locked --offline --features extended_observability --lib syntax::diagnostics::tests
cargo test --locked --offline --features extended_observability \
  --test cpu_pattern_locations --test cpu_template_occurrences --test kernel_core --test parser
cargo clippy --locked --offline --all-features --test cpu_pattern_locations -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
python3 scripts/checking-census/run.py --reference "$UPSTREAM" --binary "$BEND" \
  --phase-probe "$PHASE_PROBE" --output "$FRESH_OUTPUT" --build-receipt "$BUILD_RECEIPT"
```

[Census build instructions](../scripts/checking-census/README.md) and
[validation receipts](cpu-lambda-arm-diagnostics-validation.json) give exact commands,
binary/library/source hashes and actual precommit versus clean-source checks.
Matrix child limit: 30 seconds; integration: 20 seconds; census upstream: 30 seconds,
port/probe: 60 seconds; outer gate/census: 1,200 seconds each. The gate and census
completed in roughly 794 and 265 seconds. Workspace preflight stays 11 GiB; stop
floors stay 10 GiB workspace and 1 GiB temporary storage. No cache, source or unique
historical evidence was deleted.

No new semantic decision was required for this fix. The [pending `nat_first`
choice](cpu-failure-order-renewal.md) remains unwaived. The renewed census should guide
subsequent applicability/semantic work; the retained unary-prefix diagnostic is a
concrete independent candidate, while acceptance mismatches still require mode/host/
resource classification rather than blanket exceptions. Compact-pattern research
remains unadopted. No dependency/toolchain/pin changes, installs, desktop/GPU work,
main merge or release occurred. Compact receipts are committed; raw logs, complete
process captures and compressed binaries remain local and outside Git preservation.
