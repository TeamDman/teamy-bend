# Contextual lexical diagnostic checkpoint

Tested source: `c5edea8a75022c1654fbf568629422cf73bee1d0`; baseline checkpoint
`71893b97a5a6e5d8727618c67ca2802dce7147d9` (tested source
`cfc326bcff94c1f7820e968c2abc6ca55270d643`). Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; teamy-docs remains
`50e89d6897ce2a65167ef65b17c5fa83fe849158`.

Both unchanged originals, `parse/hex_no_digits` and `parse/unclosed_char_code`, now
match pinned load-rejection stderr and exit 1. This closes the two retained
lexical diagnostics from the preceding checkpoint; it is not full CPU parity.

## Grammar context without changing acceptance

The eager lexer still rejects malformed input. On diagnostic-enabled loading only,
it can retain a bounded prefix and one typed terminal fault. The existing parser
then interprets that prefix once to identify the actual grammar expectation. A
numeric suffix in a body expects a numeric literal and observes the suffix; the
same spelling in a declaration name expects a name and observes the digit.
A backtick in a body expects a term, while a name position expects a name.
An earlier malformed declaration is reported before the later bad token.

This is not a blanket lexical-category mapping, a second parser or source replay.
The original lexical `ParseError` is returned regardless of the diagnostic parse
result. No partially parsed book escapes. Successful parsing takes its existing
path. Public fields, Debug, Display, original first lexical error and positions
remain unchanged; CLI optional presentation can report an earlier grammar error.
The existing formatted `Ok(CliOutput)`/nonzero `exit_code` boundary applies.
Repository dispatch handles status; external invoke consumers remain unaudited.

The extra presentation path has a **64-source-token / 4,096-byte prefix cap**,
including the terminal fault and scanned trivia. Already consumed import headers
are absent from the parser's cleaned buffer. Caps are checked before copying or
rescanning fault text or constructing a parser. These are additional limits on
optional diagnostics; accepted input and existing language/runtime limits do not
change. This is not a claim of an absolute instruction-count bound: existing
book/name lookup remains part of parsing.

Temporary declarations, terms and binder IDs can be constructed in the failed
loader. The path stops with an explicit unsupported outcome before template
registration/specialization, macro substitution, foreign-path inspection,
completion of pre-existing laws, or nontrivial lowering. Only the existing
`Reply` plus empty-vars flattening identity path is allowed. This is sufficient
for the originals' preceding `law main: U32`. Unsupported prefixes and guard
refusals discard candidate metadata and retain the original diagnostic. Nothing
changes sharing, reference identity, runtime demand, cancellation boundaries,
work charging, execution limits or source-hole/structural guards. No
mid-normalization cancellation is added.

Pinned source basis: [parse_fail and grammar observation](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1493),
[name grammar](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1580),
[decimal literal grammar](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2210).
Nat/float suffix failures are not inferred from the shared legacy lexer message;
they retain the existing fallback. Backticks remain unsupported Bend terms.

## Validation and exact scope

Standard gate on the clean tested source: **1304 passed / 0 failed / 49 ignored**, 934.623 seconds.
Before that checkpoint, 73 syntax unit tests, 68 focused integration tests and
one expanded cancellation/writer/flush failure-boundary test passed. Targeted
strict Clippy passed. Five new unit tests cover exact contexts, actual barrier
outcomes, shared template identity, imported-law refusal, and bounds. Nat 129,
129-character strings and a Nat tail with 257 successors reach the existing parser guards
inside an eligible prefix; seeded diagnostic metadata is discarded. A separate
63-plus-fault versus 64-plus-fault control proves the token cap, and 4,096/4,097
byte controls include comments/trivia. Large malformed numbers do not allocate
fault text before the cap.

[Compact differential evidence](cpu-contextual-lexical-diagnostics-results.json.gz)
contains **734 observations / 358 deterministic within-lane pairs**.
All child runs are bounded at 30 seconds. No timeout or unexpected result occurred.

- Twenty-two negative cases cover both unchanged originals, name/declaration/body
  positions, nested arguments and arrays, annotations, imports, prior declarations,
  earlier errors, decimal suffixes, EOF and CRLF. Checking, six CLI boundaries,
  generated-C/JS refusal and independent strict/executable load-phase probes are
  distinct observations. Direct unchanged upstream fixture paths are also renewed.
- Twenty-six public-error cases compare complete old/new Debug and Display through
  both plain and executable loaders, twice. These establish public-error continuity
  separately from optional CLI formatting.
- Three existing valid controls still return `4294967295`, `0` and `10`.
  Twenty-four generated C/JS executions and twelve pure normalizations match;
  all six port-generated artifacts are byte-identical to the prior source.
  Complete normalized structures remain covered by the focused integration suite.
  Native pure run is normalization, not IO-runtime execution.
- Four Unicode controls retain baseline behavior and unresolved differences.
  Uppercase `\U{41}` remains accepted upstream and rejected here. Their agreement
  with the old port is not upstream parity. Refused-source execution is unrun;
  no artifact is emitted for a refusal.

