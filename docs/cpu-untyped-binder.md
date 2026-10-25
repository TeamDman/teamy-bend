<!-- SPDX-License-Identifier: MPL-2.0 -->
# Untyped compile-time binder diagnostics

Tested source `792aaf0abbb5f3c20a046ab39ecc66967350c2b1`; upstream remains
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
This bounded checkpoint closes missing-colon diagnostic differences without
changing loading acceptance, checking, evaluation or resource limits. CPU parity
is still incomplete under the [acceptance criterion](cpu-parity-acceptance.md).

## Change and contract

Pinned [`parse_tele`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2418)
allows an untyped bare name only as a quantity binder. Compile-time `~`, erased
`-` and reusable `+` binders require a colon and type. The port already enforced
this rule but its refusal reached the host error formatter. Four production
lines attach the existing optional expected-token metadata at that refusal.
The original `comptime/err_untyped` now reports `expected : ':'`, `observed : ','`
and its original Bend source window. No token consumption or binder creation
is added; the earlier 128-binder guard retains priority.

Public `ParseError` fields and Display, loader `Err` behavior, API signatures,
core terms, sharing and evaluation remain unchanged. The visible CLI diagnostic
now uses the existing `Ok(CliOutput)` failure boundary with `exit_code = 1` instead
of a host-formatted `Err`. Repository boundary tests retain nonzero process status,
no artifact creation, cancellation and writer errors. External `invoke` consumers
remain unaudited and must inspect `exit_code`; this change does not complete that
audit or introduce a new error type.

## Bounded comparisons

The [matrix](cpu-untyped-binder-results.json) records **434 commands: 408 matching, eight strict-IO exclusions and 18 retained wrong_type observations**. There are zero unexpected outcomes or timeouts, and repeated status/stdout/stderr are deterministic. Matching observations include **216 exact negative diagnostic comparisons plus 48 separate port phase probes**, **52 actual executions** (48 generated C/JS and four native IO) and **16 pure normalizations**. Eleven exact-negative roots reject during loading and one during checking. Nine missing-colon diagnostics improve; three adjacent controls were already exact.

Five upstream originals are copied byte-for-byte with Apache attribution. Exact
negative roots include `err_untyped` and `err_missing`; the latter already matched.
Authored missing-colon controls cover comma/closer/EOF, imported/multiline source,
datatype parameters and constructor fields. Missing-name and missing-type
controls preserve their distinct existing diagnostics. Compact
[pre-fix evidence](../tests/fixtures/cpu-untyped-binder/before.json) preserves the
old outputs, including an initially malformed authored quantity-literal control
that both parsers rejected. Its corrected spelling is the pinned `&2`; no original
fixture was altered. The Git whitespace check reports three inherited trailing spaces in original `#|` blank-source-line expectations; authored source passes. Those original bytes are deliberately preserved.

Four pure controls normalize to `4n`, `24`, `6` and `9`. Two unchanged IO originals
execute to `34\n` (`two_instances`) and `25\n` (`value_type`). Native IO, pure
normalization and generated C/JS execution are recorded distinctly. Direct upstream
IO interpretation was not executed; generated upstream C/JS provide the executable
oracles. Strict IO checking is excluded and remains a refusal, not a runtime pass.
C uses the pinned O3 profile, one worker and GPU off; port C also retains strict
Clang warnings. All twelve port C/JS artifacts are byte-identical to pre-fix
emission. Emission alone is not an execution claim.

`comptime/wrong_type` remains a checking diagnostic mismatch: upstream provides
Bool/Nat, binder context and the source line inside `app~0`; the port retains its
host-formatted type mismatch. Repeated phase probes establish both reject during
checking. No execution is claimed for rejected programs, and phase-only agreement
is not exact diagnostic parity.

## Validation

The standard gate passes **1,236 / 0 / 49** (passed / failed / ignored), 182 result groups, exit 0, in 943.27 seconds. Formatting, strict production Clippy, all-feature build and real Node TCP/UDP validation pass. Focused executable/template tests pass **16 / 0 / 0**; new parser/error controls **2 / 0 / 0**; cancellation/writer boundary **1 / 0 / 0**. Targeted strict Clippy passes. These are separate runs, not an additive unique-test count. The supplementary all-target lint failures remain historical and unrenewed.

