# Quote and grammar diagnostic checkpoint

Tested source: `b383255b9b584408cb61927e7cd4b34567d78f99`.
Standard gate: **1342 passed / 0 failed / 49 ignored**,
197 groups, 1040.531 seconds. Formatting, strict standard Clippy,
build and real TCP/UDP provider checks passed. Final source remained clean.
Focused validation passed 94 syntax tests and eight CLI/runtime/template tests,
including default-stack guards, stale metadata, prefix caps and shared-template identity.

The final-source matrix has **600 observations / 98 repeat pairs / zero timeouts**.
Adjacent controls improved from 6 exact/6 different to 12 exact/0 different.
The post-gate CLI hash equals the matrix snapshot. [Validation receipt](cpu-quote-grammar-context-validation.json)
and [compact streams and runners](cpu-quote-grammar-context-results.json.gz) preserve commands, hashes,
statuses and failure classifications. Raw matrix SHA-256: `949ffe621fae159869e64e0210dd0bf1f336acb0f503700b36463c01c92adcdb`.
Full-gate log SHA-256: `e6afbb237a561b4af8820db2cdc5e61828f4e5c260f98124fa7c85170fb07cef`.
Gate minima: 12.162 GiB workspace / 1.558 GiB temporary.

The exact preceding main SHA is `333c6ba8dc60abc641cf3f5cf9721d4cfb9c45a6`.
The preceding final response omitted one character in prose; the remote and report
URL had the correct value. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

Three of the four retained diagnostic gaps now match upstream. Acceptance, raw
public parser errors, evaluation policy and safety limits remain unchanged.

- Pinned [parse_quant](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1665)
  consumes one marker before a name. A combined `->` token therefore observes `>`.
  Attribution applies only after a quantity parser; ordinary, already-quantified,
  compile-time and existential names keep their own cursor semantics.
- A later unterminated literal supplies the existing bounded diagnostic prefix.
  The earlier empty-character error can be reported, while the terminal incomplete
  token is refused before decoding. The original lexical error remains the raw
  loader result. Limits stay at 64 prefix tokens and 4096 source bytes, including
  trivia and the terminal fault; semantic/template/foreign barriers remain.
- Correct raw-apostrophe parsing already found the actual grammar refusal.
  Restoring the historical raw lexical error now retains that new optional diagnostic.
  Failure metadata is cleared at parse entry so guard refusals and over-cap lexical
  failures cannot inherit stale diagnostics. Source maps and shared terms are unchanged.

There is one constant optional-state check per atom and bounded failure-only capture;
this is not a claim of zero overhead. No second diagnostic parse was introduced.

## Scope and limitations

The original 22-negative cohort improves **18 exact/4 different to 21 exact/1 different**.
Twelve adjacent negative controls now match exactly. These overlap mechanisms and
are not independent semantic fixes. All 34 negative inputs still reject at load,
across checking/evaluation/run and executable C/JS/native emission. Refused artifacts
remain absent; 68 negative raw API observations are unchanged.

Five accepted controls preserve four pure native outputs and actual native IO output;
strict IO proof exclusion remains explicit. Forty generated executions provide
20 exact corresponding upstream C/JS comparisons. C uses the pinned O3 profile;
port C also passes strict warnings. Ten generated artifacts are identical to
baseline. The integration helper's default unoptimized C profile is separate.
The public API probe and CLI were both built from the final tested source.

`four-quotes` remains an actual diagnostic difference. Pinned parsing sees a raw
apostrophe followed by another quote; legacy tokenization yields two empty literals.
Synthesizing declaration wording at a Char refusal would misattribute other grammar
contexts. A second corrected-prefix parse needs isolated book/template/fresh-ID
state and bounded accounting justified separately; no such design is adopted here.

No whole census or earlier overlapping cohort renewal was run. Historical 140 non-IO
census differences at c5edea8 (109 acceptance/31 rejection), six upstream stack
failures and 516/1302 linked originals are not current inferred totals or universal
parity. `nat_first` remains pending and compact Nat/string lowering unadopted.
Base coordinates, F32.read, external invoke audit, host/oracle/foreign/Bun/source-hole/
resource limits and supplementary all-target Clippy failures remain open.
No new exception was adopted.

Formatted failures use the existing `invoke Ok(exit_code=1)` boundary instead of an
unformatted Err in newly covered cases. Raw loader errors stay unchanged. Repository
status handling remains under regression; external consumers are unaudited. No claim
of mid-normalization cancellation or new GPU qualification is made.

The first new test build failed because an expected-path String was moved inside a
loop; borrowing fixed the test harness. The next syntax run passed 93/94 and exposed
stale metadata outside the prefix cap. The entry-clear repair passed all 94 on renewal.
Both failed attempts remain preserved. A temporary report-generator quoting error
failed before writing a file; it had no source or validation effect. Exact golden
whitespace is intentional; source-code whitespace checks passed.

## Reproduction and preservation

Frozen host settings: `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true BEND_GPU=off`.
Dependencies, upstream and toolchains are unchanged. Commands:

Verified tools: Rust/Cargo 1.96.0, rustfmt from nightly-2026-05-24,
Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4.

```sh
cargo test --locked --offline --features extended_observability --lib syntax::
cargo test --locked --offline --features extended_observability --test cpu_empty_character_context --test cpu_raw_apostrophe --test cpu_template_occurrences
cargo build --locked --offline --features extended_observability
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The compact receipt includes matrix and public-API probe runners. Differential
children have 30-second timeouts; the full gate has 1200 seconds. Disk floors remain
11 GiB prebuild/10 GiB active workspace and 1 GiB temporary. Unique raw evidence,
source bundles and immutable failure receipts are retained locally; large generated
binaries are excluded from Git and fingerprinted separately. No cleanup, installation,
desktop activity, dependency change or release was needed. Library uploads remain
gated on a connection change. After final independent review, this checkpoint follows
the standing authorized feature/main non-force workflow. Remote identities are
recorded separately after publication.
