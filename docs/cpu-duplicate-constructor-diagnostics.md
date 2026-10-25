<!-- SPDX-License-Identifier: MPL-2.0 -->
# Duplicate-constructor load diagnostics

Tested source `4d6459c0b1a81499fd106c352ae4ce8696db8281` starts from published `55aef700c7868c50a17952e98b7727d9c480282a`.
Upstream stays at `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; mounted docs stay
at `50e89d6897ce2a65167ef65b17c5fa83fe849158`. Inputs, dependencies, toolchains,
acceptance, evaluation, termination, demand, cancellation and safety budgets are
unchanged. Full CPU parity remains incomplete.

## Contract and change

The four unchanged originals `check/book_prefix_arity`, `check/ctor_shared_tag`,
`compile/ctor_tag_dispatch_000` and `_001` already reject during loading in both
implementations. The port previously exposed its host-formatted `duplicate
constructor` error. It now attaches private structured metadata at that same
failure, showing the pinned expected category, observed character and source window.
Compile-directory placement is not executable applicability.

Pinned [constructor parsing](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2575)
calls [parse_fail](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1493)
immediately after the duplicate name, before whitespace/comment skipping.
The new helper uses the consumed unqualified token's line and character column
against original registered source. Only the expected name is namespace-qualified.
It preserves raw space, tab, comment marker, LF, CRLF and actual EOF instead of
reporting the next token. Invalid/unregistered source positions produce no metadata.
The duplicate predicate, constructor-tag insertion order and public parser cursor
are unchanged. No term field, reference node, sharing edge, evaluator or budget
charge is added.

Independent review found an invalid helper offset that could incorrectly report
EOF after a newline. It was corrected before the accepted checkpoint and covered
by multiline/trailing-newline invalid-position regressions. This path is not
reachable from a valid constructor token, but fallback must still be accurate.

**Compatibility:** Public `ParseError` fields, message, `Display` and ordinary
`load`/`load_executable` error results remain unchanged. Private diagnostic metadata
and public CLI stderr intentionally change. These newly formatted failures use the
existing `Ok(CliOutput)` with `exit_code() == 1` boundary where the old unstructured
error returned `Err`. CLI processes still exit 1. Cancellation, write/flush failure
and absent diagnostic source retain `Err`. The repository production caller in
`src/lib.rs` reads `exit_code()` and returns it; command dispatch propagates the
status-bearing value. External invoke consumers were not audited and must inspect
that status. No new public signature, broad Result redesign or mid-normalization
cancellation capability is claimed.

## Differential evidence

Twelve negative roots cover the four originals, a duplicate in one datatype,
post-name space/tab/comment/LF/CRLF/EOF and an imported qualified duplicate.
**216 exact diagnostic commands and 72 separate load-phase probes** pass, with
identical status 1, empty stdout, complete stderr and no artifact, twice each.
Available lanes are pinned checking/C/JS emission and port check/eval/run plus
executable C/JS/native compilation. Negative execution is unrun.

Two additional unchanged originals, `parse/duplicate_ctor_name` and
`page/ambiguous_ctor_error`, share the same mechanism and pass **36 exact
diagnostic commands plus twelve phase probes**. Both have independent pre-fix
refusal receipts. No upstream source was modified.

Two positive roots exercise distinct local constructors, equal short tags in
separate imported namespaces and repeated import reuse. Their **52 commands**
cover eight pure normalizations, sixteen checking/phase observations, twelve
emissions/builds and **16 actual upstream/port C-O3/JS executions**. Normalization and runtime
execution results are exactly `7\n`, status 0, empty stderr, with two repetitions. Native `run_main` for
these pure programs is normalization, not native IO execution. All four port
C/JS artifacts are byte-identical to baseline. C uses pinned O3 with one CPU worker
and GPU off; port C additionally retains strict warnings. Matrix children have
30-second deadlines; no tolerance, diagnostic trimming, timeout or nondeterminism
was accepted. Total final matrix commands: **388**.

[Commands, hashes and streams](cpu-duplicate-constructor-results.json) and
[compact pre-fix evidence](../tests/fixtures/cpu-duplicate-constructor/duplicate-constructor-before.json)
retain exact observations. Unit tests compare all public tracked/plain error
fields in strict/executable loading, invalid-source fallback, and existing CLI
cancellation/writer/flush behavior. Positive fixtures also enter the existing
tracked/plain structure, fresh-ID, work/depth and normalization checks.

An initial authored positive directly matched already constructed values. Both
implementations reject that form. Its source and failed emission remain archived
in the compact evidence; the corrected control matches function parameters.
This was a fixture correction, not an acceptance change.

## Reproducible validation

Host build profile (separate from Bend runtime O3):

```bash
source "$TOOLS/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib duplicate_constructor
cargo test --locked --features extended_observability --lib source_origins_preserve_checker_work_terms_and_successful_evaluation
cargo test --locked --features extended_observability --lib pattern_load_boundary_preserves_cancellation_writer_errors_and_fallback
cargo test --locked --features extended_observability --test cpu_duplicate_constructor
cargo clippy --locked --features extended_observability --test cpu_duplicate_constructor -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The clean standard gate passes **1,216 tests / 0 failed / 49 ignored** in 888.42 seconds. Formatting, strict production Clippy, all-feature build and the real Node TCP/UDP socket-provider check pass. Existing ignores are unchanged. Focused checks pass **6 / 0 / 0**, with targeted strict Clippy also passing. Those focused runs are worktree checks; the full gate attests the named clean commit. Earlier stack/shared-reference, work/depth, aggregate composition, cancellation, false-proof and provenance controls remain passing. Independent source/evidence review found no blocker; no assertion, ignore, guard or budget was weakened.