Frozen host overrides (distinct from Bend's C O3 execution profile):

```bash
source "$TOOLS/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --offline --features extended_observability --test cpu_untyped_binder --test parser_templates
cargo test --locked --offline --features extended_observability --lib quantified_binder
cargo test --locked --offline --features extended_observability --lib pattern_load_boundary_preserves
pwsh -NoLogo -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --offline --features extended_observability --test cpu_untyped_binder -- -D warnings
```

[Validation receipts](cpu-untyped-binder-validation.json) record commands, source
and binary hashes, profiles, deadlines and totals. Tests compare tracked/plain
public errors, token bytes, cursor/fresh IDs, 127-versus-128 preceding binders,
source excerpts and successful checking work/terms. Existing shared-reference,
host-stack, template-budget, packed-word, aggregate-work and cancellation controls
remain in the full gate. No guard, ignore, assertion or safety budget was weakened.
Independent source and evidence reviews are retained. Cancellation coverage concerns existing entry/diagnostic boundaries; no mid-normalization cancellation is claimed.

## Census and remaining requirements

The renewed **1,302-fixture checking census** measures 366 both accepted, 373 exact rejections, 494 acceptance disagreements, 63 different rejections and six unchanged upstream stack failures. Every port/upstream status and failure phase is unchanged; every upstream stream is unchanged. The sole port stream change is `comptime/err_untyped`, now an exact rejection. The historical 385-member IO cohort renews byte-exactly, without widening. Residual differences outside that cohort decrease **173 → 172 = 109 acceptance + 63 rejection differences**. This is one measured original-fixture diagnostic improvement, not nine census improvements: the other eight corrected cases are authored controls. Historical triage labels divide the 172 into 109 acceptance disagreements, 34 diagnostic candidates and 29 other rejection differences; those labels do not prove every rejection difference is diagnostic-only. The six oracle failures remain `check/bystander_hypothesis_cost`, `check/deep_hyp_rewrite`, `page/deep_list_walk`, `reg/borrow_fork_hold`, `reg/dead_param_fork` and `rfc/share_loop`. No upstream variation or recovery is counted as port progress. [Reconciliation](cpu-untyped-binder-census.json) and [canonical delta](cpu-untyped-binder-census-delta.csv) preserve exact source and hash identities.

The [family ledger](cpu-untyped-binder-evidence.csv) links five originals, including the explicitly unresolved wrong_type row. Overall coverage is **478 linked / 824 unlinked**: 289 checker negatives, 521 interpreter/compiled candidates, 1 checking-only module, 13 interpreter-only cases. Linked evidence is not universal parity, and unlinked entries are not all missing runtime tests.

The next bounded diagnostic candidate is `comptime/wrong_type`: retain the
reproduction and investigate structured checker context/source propagation,
without changing its correct type rejection. `nat_first` stays pending and
unwaived; compact Nat/string lowering stays unadopted. External invoke auditing,
earlier parenthesis/error ordering, resource, source-hole/TODO, F32.read, foreign,
host/printer and supplementary all-target lint gaps remain. No new exception or
claim of overall CPU parity is made.

Minimum recorded free space is **11.073 GiB workspace / 3.630 GiB temporary**, with the 11 GiB preflight and 10/1 GiB stop floors enforced. Moving an inactive registry source cache preserved all 13,254 entries but reclaimed no overlay blocks. Moving inactive upper-layer debug artifacts then recovered 72.15 MiB; all 3,997 entries were verified and the original logical path preserved. Future debug outputs now use temporary storage. No source or evidence was removed, and no deadline/disk-floor stop occurred.

Small fixtures, expected diagnostics, sanitized before/after command receipts and
hashes are committed. Bulk logs, binary/gzip snapshots, generated artifacts and
full census JSONL remain local under `$EVIDENCE`; Git publication does not preserve
them. Unique source and historical evidence remain intact. No dependency/toolchain
change, installation, desktop/GPU work, main merge or release occurred.
