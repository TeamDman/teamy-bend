<!-- SPDX-License-Identifier: MPL-2.0 -->
# Final open-law TODO summaries

Implementation and differential source `ca333c470dd227291ad45ab1b846a633fef730eb`
starts from published
checkpoint `62ca1cef5fc71eca5bd139d0a92c07aa73ef23af`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, docs remain
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Dependencies, lockfile, toolchains,
checking/evaluation budgets, structural guards and cancellation boundaries remain
frozen. The successful standard gate uses test-only checkpoint
`6507da388f35a88767d309b14878835bda0cdcec`, with the identical production source tree.
Full CPU parity remains incomplete.

## Pinned contract and correction

The unchanged `check/axiom_warn.bend` and `grade/open_claim.bend` reject correctly,
but the old port exposed `unfilled laws: ...` through its host formatter. Pinned
[book validation](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3708)
counts final unfilled ordinary definitions, excluding native/intrinsic declarations.
The [CLI boundary](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L432)
then refuses incomplete books with:

```text
Error: 1 TODO found.
The code is incomplete, and not a valid proof yet.
```

Counts above one use `TODOs`. This is a global inventory summary: pinned output
has **no Bend location, context or source window**. The port now preserves that
provenance rather than inventing a location for an aggregate.

A shared private error factory derives the unchanged names message and optional
nonzero count from the same final `open` slice. Both existing nonempty-inventory
guards call it after validation. The CLI recognizes only that structured metadata,
checks cancellation first, then uses the existing checked write/flush/status-1 path.
A lookalike ordinary error string cannot acquire this formatting. Zero-count
metadata is absent, and valid zero-inventory books still succeed. No term/source-origin
representation, evaluator, sharing or work accounting changes.

`KernelError`'s message, Display, equality, kind and public function signatures
remain unchanged; presentation metadata stays private and is excluded from error
equality. Earlier live-use, false-proof, signature and source-hole refusals retain
priority. Existing runtime error kinds never acquire inventory metadata.

Upstream counts `book.hols + book.open`; **this correction covers final open-law
inventories, not arbitrary hole/TODO programs**. Three direct, erased and mixed
hole/law controls preserve the port's earlier unfinished-hole refusal, while
upstream reports one/two final TODOs. Those differences remain unwaived. Neither
source-hole guards nor approved runtime demand semantics were weakened.

`invoke` callers must inspect `CliOutput::exit_code()` even when the result is
`Ok`: these additional language summaries now use that existing status-bearing
path. Repository dispatch forwards the result; `src/lib.rs` reads the exit code
before emission and `src/main.rs` forwards it. External invoke-caller audit remains
outstanding. Direct kernel users still receive the original `Err` message.
Documentation now names global summaries explicitly. No new mid-normalization
cancellation facility is claimed.

## Evidence and validation

The two originals are byte-identical to the pin, with Apache attribution. Five
authored controls exercise one/two/three open laws, no main, completed subsets,
repeated imports counted once, and sealed Base exclusions. Their seven exact
stderr goldens contain no source window. See [before/after evidence](../tests/fixtures/cpu-final-todo/final-todo-before.json),
[matrix](cpu-final-todo-results.json), and [validation](cpu-final-todo-validation.json).

The before/after snapshot pair has 51 commands each. All statuses and oracle
phases stay unchanged. Seven port diagnostics become exact pinned summaries;
41 streams are byte-identical. Three source-hole host reports only move their Rust
formatter implementation line after insertion of the new branch; their original
KernelError messages and refusal phases stay unchanged. This is recorded as host
presentation movement, not a fixed Bend diagnostic or a newly waived difference.
The old binary is tested source `d85cab01eae230d423f718220d945c4cd0a25d62`,
production-identical to the starting documentation checkpoint.

| Check | Result | Scope |
| --- | --- | --- |
| Standard repository gate | **1,198 passed / 0 failed / 49 ignored**, 710.16 seconds | Formatting, strict production Clippy, all-feature build, real Node provider and non-Tracy tests. |
| Focused regressions | **338 passed / 0 failed / 1 ignored** | Clean tested source; library and selected prior diagnostic, stack, sharing, work and runtime controls. |
| New focused tests | Two unit tests and three integration tests passed | Also included in the full gate. |
| Targeted strict Clippy | Passed | New integration target and both renewed legacy integration targets; historical supplementary all-target findings remain separate. |
| Differential matrices | **268 commands**, no mismatch in their qualified contracts | Exact summaries, positive execution/normalization and checking-only applicability. |
| Full checking census | **1,302 measured** | Checking statuses/phases/diagnostics only, not runtime parity. |

