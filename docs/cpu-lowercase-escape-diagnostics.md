# Lowercase malformed-escape diagnostic checkpoint

Malformed lowercase Unicode escapes now retain their escape position for the
formatted diagnostic while preserving raw public parser errors and rejection.
No accepted value, evaluation rule, resource limit or semantic exception changed.
CPU parity remains incomplete.

Tested source: `d79486efef4cb6baf6d94515c9ee764b2fedeb77`.
Baseline: `c99e74a1556539c187bef7803a252919e19d835e`.
Upstream: `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
The baseline was separately consolidated onto main; this diagnostic checkpoint
was kept separate during validation.

[Validation](cpu-lowercase-escape-diagnostics-validation.json),
[compact outcomes](cpu-lowercase-escape-diagnostics-results.json.gz),
[review](cpu-lowercase-escape-diagnostics-review.json),
[fixtures](../tests/fixtures/cpu-lowercase-escape-diagnostics).

Pinned [parse_char](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588)
rejects malformed braced escapes immediately after consuming `u`. The port's
legacy scan/radix/non-scalar failures retained their message but discarded that
position. A private `MalformedUnicode` error now carries both. The existing
diagnostic formatter uses the saved position under its unchanged String/first-Char,
ASCII/BMP and single-line restrictions; the raw `self.error` cursor and message
remain unchanged. Ordinary unfinished escapes and lexer failures are separate.

## Results

- Full standard gate: **1324 passed / 0 failed / 49 ignored**,
  195 groups, 1010.220 seconds, on clean tested source. Formatting,
  standard strict Clippy, all-feature build and Node provider checks passed.
- Earlier focused controls: **16 unit / 15 integration tests passed**, targeted
  strict Clippy passed. Original dirty-state receipts are retained; the full
  gate renews the tests on the committed source.
- Thirty authored negatives, including fourteen unchanged existing sources:
  **3 exact / 27 differing diagnostics before; 26 exact / 4 differing after**.
  This is **23 diagnostic improvements**, with no acceptance change or previously
  exact regression. Baseline contains 120 observations.
- Matrix: **620 observations / 98 repeat pairs / 424 singletons**, no timeout.
  Every one of the 60 strict/executable public-error probe outputs equals its
  baseline, including fields and Display. Probe exit zero means observation
  completed, not that a malformed source was accepted.
- Refusals cover checking, strict evaluation, native-run entry, upstream/port
  C/JS emission and port native emission. No refused command creates an artifact.
  The unchanged pinned unknown-escape original remains exact. Rejected programs
  are not executed.
- Nine accepted controls produce **36 corresponding C/JS matches**: 28 successes,
  eight required JS scalar refusals. Eight native pure outputs and one native IO
  output match baseline; all 18 emitted port artifacts are byte-identical.
  Canonical U32 codes remain intact. Strict IO checking remains excluded.
  Upstream pure normalization was checked for successful exit, not complete
  graph/text equality in this matrix.
- Default-stack literal/expression guards, complete constructor values, input
  depth, false proofs, sharing, work accounting and existing cancellation
  boundaries remain covered. No mid-normalization cancellation is claimed.

Four new-cohort residuals remain unqualified: later-character closing-quote
ordering, cross-line literal handling, an astral observed unit and earlier
function-parameter grammar. Their exact streams remain in the compact receipt.
The earlier 34-case grammar cohort now requires 32 exact / two differing
diagnostics; its thirteen improvements overlap this batch and are not additional
closures. Historical 19/15 measurements remain unchanged. The older
`unicode-invalid` control moved explicitly from unresolved to exact; its raw
source bytes and public error are unchanged. The two older UTF-16 controls remain
unresolved.

The optional diagnostic changes eligible command failures from `Err` to
`Ok(exit_code=1)`. Process status remains one. Repository dispatch handles both;
external `invoke` consumers remain unaudited. No public type layout changed.
Exact upstream excerpt goldens include trailing spaces on empty source lines:
unfiltered `git diff --check` reports those intentional bytes. Non-golden
whitespace checks pass; goldens were not normalized to silence the check.

No whole census was rerun. The measured historical census at c5edea8 remains
140 non-IO differences (109 acceptance / 31 rejection) plus six upstream stack
failures. Authored fixes do not establish revised whole-inventory totals.
Linkage remains 516/1302, including failed receipts; 786 unlinked entries are
256 checker rejections, 517 interpreter/compiled candidates, 12 interpreter-only
and one no-main module. Existing `nat_first`, unadopted compact lowering, Base
coordinates, external invoke audit, supplementary all-target Clippy and older
F32/foreign/source-hole/resource/oracle gaps remain unwaived.

## Reproduction and preservation

Use the activated frozen environment from the repository root:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off PYTHONDONTWRITEBYTECODE=1
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=4
cargo test --locked --features extended_observability --lib literal_ -- --nocapture
cargo test --locked --features extended_observability --test cpu_malformed_literal_diagnostics --test cpu_nonscalar_literal --test cpu_c_escape_printer --test cpu_uppercase_unicode -- --nocapture
cargo clippy --locked --features extended_observability --lib --test cpu_malformed_literal_diagnostics --test cpu_nonscalar_literal --test cpu_c_escape_printer --test cpu_uppercase_unicode -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Every differential command, source hash, status, stream and artifact hash is
retained. Explicit root substitutions apply to paths and decoded byte streams;
original stream hashes remain. Each matrix command has a 30-second host timeout.
C uses `-std=c11 -O3 -lpthread -lm`, with `-Wall -Wextra -Werror -pedantic` for
port C. Generated programs use `--threads 1 --gpu off`; Node runs JS. Host Cargo
debug overrides do not change Bend runtime optimization or safety budgets.

Raw matrix SHA-256: `c8d744cd9872fa6574b18d0ac21b7e12b4548c953ae6a9754aed30eb75789b14`.
Full-gate log SHA-256: `b04a71d15e342da794af9ba9395ce8ccafa5060be46796629ede67525c2a385a`.
The final gate executable is byte-identical to the matrix snapshot:
`b9034499766f3194d8eb42e7218e5b8d34da96f272fb240b47b1125f654f59cc`.
That byte comparison does not claim extra behavioral executions.

Gate storage minima: 12.169 GiB workspace / 1.903 GiB temporary. Floors
remain 11 GiB before builds, 10 GiB active workspace and 1 GiB temporary. The
reviewed c99e74a branch and both earlier backups remain unchanged. New source
and unique evidence are preserved separately. The old lost-log limitation and
blocked Library delivery remain explicit; no retry occurred. No dependency,
toolchain, upstream reference, guard, desktop or GPU hardware change occurred.
No release was made. Current diagnostic work was not part of the earlier main
fast-forward; its publication status must be reported separately.

Next independent candidate: the retained later-character closing-quote diagnostic
ordering, with earlier grammar and lexer boundaries kept distinct. No correction
or semantic policy choice for those residuals is adopted here.
