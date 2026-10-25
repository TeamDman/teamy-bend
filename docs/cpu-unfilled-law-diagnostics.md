<!-- SPDX-License-Identifier: MPL-2.0 -->
# Live unfilled-law diagnostics

Clean tested source `d85cab01eae230d423f718220d945c4cd0a25d62`; implementation
`1348ad375b50e30ced6ef5b7f8b251576e8ecf49`. The second commit only satisfies two
strict test lint rules. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; docs remain
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. This checkpoint closes a diagnostic
family, not full CPU parity. Dependencies, lockfile, toolchains and budgets are
unchanged.

## Correction and contract

The six unchanged originals in `check/assert_fill_scope`, `check/axiom_use`,
`check/axiom_import_warn`, `check/axiom_whitelist`, `check/forward_reference_scope`
and `compile/axiom_runtime` already rejected live use of an unfilled law. The port
returned the internal message through the host error formatter, losing the pinned
expected/observed diagnostic, Bend context and source window.

The [pinned Ref inference rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3271)
requires a filled definition for live references and reports that reference's
source. The fix attaches optional diagnostic metadata at the existing rejection
in `Engine::infer_reference`. It uses existing per-use origins; it neither copies
reference nodes nor adds term metadata. The original error message, equality,
first-error ordering, erased demand, structural self-call checks and sealed
foreign/numeric/opaque exemptions stay intact. Parser types and load contracts
are unchanged. Diagnostic rendering uses the existing cloned engine with the
remaining budget and falls back to the original error if display work exhausts.

Eleven goldens cover the six originals and five authored multiline, pattern,
actual-import, template and shared-reference controls. See the
[before/after evidence](../tests/fixtures/cpu-unfilled-law/unfilled-law-before.json)
and [matrix](cpu-unfilled-law-results.json). The old snapshot is source
`208832a8282a7e1688aa725341cdf9ea0b5447c0`, production-identical to the starting
`69fb601c8726157233b5d0f2d1ffbf9084a744e6` documentation checkpoint. All 54
before/after commands keep their status and oracle phase: exactly 11 port-check
stderr observations change to pinned bytes, and the other 43 observations match.
All eight copied upstream sources remain byte-identical to the pin.

Repository production `invoke` dispatch forwards results; `src/lib.rs` captures
`CliOutput::exit_code()` before emission and `src/main.rs` forwards it. Six CLI
boundaries require status 1, empty stdout, exact stderr and no output artifact.
The existing sourced-diagnostic API caveat now applies to these additional errors:
a Rust `invoke` caller can receive `Ok(CliOutput)` carrying status 1 instead of
`Err`. External consumers have not been audited. Kernel error identity/Display is
unchanged; unsourced or display-exhausted failures retain the host-error fallback.
No mid-normalization cancellation facility is added or claimed.

## Validation

| Check | Result | Scope |
| --- | --- | --- |
| Standard gate | **1,193 passed / 0 failed / 49 ignored**, 856.54 seconds | Formatting, strict production Clippy, all-feature build, real Node provider and non-Tracy tests. |
| Focused regressions | 334 passed / 0 failed / 1 ignored | Library, provenance, pattern, closure, continuation, packed-word and runtime controls; earlier candidate with identical production source. |
| New integration tests | 2 passed / 0 failed | Exact six CLI boundaries and successful definition controls; also renewed in full gate. |
| Targeted strict Clippy | Passed | New integration test target; historical unrelated all-target lint failures remain unwaived. |
| Differential negatives | 198 exact diagnostics + 66 phase probes | Eleven cases, two repetitions, nine CLI/generation boundaries. |
| Differential positives | 128 commands, no mismatch | 42 executions, 16 normalizations, 30 generation/build, 40 checking/phase. |
| Full checking census | 1,302 measured | Phase/status/diagnostic qualification only; not runtime parity. |

The 49 full-gate ignores remain the existing 47 CUDA hardware tests and two
optional profilers. No ignore or safety assertion was weakened. See the
[validation receipt](cpu-unfilled-law-validation.json).

The 198 negative observations are eleven cases × two repetitions × nine lanes:
upstream check and C/JS generation refusal, port check/eval/run and executable
C/JS/native generation refusal. The 66 separate probes establish check-phase
rejection in upstream, strict port and executable port modes. No negative
executable was emitted or run. Status-only agreement is not substituted for
byte-exact diagnostic comparison.

The positive matrix contains 128 commands: 42 actual executions (40 port/upstream
C/JS and two native IO), 16 pure-normalization observations, 30 generations/builds
and 40 checking/phase commands. Four filled/erased/recursive/imported entries
normalize to `7`, and their generated programs print `7\n`. The sealed Base
numeric/IO control prints `3\n`; strict proof mode still excludes IO. Native
`run_main` on the four pure entries uses normalization, not the IO interpreter.
All repeated streams/statuses are deterministic; no timeout. Primary upstream C
uses its actual O3 profile; port C additionally keeps strict warning flags.

Focused library and integration controls cover shared reference identity, template
cache behavior, default-stack guards, exact earlier diagnostics, aggregate
composition, packed words, first errors, work/depth budgets and existing
cancellation boundaries. New unit coverage checks unchanged Rc ownership, fresh
IDs and error equality, erased demand and structural recursion. Fifteen additional
tracked/plain source-origin cases compare trees, checker fuel/depth/fresh IDs and
successful normalized results. Independent source and evidence reviews completed.

Initial authored controls had four syntax mistakes, corrected after both checkers
rejected them; draft sources/receipts remain local. The first targeted Clippy run
failed only on two authored test-style lints; both were corrected without weakening
assertions. An initial private positive-runner attempt used a nonexistent fixture
root; it qualified no case and is retained separately from the completed renewal.

