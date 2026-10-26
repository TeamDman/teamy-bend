# Character closing-quote diagnostic checkpoint

Tested source: `6c26f207626582deb04be75885325cfc9389197d`; baseline main: `da1cb0e7a89641253bbfc969575fa9e29cab8229`.
Upstream remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

A Char literal must close immediately after its first decoded code. For example,
`'a\u{g}'` now reports `expected : a closing '` at the backslash, matching pinned
upstream, while the raw public loader still returns its original invalid-Unicode
error and original next-token cursor. Acceptance, evaluation, limits and first-code
escape rejection are unchanged.

Pinned evidence: [parse_char](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588),
[Char parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1930),
and [parse_take/parse_fail](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1493).
`parse_take` skips no trivia: immediate spaces, tabs and comment markers remain
observable. The private single-code decoder is shared with ordinary literal
decoding; optional provenance replays only one canonical code and bounded Unicode
lookahead. It does not rescan a malformed unbounded Unicode prefix. This adds a
nonrecursive helper call in unoptimized host builds; no zero-overhead claim is made.

## Measured results

- Standard gate: **1327 passed / 0 failed / 49 ignored**, 195 groups,
  1019.341 seconds, clean tested source above.
- Focused parser tests: 20 passed. CLI integration: seven passed. Strict focused Clippy passed.
- New 31-negative cohort: **5 exact / 26 different before → 25 exact / six different after**;
  **20 diagnostic closures**, no acceptance changes. Six closures are retained
  earlier Char fixtures, referenced unchanged rather than rewritten.
- 751 differential observations, 122 deterministic repeat pairs, 507 singletons,
  zero timeouts. All **62 raw loader observations** (strict/executable) unchanged.
- 14 positive controls: **56 corresponding C/JS execution matches**, 48 successes
  and eight required JavaScript scalar refusals. **13 native pure outputs and one
  native IO output** unchanged; **28 generated artifacts byte-identical** to baseline.
  C uses the actual upstream `-O3` profile, with strict warnings additionally required
  for port C. Pure CLI normalization equality to baseline is not complete upstream
  graph equality; that stronger comparison was not rerun here.
- Unchanged pinned originals: invalid string escape, unclosed backquote char-code,
  string escape printing and Unicode Char printing. The backquote original is a
  lexical refusal, not evidence that a quoted Char executed.

Full-gate log SHA-256: `c95dc0b0c643879e463b7b4e82a8e5a2112076112b15d7104f7a832d0441d251`.
Raw matrix SHA-256: `d282a71210704ab91c3ff090088d4de7e1d26f20ab195a9f8d14a5d15653a2c9`.
[Validation receipt](cpu-character-quote-diagnostics-validation.json) and
[compact exact observations](cpu-character-quote-diagnostics-results.json.gz)
include commands, source hashes, statuses, stderr/stdout, repeat and timeout data.
Public streams replace environment roots with explicit placeholders and retain
raw-stream hashes. Immutable originals, runners and logs remain in local evidence
and a verified preservation archive; binaries/generated programs are not in Git.

## Residuals and compatibility

Six differences in this cohort remain unwaived: astral observed successor, empty
Char, unterminated Char, multiline Char, the 129-code length guard and earlier grammar. The preserved invalid template/import probes never reached Char parsing; their
earlier grammar diagnostics already matched baseline and still match. Valid
template/import controls are distinct. The matrix runner required exact equality
for 23 cases and conservatively labeled the other eight as different. Independent
byte comparison establishes the two unchanged grammar matches; the explicit
classification correction in the compact receipt supersedes those two labels.
Raw observations and original labels remain preserved. Empty `''` needs
a separate contextual diagnostic treatment because upstream consumes the apparent
closing quote as its first code. No blanket lexer mapping was added.

Overlapping current cohorts are uppercase 35 exact/four different (39), lowercase
grammar 33/one (34), and lowercase malformed 27/three (30). These are metadata updates
for the same six closed observations, not six additional improvements. Historical
receipts remain tied to their original commits. The whole census was **not** rerun:
its last 140 non-IO differences at `c5edea8` (109 acceptance/31 rejection) and six
upstream stack failures remain historical measurements, not inferred new totals.

Public `ParseError` fields, Display and loader rejection remain unchanged. Optional
formatted diagnostics can make command `invoke` return `Ok` with `exit_code=1`
instead of `Err`; repository dispatch checks exit status, but external consumers
remain unaudited. No public signature was added or changed in this patch.

The pending `nat_first` C diagnostic-order decision, compact Nat/string lowering,
Base coordinate differences, foreign/Bun/host/oracle limits, supplementary all-target
lint failures and other recorded parity gaps remain open. Strict IO proof exclusion
is distinct from runtime success. No GPU/desktop checks, dependency changes,
security changes, raised limits or new semantic exceptions. No claim of full CPU parity.

## Reproduction and preservation

Use the pinned tools and locked offline dependencies. Host-only build overrides:
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`, jobs four;
`CARGO_NET_OFFLINE=true BEND_GPU=off`. These host debug settings do not change Bend
runtime optimization profiles.
The differential snapshot uses all features (including Tracy); the gate builds all
features and tests `extended_observability` without Tracy, producing a different
host binary. Both hashes/profiles are preserved. A bounded 73-observation bridge
checks the non-Tracy binary against the matrix: 31 negative CLI checks, 14 native
outputs and 28 byte-identical C/JS artifacts. Emission status/stderr and all metadata
fields match except the intentionally distinct output destinations, each checked
against its exact command. The first bridge attempt stopped at that path difference;
its 33 observations remain preserved and are not counted as a complete bridge. The all-features snapshot is separately
reproduced by an identical recorded build. This is bounded profile agreement, not
universal binary or cross-profile equivalence.

```sh
cargo test --locked --offline --lib literal
cargo test --locked --offline --test cpu_malformed_literal_diagnostics
cargo clippy --locked --offline --features extended_observability --lib --test cpu_malformed_literal_diagnostics --test cpu_nonscalar_literal --test cpu_c_escape_printer --test cpu_uppercase_unicode -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The matrix runner source is included in the compact receipt with its SHA and
explicit environment-root substitutions. Each child has a 30-second timeout; the
full-gate wrapper has a 1200-second timeout. Disk floors remain 11 GiB before builds,
10 GiB active workspace and 1 GiB temporary storage; gate minima were 12.168 GiB
and 1.823 GiB. No cleanup was needed.

Two setup failures are retained, not counted as passes: an initially missing test
metadata field (19 passed/one failed) and Clippy's equivalent inclusive-range
suggestion. Both were corrected before the clean tested source. Earlier invalid
template/import probe sources and their failures are retained separately.

Review and full validation precede feature publication and a non-force expected-SHA
main update. The receipt commit contains documentation only after the tested source;
remote identities are verified independently. No branch deletion or release.
Library upload remains blocked pending a connection change; no retry was made.