The 49 standard-gate ignores remain 47 CUDA hardware tests and two optional
profilers. No ignore, assertion or guard was weakened. The successful gate uses clean test-only
checkpoint `6507da388f35a88767d309b14878835bda0cdcec`; production source is identical to the matrix/census commit.

Negative matrix: seven cases × nine CLI/generation boundaries × two repetitions
= **126 exact stream observations**, with **42 separate check-phase probes**.
No negative artifact was emitted or executed. Positive matrix: **76 commands**,
including 26 actual executions (24 port/upstream C/JS plus two native IO), eight
pure normalizations, 18 generations/builds and 24 checking/phase commands. Its
phase records are already included in the command count. Two pure controls
normalize to `7` and generated programs print `7\n`; the sealed IO control prints
`3\n`. Strict proof checking still excludes IO.

Two zero-inventory modules add **24 applicability commands**: eight successful
checking observations and 16 identical `Error: a build needs import Base` emission refusals. The
Base/IO prerequisite is reached before the missing-main boundary, which these
controls do not establish. They have no
executable artifact, and execution remains unrun. Total matrices: **268 commands**.
All repetitions retain identical streams/statuses; zero timeout. Upstream C uses
its actual O3 profile; port C also keeps strict warning flags.

Focused controls verify error identity, zero/nonzero counts, same-text fallback,
cancellation before output, failed writes/flushes, source-hole priority, public
strict/executable error agreement, earlier sourced diagnostics, sharing, template
caches, stack/work/depth guards, aggregate composition and packed words. Source
and evidence were independently reviewed. A malformed authored false-proof draft
was rejected by both parsers and corrected before qualification; the initial
source/receipt is retained separately. No failed draft is counted as passed.

The initial full gate at `ca333c4` stopped with one existing test failure after
597.00 seconds: a list callback snapshot expected host formatter line 34 instead
of 44. Six current host snapshots were renewed from 32 actual commands across the
list/prefix families; every other byte, failure status, stdout and artifact check
is unchanged. Historical snapshots remain untouched. Tests prefer the explicit
current snapshot where supplied and otherwise retain the historical one. Actual
comparisons still use full-byte equality after their existing fixture-path
substitution; no host-line normalization is applied to assertions. Independent
review approved this test-only renewal, and both affected suites pass (seven tests).
The failed gate retains 738 passed / 1 failed / 48 ignored completed tests, with
later groups unrun in that attempt. It is not relabelled as a pass.
See [renewal observations](cpu-final-todo-host-renewal.json).

## Current residuals

The fresh full 1,302-fixture census measures **366 both accepted, 327 exact
rejections, 493 acceptance disagreements, 109 different rejections and seven
upstream host-stack failures**. All port statuses and phases match the preceding
`d85cab01` census. Exactly two raw diagnostic mismatches become exact matches:
`check/axiom_warn` and `grade/open_claim`. See the
[reconciliation](cpu-final-todo-census.json) and [per-fixture CSV](cpu-final-todo-census.csv).

Another 156 port stderr streams change only their coloured host-formatter source
line (`checker_diagnostic.rs:28→38`, `34→44` or `92→102`). Every other byte is unchanged;
those remain raw diagnostic disagreements, not newly normalized matches. An
initial attribution matcher incorrectly expected an uncoloured line/column form
and failed; its output is retained. The corrected classifier verifies the exact
ANSI location and +10 line shift solely to explain changes. The census's raw
exact-match comparisons and counts are untouched.

All 385 historical IO members renew byte-for-byte. **217 measured differences
remain outside that exact set: 108 acceptance and 109 rejection differences**.
The 386 IO-symptom candidates do not define a new exemption: the additional
`reg/cop_let_descent_escape` candidate remains in the residual. Existing triage
labels describe 108 acceptance disagreements, 80 diagnostic-only candidates and
29 other rejection differences; they are not new semantic waivers.

