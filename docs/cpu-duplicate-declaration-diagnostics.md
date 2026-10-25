<!-- SPDX-License-Identifier: MPL-2.0 -->
# Duplicate-declaration load diagnostics

Tested source `f0ff543052518d156537ac3380764b342d9e5dd2` starts from published `2a7faf5678520a6c0a9be31ab81f0ac21264bfde`.
The upstream target remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, and mounted
docs remain `50e89d6897ce2a65167ef65b17c5fa83fe849158`. Dependencies, toolchains,
evaluation, demand, termination, acceptance, structural/work/depth guards and
cancellation boundaries are unchanged. Full CPU parity is incomplete.

## Correction and scope

The four unchanged originals `check/data_table_checked_000`, `comptime/err_dup`,
`parse/duplicate_declaration` and `parse/reserved_def_name_000` already reject
while loading. The port now attaches private metadata at those existing refusals,
showing pinned `a fresh name (duplicate declaration: NAME)` output, the raw
character after the name and the original source window.

Pinned [parse_fresh](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2446)
checks that declaration name. [parse_def](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2452)
first distinguishes completion of an unfilled law from a new declaration.
[parse_fail](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1493)
observes the character at its current raw cursor.

The port keeps its existing rejection predicate, ordering and cursor. Type/law
parsing saves the name token before consuming it; definition parsing reuses its
existing saved token index, because its guard runs after `(`. The shared private
post-name formatter now accepts an expected category, retaining the previous
constructor behavior and checks for missing/invalid source positions. It reads
original trivia/CRLF/EOF rather than the next token. Ordinary repeated filled
definitions receive metadata only when their resolved rejected name equals the
pinned namespace-qualified declaration name. Differing targets stay untouched and
unqualified. No core term, reference identity, sharing edge or budget charge is added.

**Public compatibility:** `ParseError` source, line, column, message, `Display`
and ordinary `load`/`load_executable` error results remain unchanged. Public CLI
stderr intentionally changes for matching duplicate refusals. These newly
formatted failures use the existing `Ok(CliOutput)` boundary with `exit_code() == 1`
instead of the earlier unstructured `Err`; CLI processes still exit 1. Cancellation,
write/flush errors and unavailable source metadata retain `Err`. The production
caller in `src/lib.rs` consumes `exit_code()`, while dispatch propagates the output.
External invoke consumers were not audited; that compatibility work remains open.
No new public signature or mid-normalization cancellation capability is claimed.

## Exact comparisons and controls

Thirteen negative roots cover the four originals, repeated types/laws/filled
definitions, template duplication, space/comment/newline/CRLF/EOF positions and
an imported qualified source. They pass **234 exact diagnostic commands plus
78 separate load-phase probes**, twice per available lane. An additional unchanged
original, `import/duplicate_name`, passes **18 exact commands plus six phase probes**.
All reject at load, status 1, with empty stdout and byte-exact pinned stderr.
No negative artifact is emitted; execution is unrun. Lanes include pinned checking
and C/JS emission, plus port check/eval/run and executable C/JS/native emission.

Two positive roots cover ordinary law completion, actual template specialization,
imported unfilled-law completion, repeated imports and identical short names in
different namespaces. Their **52 commands** comprise eight pure normalizations,
sixteen checking/phase observations, twelve emissions/builds and **16 actual
port/upstream C-O3/JS executions**. Normalization/runtime results are exactly
`7\n`, status 0 and empty stderr. The port's strict-check JSON matches two explicit
old-binary checks byte-for-byte. All four generated port C/JS artifacts are
byte-identical to baseline. Pure native `run_main` is normalization, not native IO.
C uses the pinned O3 profile, one CPU worker and GPU off; port C keeps strict warnings.

Total matching matrix commands: **388**. Every matrix child is bounded at 30 seconds,
with exact repetitions and no accepted timeout or output normalization.
[Exact commands and receipts](cpu-duplicate-declaration-results.json) and
[compact pre-fix failures](../tests/fixtures/cpu-duplicate-declaration/duplicate-declaration-before.json)
retain source/binary hashes. The baseline binary is from `4d6459c`, production-identical
to the starting documentation checkpoint. New tests also compare tracked/plain
public errors, old constructor goldens, successful term/work/fresh-ID behavior,
and existing cancellation/write/flush/fallback contracts.

## Retained discrepancies and next substantive work

[Seventy-four additional observations](cpu-duplicate-declaration-open-cases.json)
retain three authored cases separately from matching counts and from the 1,302-fixture
census. These are not parity passes or newly approved exceptions:

- A repeated `def value` without `(` reaches the port's existing parenthesis error
  before its duplicate guard. Upstream reports freshness first. Both reject during
  loading; their diagnostics still differ.
- An import alias `D` points to `defs.bend`, which defines `value() = 3`. Declaring
  `def D.value() -> U32: 4` is a new qualified declaration in pinned upstream, while
  expression lookup through alias `D` still reaches `defs.value`. Upstream main
  returns `3`; the port incorrectly refuses the already resolved `defs.value`.
- Inside an imported file, declaring a local `Nat.add() -> U32: 7` shadows the
  filled Base name with a namespace-qualified declaration upstream. Its main
  returns `7`; the port instead refuses the globally resolved `Nat.add`.

The two accepted upstream programs each pass pure normalization and C-O3/JS
execution twice (**eight actual upstream executions** total). Every port CLI lane
still refuses during load, with no artifact or runtime execution; the failures
are unchanged from the baseline. Preservation tests ensure the current diagnostic
fix does not relabel those mismatches as freshness agreement.

