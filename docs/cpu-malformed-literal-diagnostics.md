# Malformed-literal diagnostic checkpoint

Tested source: `cfc326bcff94c1f7820e968c2abc6ca55270d643`; baseline checkpoint
`39a9442dfbd5f85a01461f257a5ee61379206027` (tested source
`4d62ceac607c4050c1d53ded536feb837118bf89`). Upstream stays
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; teamy-docs stays
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Two of the four requested originals
are corrected; two remain non-exact. This is not full CPU parity.

## Corrected grammar-owned diagnostics

The unchanged `parse/invalid_string_escape` and `parse/word_overflow_literal`
originals now match pinned load diagnostics, including exact expected/observed
text and the original source window. Both already rejected with exit 1.
The old public parser error was constructed after advancing to the next token,
losing the consumed literal's diagnostic position.

A private typed decoder failure now carries the byte offset after an unknown
escape. For `"a\qb"`, upstream consumes `q` and reports the following `b`.
Overflow metadata retains the original decimal digits and observes the character
immediately after them, including CRLF or EOF. Non-inline failure-only helpers
attach metadata after the existing grammar-owned rejection; they do not move
public error cursors or reparse source.
[Pinned escape parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588),
[numeric parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2210),
[failure observation](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1493).

Successful decoding still follows the same character/escape sequence. A direct
regression compares actual values for newline, tab, CR, NUL, slash, both quotes,
emoji, BMP text and U+10FFFF. Metadata uses source-owned token positions, including
imported and template-replayed tokens; it does not copy or alter core references,
term identity or sharing. Resource guards, work charging, runtime demand and
cancellation boundaries are unchanged. No mid-normalization cancellation is added.

Public `ParseError` fields, Debug, Display, messages and next-token positions stay
unchanged. This is verified against a probe linked to the previous source/library,
not only by comparing two paths through the new loader. Private metadata/Debug
and CLI presentation change. The newly formatted refusals use the existing
`Ok(CliOutput)` with exit code 1; production repository dispatch forwards it.
External invoke consumers remain unaudited and must inspect `exit_code`.

## Retained lexical and Unicode gaps

`parse/hex_no_digits` and `parse/unclosed_char_code` remain non-exact. The port's
eager lexer rejects before the parser knows the grammar position:

| Source position | Pinned expectation | Port refusal |
| --- | --- | --- |
| `def main(): 0x` | numeric literal; observes `x` | invalid numeric suffix |
| `def 0x(): 0` | name; observes `0` | invalid numeric suffix |
| backtick in a function body | term | unexpected character |
| backtick where a declaration name belongs | name | unexpected character |

An earlier `def (): 0` followed by either malformed token makes upstream report
the earlier name error; eager lexing reports the later token. Blanket category
mapping or prefix heuristics would therefore invent diagnostics. Independent
review recommends a future lexer/parser context integration with an explicit
first-error-order contract. No diagnostic parser replay, acceptance change or
public-error compatibility change was adopted to make these two fixtures appear
qualified. Their original C/JS generation and CLI operations still refuse at load;
execution is unrun.

The pinned Unicode regex is case insensitive, so `"\U{41}"` is accepted upstream
and rejected here. That separate acceptance difference is retained. This authored
control is outside the 1,302-original census and does not change its denominator.
Malformed Unicode escapes and UTF-16 surrogate observations also remain non-exact. The
small unknown-escape correction conservatively excludes uppercase U and astral
escape/observed characters; these are unchanged gaps, not new policy exceptions.

## Validation

Standard gate: **1298 passed / 0 failed / 49 ignored**, 1115.213 seconds, at the exact clean source above.
The full gate renews prior stack, shared-reference, aggregate-work, partial-call,
import, literal, writer/flush, cancellation and safety-limit regressions. Before
that checkpoint, five focused unit tests, one expanded failure-boundary test and
67 integration tests passed. Final strict targeted Clippy passed. The new decoder
unit checks actual values in addition to the runtime length control.

[Complete compact observations](cpu-malformed-literal-diagnostics-results.json.gz)
retain **598 commands / 290 deterministic within-lane pairs**, with no unexpected
outcomes or timeouts:

- Main matrix: 364 commands, 173 pairs. Thirteen negative cases have 234 exact
  CLI stream observations and 52 separate strict/executable load-phase probes.
  Cases cover original failures, EOF, CRLF, leading zeros, character/BMP literals,
  imported/template provenance and first-error ordering.
- Three valid programs return `4294967295`, `0` and `10`: 24 generated C/JS
  executions, 12 pure-normalization observations, 24 checking/phase commands and
  18 generation/build commands. Six port artifacts match baseline bytes. Complete
  port normalized structures are tested separately from text. Native pure run
  is normalization, not IO runtime evidence.
- Direct originals: 88 commands, 44 pairs across checking, phase probes and C/JS
  generation plus port eval/run/compile boundaries. Two exact originals and two
  retained diagnostic mismatches remain distinctly labelled; no refused-source
  artifact was emitted and no refused-source execution is claimed.