## Remaining scope

The renewed full 1,302-fixture census at the tested source measures **366 both
accept, 325 exact rejections, 111 different rejections, 496 acceptance
disagreements and four oracle failures**. All 1,302 port statuses and phases are
unchanged from the historical A1 census; exactly six port stderr streams change
to exact pinned matches. See [reconciliation](cpu-unfilled-law-census.json) and
[per-fixture categories](cpu-unfilled-law-census.csv).

All 385 historical IO members renew byte-for-byte. Outside that exact set, **222
measured differences remain: 111 acceptance and 111 rejection differences**. The
386 IO-symptom candidates are not interchangeable with the historical set:
`reg/cop_let_descent_escape` remains in the residual. The count is not an
all-non-IO classification and does not prove all rejection differences are merely
wording. Old triage labels among the current residual are 109 acceptance/rejection
mismatches, 31 other rejection differences, 80 diagnostic-only candidates and two
new/unclassified observations; labels are observations, not waivers.

The reduction from historical 227 to current 222 combines six corrected
diagnostics with one changed **upstream** observation. `reg/borrow_fork_hold`
accepted in this census after a prior upstream host-stack failure; the port is
unchanged. Together with `reg/dead_param_fork`, it remains unstable oracle evidence:
three supplemental checking-only probes give borrowed-fork one acceptance/two
stack failures and dead-parameter three stack failures. They do not replace the
census or qualify execution. The four current failures are
`check/bystander_hypothesis_cost`, `check/deep_hyp_rewrite`, `page/deep_list_walk`
and `rfc/share_loop`; the historical six-failure cohort is preserved separately.
No upstream acceptance shift is claimed as a port correction.

`check/axiom_warn` and `grade/open_claim` remain explicit adjacent mismatches:
both reject at checking, but upstream's
[final incomplete-proof check](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L425)
prints a TODO count while the port reports `unfilled laws: ...` through its host
formatter. This is a distinct boundary from live reference inference and is the
next demonstrated independent diagnostic gap. Neither is counted as fixed here.

Nat-first C failure ordering remains pending and unwaived; the six approved
wildcard observations are a separate historical scope. The previous demand
renewal at `208832a` remains 92 exact / 6 approved / 2 pending, runner exit 1; it
is not relabelled as a current-source differential run. Nat/string compact
lowering remains unadopted. Existing F32.read, source-hole/TODO, foreign, host,
resource, oracle, external-API and supplementary all-target lint limitations remain
unwaived. No new semantic exception or runtime policy was adopted.

Inventory linkage increases from **445 to 451 of 1,302** through the six original
rejections. This counts applicable pinned inventory entries, not authored/noninventory
ledger strings or universal parity. The remaining **851** unlinked entries comprise
522 interpreter/compiled candidates, 315 checker negatives, 13 interpreter-only
cases and one checking-only module. Linked receipts retain their actual tested
commits; no historical result is relabelled as current. Full CPU parity remains
incomplete under the unchanged acceptance criterion.

## Reproduction and preservation

Activate the installed frozen tools and use these host build settings:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --features extended_observability --test cpu_unfilled_laws
cargo test --locked --features extended_observability --lib \
  --test cpu_template_occurrences --test cpu_definition_provenance \
  --test cpu_pattern_locations --test kernel_open_closures \
  --test runtime_review --test runtime_packed_words \
  --test compiler_executable_c_continuations
cargo clippy --locked --features extended_observability \
  --test cpu_unfilled_laws -- -D warnings
node "$UPSTREAM/bend2/main.ts" \
  tests/fixtures/cpu-unfilled-law/check/axiom_use.bend
"$BEND" check tests/fixtures/cpu-unfilled-law/check/axiom_use.bend
"$BEND" compile tests/fixtures/cpu-unfilled-law/positive/filled.bend \
  --executable --target c --output "$RUN/port.c"
"$TEAMY_BEND_CC" -std=c11 -O3 -Wall -Wextra -Werror -pedantic \
  "$RUN/port.c" -lpthread -lm -o "$RUN/port"
"$RUN/port" --threads 1 --gpu off
python3 scripts/checking-census/run.py --reference "$UPSTREAM" \
  --binary "$BEND" --phase-probe "$PROBE" --output "$FRESH_CENSUS" \
  --build-receipt "$BUILD_RECEIPT"
```

`BEND` and `PROBE` must come from the clean tested source; see the matching
[build/validation receipt](cpu-unfilled-law-validation.json) and
[census build instructions](../scripts/checking-census/README.md). Matrix JSON
records every command and hashes with `$REPO`, `$UPSTREAM`, `$RUN`, `$EVIDENCE`
placeholders; repeated equality is verified before compaction. Comparison children
have 30-second bounds; the census uses its existing four workers, 30-second oracle
and 60-second port/probe bounds. Host Rust debug/incremental settings limit cache
size and do not change Bend runtime profiles.

Minimum measured disk headroom was **11.260 GiB workspace / 4.692 GiB
temporary**. The recorded 11 GiB workspace preflight and 10 GiB workspace / 1 GiB
temporary stop floors remain enforced. Existing debug-cache placement and disabled
Rust debug/incremental data kept build-cache growth bounded; no cache or unique evidence
was deleted. No frozen checkout, dependency, lockfile or toolchain changed.

Compact unique failure evidence, source fixtures, goldens, census categories and
commands are in Git. Raw source-bearing streams, full logs, initial drafts,
generated executables and lossless binary snapshots remain local and **are not
preserved by Git pushing**. No installation, desktop/GPU work, main merge or release
occurred. Publication is a reviewed isolated checkpoint, not parity acceptance.