[Qualification and resolution](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1646)
are distinct operations upstream. Its `parse_def` resolves a name only to complete
an eligible unfilled law; otherwise it uses the qualified raw declaration spelling
for freshness. **Next substantive task:** correct that port declaration-target
selection, with imported-law completion, template, alias and shadowing controls.
This checkpoint preserves acceptance and does not implement that semantic correction.
The missing-parenthesis diagnostic order remains a separate unwaived case.

## Validation and reproducibility

Host build profile, separate from Bend runtime O3:

```bash
source "$TOOLS/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib duplicate_
cargo test --locked --features extended_observability --lib source_origins_preserve_checker_work_terms_and_successful_evaluation
cargo test --locked --features extended_observability --lib pattern_load_boundary_preserves_cancellation_writer_errors_and_fallback
cargo test --locked --features extended_observability --test cpu_duplicate_declaration --test cpu_duplicate_constructor
cargo clippy --locked --features extended_observability --test cpu_duplicate_declaration --test cpu_duplicate_constructor -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The clean standard gate passes **1,220 tests / 0 failed / 49 ignored** in 907.16 seconds. Formatting, strict production Clippy, all-feature build and real Node TCP/UDP provider checks pass; existing ignores are unchanged. The corrected focused command passes **six tests and strict Clippy**. The earlier compound command's twelve successful tests are retained as part of a failed command, not relabelled a successful gate. Full validation renews prior host-stack/shared-reference, work/depth, aggregate composition, false-proof, cancellation, declaration completion and diagnostic controls. Independent source/evidence review found no blocker. No assertion, ignore or budget was weakened.

The initial focused compound command had twelve passing tests and then failed
Clippy because `definition` grew to 101 lines against its 100-line rule. Extracting
the unchanged duplicate-definition error construction fixed that lint without
suppression; the failed command remains recorded. The final unit/Clippy rerun and
clean full gate have separate receipts.

The first negative matrix remains **incomplete** after 207 exact raw diagnostic
comparisons and 66 saved phase probes. A new assertion used Python `text=True`
for phase stderr, which normalized CR characters and caused a harness failure.
The corrected runner captures bytes and compares them without normalization; the
full 13-case rerun passes. No production change, test weakening or waived CRLF
comparison was used. [Validation evidence](cpu-duplicate-declaration-validation.json).

Committed CRLF bytes match the baseline and HEAD/index/worktree. Default Git
whitespace checking reports five deliberate CRLFs and three trailing spaces in
unchanged upstream expectation comments. CR recognition retains those three
original-comment warnings. Checking the other changed files with CR recognition
passes; all excluded originals are byte-verified against the pin. No original
source, global setting, compiler warning or test was altered to silence warnings.

## Current census and remaining scope

The renewed 1,302-fixture census records **366 both accepted, 368 exact rejections, 493 acceptance disagreements, 68 different rejections and 7 upstream host/oracle failures**.

Every port status and phase is unchanged. Exactly five port stderr streams change: the four requested originals and `import/duplicate_name`, now exact. All other 1,297 port streams remain unchanged. The 385 historical IO members renew byte-for-byte. **176 measured differences remain outside that cohort: 108 acceptance and 68 rejection differences**.

The preceding residual was 182. Five corrections are attributable to the port change. Separately, `flatten/literal_rows_cubic` changed from upstream acceptance to an upstream load-phase host-stack failure. The port's Nat-literal-above-128 refusal is unchanged. Thus 182 minus five corrections minus one temporarily unmeasurable oracle cell gives 176; this is **not six port fixes** or a waived resource mismatch. Current oracle failures: `check/bystander_hypothesis_cost.bend`, `check/deep_hyp_rewrite.bend`, `flatten/literal_rows_cubic.bend`, `page/deep_list_walk.bend`, `reg/borrow_fork_hold.bend`, `reg/dead_param_fork.bend`, `rfc/share_loop.bend`. Older instability, including `reg/borrow_fork_hold`, remains unwaived. These are checking-only measurements.

The [compact delta](cpu-duplicate-declaration-census-delta.csv) extends the prior
public census chain without overwriting historical receipts. Both-accept is only
checking status agreement. Historical IO membership is subtracted only after its
exact 385-member renewal; new symptoms never become exclusions.
[Measured reconciliation](cpu-duplicate-declaration-census.json).

Original fixture linkage rises **466→471 of 1,302**. The **831 unlinked** entries comprise 521 interpreter/compiled candidates, 296 checker negatives, 13 interpreter-only cases and 1 checking-only module. These are applicability categories, not 831 missing runtime tests; links and authored controls do not establish universal parity.

`nat_first` remains pending and unwaived; compact Nat/string lowering remains
unadopted. Prior resource, source-hole/TODO, foreign/host, F32.read, upstream printer,
supplementary all-target lint and external invoke-audit gaps remain open. The two
new authored namespace acceptance cases are additional evidence outside the census,
not hidden in its totals. Passing the standard gate does not establish full parity.

Minimum recorded free space: **11.082 GiB workspace / 4.374 GiB temporary**. The 11 GiB build preflight and 10 GiB workspace / 1 GiB temporary stop floors remain enforced. Existing cache placement and disabled host debug/incremental data remain in use. Historical evidence dominates workspace storage; new growth is bounded snapshots/receipts. No cache or unique evidence was deleted.

Compact sources, errors, commands, hashes and reports are preserved in Git. Bulk
logs, binary/gzip snapshots, generated programs and full census JSONL remain local
under `$EVIDENCE`; pushing Git does not preserve those files. All unique historical
evidence is retained. No installation, dependency/toolchain change, desktop/GPU
operation, main merge or release occurred.