- Adjacent/API matrix: 146 commands, 73 pairs. Ten lexical/Unicode controls retain
  their baseline port streams; uppercase U retains its acceptance disagreement.
  Public load and executable-load errors match the old binary for 23 fixtures,
  each tested twice, including complete Debug and Display output.

Each differential child is bounded at 30 seconds. Upstream C uses its actual
`-std=c11 -O3 -lpthread -lm` profile; port C additionally uses
`-Wall -Wextra -Werror -pedantic`. [Validation receipt](cpu-malformed-literal-diagnostics-validation.json)
records commands, clean source, binary/library/probe hashes, profiles and log
identities. Published streams replace machine roots only and retain raw hashes.

Reproduce repository checks from the checkout after existing tool activation:

```sh
export PYTHONDONTWRITEBYTECODE=1 CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_BUILD_JOBS=4 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --offline --features extended_observability --lib literal_diagnostic -- --nocapture
cargo test --locked --offline --features extended_observability --lib literal_decoder -- --nocapture
cargo test --locked --offline --features extended_observability --lib pattern_load_boundary -- --nocapture
cargo test --locked --offline --features extended_observability --test cpu_malformed_literal_diagnostics --test cpu_missing_import_diagnostics --test cpu_template_occurrences --test kernel_core --test kernel_normalization --test kernel_open_closures
cargo clippy --locked --offline --features extended_observability --lib --test cpu_malformed_literal_diagnostics -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The [checking census adapter](../scripts/checking-census/README.md) remains
unchanged. Host debug/cache overrides are recorded separately from Bend's O3
runtime profile and budgets. Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0,
Clang 19.1.7 and PS 7.5.4 remain pinned. Dependencies and lockfiles are unchanged.
Git's full whitespace check flags exact upstream/golden trailing spaces and CRLF;
those fixture bytes are preserved. The Rust source whitespace check passes.

## Census and remaining requirements

The renewed 1,302-fixture census measures 366 both accepted, 403 exact rejections,
495 acceptance disagreements, 33 different rejections and five oracle failures.
All 385 historical IO-cohort observations renewed exactly. Outside that cohort,
143 differences remain: 110 acceptance + 33 rejection. Only the two targeted port
stderr streams changed; every port status and phase is unchanged. Upstream
dead_param_fork changed from a host-stack failure to acceptance, creating one
additional measurable acceptance disagreement. This is oracle variation, not
a port regression or a third improvement. No new exclusion is added.
[Reconciliation](cpu-malformed-literal-diagnostics-census.json) and
[three-row delta](cpu-malformed-literal-diagnostics-census-delta.csv) preserve
the actual source/observation identities. Both-accept is checking status only.

Coverage linkage is 513 linked / 789 unlinked, including two newly linked retained differences.
Unlinked applicability: 256 checker rejection, 520 interpretation and compiled candidates, 1 checking without main, 12 interpretation only. Linked receipts do not establish parity.
The two lexical-context originals, uppercase-U acceptance and Unicode observations
remain open. Prior nat_first ordering, unadopted compact Nat/string lowering,
Base coordinates, external invoke audit, supplementary all-target lint failures,
source-hole, F32, foreign, resource and oracle limitations remain unwaived.
Desktop/GPU, supplementary all-target Clippy and newer upstream adoption are unrun.

Next architectural work is a context-aware lexical diagnostic boundary that can
retain public loader error compatibility and prove first-error ordering under
existing limits. The declaration-position and earlier-error controls above are
required counterexamples for that work; token-category substitutions are insufficient.

## Evidence preservation and incident

All 21 historical cleanup/restore records retain their hashes; no new cleanup or
Library upload was attempted. The unchanged 11 GiB build preflight, 10 GiB active
workspace floor and 1 GiB temporary floor remain enforced. Gate minima:
12.177723 GiB workspace / 2.392 GiB temporary storage.

One evidence-runner mistake lost a historical successful focused-run raw log.
The copied runner retained the previous batch's output directory and checked only
for an existing JSON receipt. That historical run had a log but no JSON, so the
new unit run overwrote `cpu-missing-import-diagnostics/focused-unit.log`.
The new log was moved into this batch. The old raw bytes cannot be recovered from
known retained copies; their published hash remains. The later focused-unit-final
log, prior clean full-gate log, all failure receipts and source/restoration records
remain intact. This loss does not invalidate the independently renewed full gate,
but it is not described as successful preservation of all raw logs.

The runner now uses its own script directory, restricts names to basenames and
creates JSON/log files exclusively. A regression confirms repeated and escaping
names cannot change existing evidence bytes. [Incident record](cpu-malformed-literal-diagnostics-evidence-incident.json).
A new private lossless backup verifies 1,399 retained records (3,236,685 logical
bytes; 479,560 archive bytes), including the surviving prior focused/full-gate
logs. Originals remain in place. Its SHA-256 is
`f79acf496530fd4a4843edbe972c761e7c9be371fab513c291f81afef2df80c9`.
It does not recover the lost log.

Bulk logs, binary snapshots, private archives and the two failed Library deliverables
remain local and require separate transfer when preserving the environment.
Compact receipts/source are in Git. No Library IDs exist; the connection-change
retry gate remains in effect.
