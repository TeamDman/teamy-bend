<!-- SPDX-License-Identifier: MPL-2.0 -->
# Unknown-template load diagnostics

Production and comparison source `eb65f84a7a194c1a6d8aae39563705a14fe41842`; renewed full-gate source
`e9638be533aed465820168ced57b6f27f54e82d9` adds only a current diagnostic golden and compact evidence,
with production source identity verified. Starting from published
`5e3ce421aff0be3e2874ae7b8b6cb7fa24de5ca4`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; mounted docs remain
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Dependencies, toolchains,
evaluation policy, guards, budgets and cancellation are unchanged. Full CPU parity
is incomplete.

## Correction and public contract

The existing `atom_inner` refusal for an unconsumed `~` now records the existing
optional `expected_syntax` metadata. Previously the authored `D.missing(~9)`
reproducer emitted an internal ParseError through the host formatter. It now emits
pinned `expected : a term`, `observed : '~'` and the correct Bend source window,
with exit 1 and empty stdout. The two source files preserve the prior alias-fallback
reproducer byte-for-byte. Its earlier phase-only receipt remains historical; this
checkpoint supplies exact evidence.

Pinned [parse_term_ops](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2026)
consumes leading markers only for a known template. Other markers reach
[parse_term_base](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1961)
and fail as terms. The correction attaches metadata at that same reached refusal,
without changing token consumption, template lookup/replay, checking or execution.

Plain `load`/`load_executable` retain Err with identical public ParseError fields
and Display. There are no new public types/signatures. For these newly formatted
failures, CLI `invoke` now follows the existing `Ok(CliOutput)` boundary with
`exit_code() == 1` rather than propagating the unformatted host Err. Repository
CLI exit handling is exercised in six command lanes; external consumers remain
unaudited. Writer failures, flush failures, cancellation before output and missing
metadata still return Err. No mid-normalization cancellation is claimed.

## Exact and executable evidence

The [matrix](cpu-unknown-template-results.json) records **464 matching commands**, zero unexpected outcomes or timeouts, and deterministic repeated status/stdout/stderr. This includes **252 exact negative diagnostic comparisons plus 56 separate port phase probes**, **48 actual generated C/JS executions** and **24 pure normalizations**. Thirteen negative roots reject during loading and one during checking. Twelve diagnostics improve from prior mismatch; two adjacent controls were already exact. Phase observations are not substituted for exact text comparisons.

Four unchanged Apache-2.0 upstream originals are attributed in the fixture NOTICE:
`comptime/err_head`, `comptime/err_later`, `comptime/err_runtime`, and
`parse/comp_arrow_position`. Eight authored tilde negatives cover unknown/plain
heads, misplaced markers, standalone terms, imported/multiline locations and
replayed template bodies. Two adjacent controls preserve already matching
undefined-member checking and unfinished-call loading errors. All actual phases
are recorded; rejected programs emit no artifact and their execution is unrun.

Six valid authored controls cover imported templates, raw alias fallback,
known-member precedence, nested specialization, repeated cache/replay calls and
ordinary functions. Complete normalized U32 values are `12, 28, 9, 12, 9, 12`.
Native pure results are normalization, not native IO execution. No foreign,
GPU or device-host execution is qualified by this batch.

Twelve generated port C/JS artifacts remain byte-identical to the captured baseline.
C execution uses O3, one worker and GPU off; port C retains strict warnings.
Baseline emission alone is not an execution claim. Existing strict IO exclusions,
upstream host/printer failures and all unrun lanes remain outside these matches.

## Regression gates

The standard gate passes **1,232 / 0 / 49** (passed / failed / ignored), 181 result groups, exit 0, in 792.99 seconds. Formatting, strict production Clippy, all-feature build and real Node TCP/UDP validation pass. The initial gate failed on the retained missing-marker golden (764 passed / 1 failed / 48 ignored; later targets unrun). Its test-only renewal and seven exact upstream/port checks are preserved separately. Initial focused integration: **55 / 0 / 0**; library controls: **290 / 0 / 1**; expanded final focused integration: **2 / 0 / 0**. The affected marker-renewal focused suite passes **6 / 0 / 0**. Targeted strict Clippy passes. These are separate runs, not an additive unique-test count.

Exact commands and log/source hashes are in
[cpu-unknown-template-validation.json](cpu-unknown-template-validation.json).
Host build profile overrides do not change Bend's C-O3 runtime profile:

