# Lowercase Unicode escape grammar checkpoint

The parser now rejects lowercase escapes with a leading plus or more than eight
hexadecimal digits, matching the pinned Bend grammar. Canonical escapes retain
all U32 codes, including non-scalars. CPU parity remains incomplete.

Tested source: `b46f9570a2174b6f475b4f58836d960534a967a8`.
Baseline checkpoint: `99b78be666af894710788963b0730d28dc770cc5`.
Upstream: `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
This checkpoint is local only; publication requires separate approval.

[Validation](cpu-lowercase-escape-grammar-validation.json),
[compact outcomes](cpu-lowercase-escape-grammar-results.json.gz),
[case table](cpu-lowercase-escape-grammar-evidence.csv),
[review](cpu-lowercase-escape-grammar-review.json),
[fixtures and baseline](../tests/fixtures/cpu-lowercase-escape-grammar).

## Contract and correction

Pinned [parse_char](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588)
matches braced `u`/`U` escapes within an eleven-character window: one to eight
ASCII hexadecimal digits. The previous lowercase path passed collected digits
to Rust's integer parser, which also accepts a leading plus and extra leading
zeroes. Both original authored reproducers, `\u{+41}` and `\u{000000041}`,
therefore normalized to 65 and generated C/JS programs that returned 65, while
pinned upstream refused them during loading.

The correction uses the existing grammar check before accepting a decoded value.
It preserves the existing scan, radix and noncanonical non-scalar refusal paths;
a successfully decoded noncanonical scalar now becomes an unknown-escape error.
Every successfully decoded escape follows the pinned grammar, so the private
prefix-eligibility flag is no longer needed. Diagnostic eligibility still excludes
later character codes, cross-line cursors and unsupported observed UTF-16 units.
No kernel, runtime, compiler, dependency, guard or budget changed.

## Measured results

- Fresh full gate on clean tested source: **1322 passed / 0 failed /
  49 ignored**, 195 result groups, 941.061 seconds. Formatting,
  standard strict Clippy, all-feature build and the Node provider checks passed.
- Earlier focused work: **15 unit tests and 14 integration tests passed**;
  targeted strict Clippy passed. These were run before the source commit;
  their original dirty-state receipts are retained. The clean full gate renews
  the regression tests. Supplementary all-target Clippy was not rerun.
- Baseline: **148 observations**, including four actual generated executions
  of the two authored reproducers. In the 34 authored negative controls, the
  baseline accepted 14, rejected two with exact diagnostics, and rejected 18
  with differing diagnostics.
- Corrected matrix: **676 observations / 106 deterministic pairs / 464
  singletons**, no timeout or unexpected outcome. All 34 controls now reject:
  **19 exact / 15 differing diagnostics**. This closes **14 acceptance gaps and
  three diagnostic gaps**, with no previously exact diagnostic regression.
- The unchanged pinned unknown-escape original also rejects exactly. Checking,
  evaluation, native-run entry and C/JS/native emission refuse the malformed
  programs without creating artifacts. These are refusal observations, not
  execution of rejected programs.
- Nine accepted controls produce **36 corresponding C/JS execution matches**:
  28 successes and eight required JS scalar refusals. Eight native pure
  normalization outputs and one actual native IO output are unchanged; all
  18 generated port artifacts are byte-identical to the baseline. The pinned
  string-printer original is unchanged. Strict checking of the IO control
  remains an exclusion; runtime success is measured separately.
  Upstream pure normalization was checked for successful exit, not complete
  graph/text equality in this matrix; that stronger claim is not made here.
- Canonical boundaries, first-error order, escaped backslashes, complete
  constructor values, literal/expression/input guards, false proofs, sharing,
  work accounting and cancellation before effects remain under regression
  controls. No mid-normalization cancellation guarantee is claimed.

The 15 diagnostic residuals are `missing-brace`, `missing-close`, `empty`,
`nonhex`, `overflow`, `nine-max`, `plus-surrogate`, `nine-surrogate`, `fullwidth`,
`negative`, `only-plus`, `underscore`, `whitespace`, `char-second` and
`earlier-grammar`. Exact differing streams are retained. The first thirteen
preserve the original public errors. The latter two remain earlier grammar/
closing-quote mismatches; matching rejection does not qualify their diagnostics.

Three overlapping controls in the older uppercase cohort now match, making
that cohort 31 exact / eight differing. These are the same three improvements,
not additional closures. Its historical 28/11 receipt remains unchanged.

## API and acceptance limits

Thirteen legacy malformed/overflow/noncanonical non-scalar cases preserve public
`ParseError` fields and Display across strict, executable and diagnostic loaders.
First-error dominance intentionally changes for composed inputs:
`"\u{+41}\q"` now fails at `u`, while `"\q\u{+41}"` still fails at `q`.
The later-code character control can also change its raw error and remains
unqualified. This is not a blanket promise that all raw errors are unchanged.

Previously accepted programs now reject. Formatted command failures can return
`Ok(exit_code=1)` instead of `Err`; process status remains nonzero. Repository
dispatch handles both paths, but external `invoke` consumers remain unaudited.
No public type layout changed and no new semantic exception was adopted.

No whole census was renewed. The measured historical census at `c5edea8` remains
140 non-IO differences (109 acceptance / 31 rejection), plus six upstream stack
failures. Authored control fixes cannot be subtracted from that historical count.
Coverage linkage stays 516/1302, including failed receipts; 786 unlinked entries
comprise 256 checker rejection, 517 interpreter/compiled candidates, 12
interpreter-only and one no-main module. Links are not completed parity.

Pending `nat_first` C ordering, unadopted compact Nat/string lowering, Base source
coordinates, external invoke auditing, supplementary lints and older F32,
foreign, source-hole, resource and oracle gaps remain explicit. Next independent
candidate: the retained lowercase malformed-escape diagnostic family, beginning
with `\u{}` and invalid digits while preserving the public loader contract.

## Reproduction and preservation

Use the existing activated environment from the repository root:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off PYTHONDONTWRITEBYTECODE=1
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=4
cargo test --locked --features extended_observability --lib literal_ -- --nocapture
cargo test --locked --features extended_observability --test cpu_malformed_literal_diagnostics --test cpu_nonscalar_literal --test cpu_c_escape_printer --test cpu_uppercase_unicode -- --nocapture
cargo clippy --locked --features extended_observability --lib --test cpu_malformed_literal_diagnostics --test cpu_nonscalar_literal --test cpu_c_escape_printer --test cpu_uppercase_unicode -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The compact receipt retains every differential command, source hash, status,
stream and artifact hash. Paths use explicit root substitutions, including inside
decoded streams; original stream hashes remain. C uses `-std=c11 -O3 -lpthread
-lm`, with additional `-Wall -Wextra -Werror -pedantic` for the port. Generated
programs run with `--threads 1 --gpu off`; Node runs JS. Each observation has a
30-second host timeout. Cargo debug overrides do not alter Bend optimization
profiles or runtime budgets.

Raw matrix SHA-256: `31e2c14dde063d4719440a3a9b44bb116164485fd51060183812a1f669994c77`.
Fresh full-gate log SHA-256: `b8e2771ab005eadff53ec11ab822f0a966695205fefd91e997563e01d0bcac82`.
The post-gate binary is byte-identical to the matrix snapshot:
`94333b6e355c4581ce791ebd720fafb4664f905eff0de56f1aa6d0eec3d0fcf2`.
No redundant behavioral execution is claimed by this byte comparison.

The user-requested pause interrupted the first full gate after 402 passes,
zero failures and two ignores, exit 143. That immutable receipt is incomplete
and is retained separately from this fresh completed gate. Resumption verified
the source bundle and all 150 archived evidence members against originals.
The old bundle/archive remain intact. New local validation evidence is preserved
separately; no source, unique evidence or cleanup-restoration record was deleted.
Initial resumption verification used the wrong manifest key and stopped before
writing; the corrected read-only audit verified the actual `files` field.

Gate minima: 12.171 GiB workspace / 1.966 GiB temporary. Floors remain
11 GiB before builds, 10 GiB active workspace and 1 GiB temporary. The older
lost historical log remains unrecovered. Library delivery remains blocked and
was not retried. Dependencies, toolchains and reference pins remain frozen.
No desktop, GPU, installation, push, merge or release occurred in this resumption.