Two earlier gate runs were stopped by this task at 26.38 and 17.79 seconds, both
exit 143, and remain incomplete receipts. The root Git attributes initially
normalized the CRLF fixture. Adding a scoped `-text` rule alone did not refresh
the cached index, and a missing shell `set -e` allowed the second gate to start
after a failed byte assertion. The task stopped it immediately, explicitly
re-staged that one fixture, and verified HEAD/index/worktree bytes before the
successful run. The final committed CRLF SHA-256 is
`559845cf5291309d7dc557efba2c478c1197a303e8107bc9e0ec72bfc54e8cc7`.
Production source is identical across those three source checkpoints; incomplete
runs are never substituted for the clean final gate. [Validation receipts](cpu-duplicate-constructor-validation.json).

The default Git whitespace check flags the fixture's six deliberate carriage
returns. An invocation-scoped check with explicit `cr-at-eol` recognition passes
while retaining ordinary trailing-space/blank-line/tab checks. Both outcomes are
recorded; no global Git configuration, compiler warning or test was disabled.

## Current census and remaining work

The renewed 1,302-fixture census measures **366 both accepted, 363 exact rejections, 494 acceptance disagreements, 73 different rejections and 6 upstream host/oracle failures**.

Every port status and phase is unchanged. Exactly six port stderr streams change: the four requested originals plus the two adjacent duplicate-constructor originals, now exact. All other 1,296 port streams remain unchanged. The 385 historical IO members renew byte-for-byte. **182 measured differences remain outside that exact cohort: 109 acceptance and 73 rejection differences**.

The preceding residual was 189. Six corrections are attributable to this port change. Separately, `reg/borrow_fork_hold` changed from upstream acceptance back to an upstream load-phase stack failure; the port still refuses its Nat literal under the unchanged 128-level parser guard. Thus 189 minus six port fixes minus one temporarily unmeasurable oracle cell gives 182. This is **not seven port fixes** or a waived resource difference. Current host/oracle failures: `check/bystander_hypothesis_cost.bend`, `check/deep_hyp_rewrite.bend`, `page/deep_list_walk.bend`, `reg/borrow_fork_hold.bend`, `reg/dead_param_fork.bend`, `rfc/share_loop.bend`. Historical instability remains unwaived even when a particular run completes. These are checking-only measurements.

The compact census delta extends the existing full-base/delta chain through
[cpu-forward-data-census-delta.csv](cpu-forward-data-census-delta.csv); old
receipts are not overwritten. Both-accept counts establish checking status only.
Historical IO membership is subtracted only after exact 385-member renewal;
new IO symptoms never become exclusions. Oracle instability is kept separate
from port changes. [Measured reconciliation](cpu-duplicate-constructor-census.json).

Original fixture linkage rises **460→466 of 1,302**. The **836 unlinked** entries comprise 521 interpreter/compiled candidates, 301 checker negatives, 13 interpreter-only cases and 1 checking-only module. These are applicability categories, not 836 missing runtime tests. Links and authored controls do not establish universal parity.

Next bounded diagnostic family: duplicate declaration errors in
`check/data_table_checked_000`, `comptime/err_dup`, `parse/duplicate_declaration`
and `parse/reserved_def_name_000`. [Twenty bounded observations](cpu-duplicate-constructor-next.json)
confirm load rejection in both implementations and retain the differing output.
No correction or runtime qualification is claimed for that next family.

`nat_first` remains pending and unwaived; compact Nat/string lowering remains
unadopted. Existing resource, source-hole/TODO, foreign/host, F32.read, upstream
printer failures, supplementary all-target lint failure and external invoke audit
remain open. Passing this gate or linking a receipt does not meet the full CPU
acceptance criterion. No new semantic exception is introduced.

Minimum recorded free space: **11.106 GiB workspace / 4.417 GiB temporary**. The 11 GiB build preflight and 10 GiB workspace / 1 GiB temporary stop floors remain enforced. Existing rebuildable cache placement and disabled host debug/incremental data remain in use. Bulk historical evidence dominates workspace usage; fresh growth is bounded snapshot/receipt data. No cache or unique evidence was deleted.

Compact source/expected failures, commands, checksums and reports are retained in
Git. Bulk raw logs, binary/gzip snapshots, generated programs, complete census
JSONL and interrupted-run logs remain local under `$EVIDENCE`; pushing Git does
not preserve those files. Prior unique evidence is retained. No install,
dependency change, desktop/GPU operation, main merge or release was performed.