```bash
source "$TOOLS/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test cpu_unknown_template --test cpu_alias_fallback --test parser_templates --test parser
cargo test --locked --features extended_observability --lib
cargo test --locked --features extended_observability --test cpu_unknown_template
cargo test --locked --features extended_observability --test cpu_list_for_each --test cpu_unknown_template
cargo clippy --locked --features extended_observability --test cpu_unknown_template -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Direct regressions compare tracked/plain error fields, source locations, token
bytes, cursor/fresh-ID state and earlier 64-level expression and 128-argument refusals. Guards
must retain their unformatted resource cause and acquire no tilde metadata.
Successful checking work, full terms and fresh IDs remain equal. Existing template
growth, packed words, shared references, host-stack, aggregate-work, builtin-origin
and cancellation controls remain in the full gate. Independent source/evidence
reviews are recorded; no assertions, ignores or limits were weakened.

Initial authored positive controls used default Nat `+` for U32 operands; both
implementations correctly rejected them. Their initial sources/receipts are retained,
and explicit `U32.add` corrects the authored controls. No original was altered.
An early snapshot identity check found the all-feature Tracy binary during the
build/test transition. That capture was rejected before matrix/census use, preserved
separately, and replaced with a verified default+extended_observability snapshot.
No failed preparation is counted as a qualified comparison.

The first full gate caught the old current stderr expectation for
`cpu-list-for-each/negative/missing-marker`. Seven bounded pinned checker/generator
and port executable-boundary comparisons confirmed the new diagnostic exactly.
Only its current golden changed to match the existing upstream golden; assertions
and all original/historical receipts remain unchanged. [Failed gate and renewal](cpu-unknown-template-renewal.json).

## Census, coverage and remaining gaps

The renewed **1,302-fixture checking census** records 366 both accepted, 372 exact rejections, 494 acceptance disagreements, 64 different rejections and six unchanged upstream stack failures. All port/upstream acceptance statuses and failure phases remain unchanged; all upstream streams remain unchanged. Seven port stderr streams change: the four original tilde diagnostics become exact, while `base/list_fold`, `gfx/app_linear` and `gfx/app_play` retain their strict-mode rejection and acceptance disagreement. Formatting those three refusals does not establish executable parity or admit a new exclusion.

The historical 385-member IO cohort renews byte-exactly without widening. The measured residual outside it decreases **177 → 173 = 109 acceptance + 64 rejection differences**, entirely through the four demonstrated diagnostic fixes. Six oracle failures stay separate (`check/bystander_hypothesis_cost`, `check/deep_hyp_rewrite`, `page/deep_list_walk`, `reg/borrow_fork_hold`, `reg/dead_param_fork`, `rfc/share_loop`); no oracle recovery or regression is claimed. [Reconciliation](cpu-unknown-template-census.json) and [four canonical status delta rows](cpu-unknown-template-census-delta.csv) preserve the current observation identities; the three additional stream-only changes remain recorded in the reconciliation. Historical triage categories are observational: the 173 split into 109 acceptance, 35 diagnostic candidates and 29 other rejection differences; do not call every rejection difference diagnostic-only.

Four new exact-negative inventory links give **475 linked / 827 unlinked**: 521 interpreter/compiled candidates, 292 checker negatives, 13 interpreter-only cases and one checking-only module. The [family ledger](cpu-unknown-template-evidence.csv) records rejection and unrun execution explicitly. Links are evidence coverage, not universal parity; unlinked entries are not all missing runtime tests.

`nat_first` remains pending and unwaived. Compact Nat/string lowering remains
unadopted. External invoke auditing, earlier missing-parenthesis ordering, resource,
source-hole/TODO, F32.read, foreign/host, printer and supplementary all-target lint
gaps remain. No new semantic exception is introduced. Next bounded diagnostic candidate: `comptime/err_untyped`, where both reject loading but the port retains a quantified-binder message rather than the pinned colon expectation. This remains a measured discrepancy, not fixed here.

Minimum recorded free space: **11.027 GiB workspace / 4.114 GiB temporary**. Existing 11 GiB preflight and 10/1 GiB stop floors remained enforced. No new cache cleanup, timeout or disk-floor stop occurred. Earlier inactive-cache placement and disabled host debug/incremental settings remain; unique evidence is preserved.

Compact fixtures, exact diagnostic goldens, commands/statuses/streams and hashes are
committed. Bulk logs, binary/gzip snapshots, generated artifacts and full census
JSONL remain local under `$EVIDENCE`; Git publication does not preserve those files.
No source or unique historical evidence was deleted. No dependency/toolchain
change, installation, desktop/GPU work, main merge or release occurred.