Upstream C uses its actual `-std=c11 -O3 -lpthread -lm` profile. Port C also uses
`-Wall -Wextra -Werror -pedantic`. Rust 1.96.0, formatter nightly-2026-05-24,
Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4 remain frozen. The host debug/cache
overrides below do not change Bend's execution profile or limits.

```sh
export PYTHONDONTWRITEBYTECODE=1 CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_BUILD_JOBS=4 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --offline --features extended_observability --lib syntax:: -- --nocapture
cargo test --locked --offline --features extended_observability --lib pattern_load_boundary -- --nocapture
cargo test --locked --offline --features extended_observability --test cpu_malformed_literal_diagnostics --test cpu_missing_import_diagnostics --test cpu_template_occurrences --test kernel_core --test kernel_normalization --test kernel_open_closures
cargo clippy --locked --offline --features extended_observability --lib --test cpu_malformed_literal_diagnostics -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

[Validation receipt](cpu-contextual-lexical-diagnostics-validation.json) records
commands, source/tree/binary/probe hashes and profiles. The gate rebuild differs
from the differential snapshot only in its embedded
build timestamp and ELF build ID; both hashes are recorded separately. The matrix
and census are not relabelled as runs of the later binary. Private roots alone are
replaced in published streams; raw stream hashes are retained. The checking census
uses the unchanged [adapter](../scripts/checking-census/README.md).

Development failures remain recorded: the initial unit build had incorrect test
module imports; one control rejected before reaching its intended match-lowering
barrier; Clippy found an oversized definition function and then unused C helpers
in a negative-only integration module. Imports/control were corrected, lookup was
extracted without changing selection, and the new negative test was placed in the
existing integration module. No warning was suppressed or assertion weakened.
Exact upstream/golden trailing whitespace and CRLF remain intentional fixture bytes.

## Census, applicability and remaining work

The renewed 1,302-fixture census measures 405 exact rejections, 366 both accepted,
494 acceptance disagreements, 31 different rejections and six oracle failures.
All 385 historical IO-cohort observations renew exactly. Outside that cohort,
**140 differences remain: 109 acceptance + 31 rejection**. Only the two targeted
port stderr streams changed; all port statuses, phases and phase probes are
unchanged. Three upstream-only variations remain separate: `literal_rows_cubic`
and `dead_param_fork` changed from acceptance to host-stack failure;
`borrow_fork_hold` changed from host-stack failure to acceptance. These account
for one net residual-count decrease beyond the two diagnostic fixes, not a third
port improvement. No exclusion or oracle exception was added.
[Reconciliation](cpu-contextual-lexical-diagnostics-census.json) and
[five-row delta](cpu-contextual-lexical-diagnostics-census-delta.csv) preserve
source and observation identities. Both-accept is checking status only.

Coverage linkage remains 513 of 1,302 originals. The 789 unlinked entries comprise
256 checker-rejection cases, 520 interpreter/compiled candidates, 12 interpreter-only
cases and one no-main checking module. These categories are applicability, not
missing-runtime-test counts.

The two originals already had linked retained-difference receipts. Their new exact
qualification adds no new original to the linked inventory. Linkage and successful
checking are not execution coverage. Historical receipts stay tied to their tested
commits; the old unresolved-case manifest's six lexical controls now have exact
coverage here, while its four Unicode controls remain unresolved.

The next demonstrated acceptance gap is uppercase Unicode escape parsing
(`\U{41}`); this change does not adopt a new escape policy. Broader prefixes may
retain contextual diagnostic gaps where the presentation fence or semantic barrier
applies. Pending C `nat_first` ordering, unadopted compact Nat/string lowering,
Base source coordinates, external invoke audit, supplementary all-target lint
failures, F32, foreign, resource, source-hole and oracle limitations remain open.
No new exception is adopted. Desktop/GPU, newer upstream and supplementary
all-target Clippy remain unrun.

## Preservation

All 21 historical cleanup/restoration records, the prior private backup and both
Library deliverables retain their hashes. No cleanup, dependency change or Library
retry occurred. The prior successful focused-test raw log remains unrecovered;
its published hash and surviving later validation records remain available. New
runner paths are unique and evidence names are exclusive/basename-checked. Source
and compact sanitized receipts are preserved on the feature branch; raw logs,
binary snapshots and private backups remain local and require separate transfer.

The unchanged floor is 11 GiB before builds, 10 GiB during work and 1 GiB temporary
space. Full-gate minima: 12.176178 GiB workspace / 2.319 GiB temporary.
A new private lossless archive verifies 1,406 records
(3,130,245 logical bytes; 462,599 archive bytes),
with all originals retained. Archive SHA-256:
`467920fd8eb93c4429a0a0358886a59dcc22fe3fe9db4a017b5df472daf38ee7`. This does not recover the older lost log.
The private backup excludes generated binaries/artifacts; those remain local.