The measured drop from 222 to 217 comprises **two port diagnostic fixes plus three
upstream transitions from acceptance to host-stack failure**:
`flatten/literal_rows_cubic`, `reg/borrow_fork_hold` and `reg/dead_param_fork`.
Supplemental checking-only probes (three each) give respectively one/two/two
acceptances and two/one/one stack failures. Those unstable observations do not
replace the census and qualify no runtime lane. The other four failures remain
`check/bystander_hypothesis_cost`, `check/deep_hyp_rewrite`, `page/deep_list_walk`
and `rfc/share_loop`. Historical cohort and failure receipts are preserved; no
oracle transition is attributed to a port improvement.

Nat-first C ordering remains pending/unwaived. The six approved wildcard
observations remain a distinct historical scope. The previous demand differential
at `208832a` (92 exact / 6 approved / 2 pending, exit 1) is not relabelled as a
current run. Nat/string compact lowering remains unadopted. Existing F32.read,
source-hole/TODO, host/foreign, resource, oracle, external-API and supplementary
all-target lint limits remain explicit. No new semantic exception was introduced.

Applicable inventory linkage increases **451→453 of 1,302** through the two
unchanged originals. The remaining **849** unlinked entries comprise 522
interpreter/compiled candidates, 313 checker negatives, 13 interpreter-only cases
and one checking-only module. Authored/noninventory ledger strings are not counted.
Historical receipts retain their actual commits. Links and passing test counts do
not establish complete CPU parity under the unchanged acceptance criterion.

The next demonstrated independent family is structural-recursion refusal
provenance: `check/cop_circular_witness_000`, `check/descent_field_order` and
`check/partial_self_call_stuck`. Both sides reject during checking, but the port
still exposes its internal self-call message instead of the pinned expected/
observed diagnostic and source window. [Compact current failures](cpu-final-todo-next.json)
are saved. A future correction must preserve left-to-right descent, sharing and
all structural/work guards; no termination-policy change is proposed here.

## Reproduction and preservation

Activate the installed tools, then use the existing frozen host profile:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --features extended_observability --test cpu_final_todo
cargo test --locked --features extended_observability --lib \
  --test cpu_unfilled_laws --test cpu_template_occurrences \
  --test cpu_definition_provenance --test cpu_pattern_locations \
  --test kernel_open_closures --test runtime_review --test runtime_packed_words \
  --test compiler_executable_c_continuations
cargo clippy --locked --features extended_observability \
  --test cpu_final_todo -- -D warnings
cargo clippy --locked --features extended_observability \
  --test cpu_list_for_each --test cpu_prefix_markers -- -D warnings
node "$UPSTREAM/bend2/main.ts" tests/fixtures/cpu-final-todo/check/axiom_warn.bend
"$BEND" check tests/fixtures/cpu-final-todo/check/axiom_warn.bend
"$BEND" compile tests/fixtures/cpu-final-todo/positive/filled.bend \
  --executable --target c --output "$RUN/port.c"
"$TEAMY_BEND_CC" -std=c11 -O3 -Wall -Wextra -Werror -pedantic \
  "$RUN/port.c" -lpthread -lm -o "$RUN/port"
"$RUN/port" --threads 1 --gpu off
python3 scripts/checking-census/run.py --reference "$UPSTREAM" \
  --binary "$BEND" --phase-probe "$PROBE" --output "$FRESH_CENSUS" \
  --build-receipt "$BUILD_RECEIPT"
```

`BEND` and `PROBE` must match the clean tested commit; see the recorded binary,
library/fingerprint and phase-probe identities in the validation receipt and the
[census instructions](../scripts/checking-census/README.md). Compact matrices use
`$REPO`, `$UPSTREAM`, `$RUN`, `$EVIDENCE` placeholders for exact commands and record
artifact hashes. Equality is verified before repeated rows are compacted. Matrix
children use 30-second bounds; the census retains four workers, 30-second upstream
and 60-second port/probe bounds. Rust debug/incremental overrides bound build-cache
growth and are distinct from Bend runtime optimization profiles.

Minimum measured headroom was **11.228 GiB workspace / 4.623 GiB temporary**.
The existing 11 GiB workspace preflight and 10 GiB workspace / 1 GiB temporary stop
floors remain enforced. Existing debug-cache placement and disabled Rust debug/
incremental data keep build growth bounded. No cache or unique evidence was deleted;
frozen source, dependency, lockfile and toolchain checks remain intact.

Unique compact failures, fixtures, goldens, reproducible commands and census
classifications are committed. Bulk raw logs/streams, generated binaries, draft
sources and lossless build snapshots remain local and **are not preserved by Git
pushing**. No installs, desktop/GPU work, main merge or release occurred.
