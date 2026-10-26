# Raw apostrophe compatibility checkpoint

Tested source: `61cab3744713e4841f44efd8b8bd3853e1026206`. Baseline main: `878c0bb3860b8fd590c0949680a2ea263ff048d9`;
local diagnosis: `b9ac3c20fc78782430c5083549680e9ad99988d7`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

The port previously read `'''` as an empty character followed by an unfinished
literal. It now accepts that spelling as apostrophe code 39, as pinned upstream
does. Six/nine adjacent quotes also supply two/three character arguments where
argument separators are optional. Token text, byte spans and template replay
remain intact; no semantic parser or template specialization is replayed.

Pinned mechanisms: [character decoding](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588),
[one-code Char grammar](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1930),
and [optional argument separators](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2159).

## Results and scope

- Standard gate: **1334 passed / 0 failed / 49 ignored**, 196 groups,
  1016.364 seconds at the clean source above. Strict Clippy, build and real
  TCP/UDP socket-provider controls also passed.
- Focused controls: **five unit and two integration tests passed**; targeted strict
  Clippy passed. Coverage includes retained template-body tokens, distinct specializations,
  cached reuse, imports, adjacent arguments, patterns, source spans, stale diagnostic
  clearing, literal-size refusal and internal parser-depth refusal.
- Authored cohort: **12 accepted controls / 17 load refusals**. Ten of the accepted
  controls were previously refused (nine strict-check repairs plus one executable IO repair);
  escaped apostrophe and string content are unchanged
  positives. Three original diagnosis sources are byte-identical. These are authored
  controls, not new upstream inventory links.
- **513 differential observations / 106 deterministic repeat pairs / zero timeouts**.
  **96 generated executions** (12 controls × upstream/port × C/JS × two runs) all
  succeed; the 48 corresponding port/upstream comparisons match exactly, including stderr/status.
  C uses upstream's actual `-O3` profile; port C additionally passes strict Clang warnings.
- Eleven pure raw/escaped pairs have equal complete normalized value graphs and
  native pure output; the IO pair has equal actual native IO output. This does not claim
  complete graph equivalence to upstream. Native pure output `Chr{39}` differs from
  quoted upstream/C/JS presentation; generated programs print the escaped apostrophe
  spelling. Adjacent argument controls produce `78` and `117`; IO prints apostrophe/newline.
  Strict proof checking still refuses the IO program; executable checking succeeds.
- The 17 negatives remain load refusals through check/eval/run and C/JS/native emission;
  no refused artifact is created. **Three exact upstream diagnostics / 14 retained
  diagnostic differences**, both before and after; no diagnostic closure is claimed.
  Matching phase is not exact diagnostic parity.

[Validation](cpu-raw-apostrophe-validation.json) and
[compact observations and runners](cpu-raw-apostrophe-results.json.gz) preserve
commands, source hashes, streams, statuses, timeouts and repeats. Public copies
substitute explicit environment-root placeholders and retain hashes of original
streams. Raw matrix SHA-256: `0e77d7f32b7efe17cb74ee08e7acd1c92b2d8f4bb9b64265c71cf4e9e78cbd59`.
Full gate log SHA-256: `ba069fc0c163c0168e2d107fa04b1964e5993bab0a54e780402731abbab7afe0`.
The source-built probe is byte-identical to the earlier candidate probe; its
independent rebuild receipt is retained. The gate's final non-Tracy executable is
byte-identical to the differential snapshot, avoiding an unproved profile substitution.

## Compatibility and remaining gaps

The trial tokenization falls back to the whole legacy lexical result if it fails.
If canonical tokens reach a parser failure, an existing global lexical refusal
is restored and incompatible optional diagnostic metadata is cleared. There can
be extra lexical scans; no zero-overhead claim is made. No parse-state replay,
Term changes, evaluation-order change or guard exemption was added.

Fifteen negatives retain exact raw public load-error fields/Display. Two changes
are explicit: an invalid declaration name now reports the canonical three-quote
token instead of an empty-character token; a repaired imported module loads
successfully and exposes a later root-file grammar error. Public signatures and
error types are unchanged. Newly exposed formatted failures can use the existing
`invoke` result `Ok` with `exit_code=1` instead of `Err`; in particular, the repaired
import changes which module reaches that boundary. External consumers must inspect
the status and remain unaudited. This is not universal counterfactual preservation of
errors in programs containing newly accepted syntax. Internal 64-level parser
and 128-character literal guards remain unchanged; an older eager lexical refusal
can still be the public error for an input that reaches the canonical depth guard.

Fourteen diagnostic differences in this cohort remain unwaived: empty/unfinished
quote forms, malformed templates, earlier/later grammar, lexical-after and the
import-later-error observed-token difference. `positive/unused-template.bend`
retains its exploratory path, but its manifest and results correctly classify it
as rejected. No whole census was rerun; the last 140 non-IO differences at `c5edea8`
(109 acceptance / 31 rejection) and six upstream stack failures remain historical,
not inferred new totals. Original inventory linkage remains 516/1302, with failed
and partial receipts included; it does not mean completed CPU parity.

Pending `nat_first` C diagnostic order, unadopted compact Nat/string lowering,
Base coordinates, external `invoke` consumer audit, F32.read, foreign/Bun/host/oracle
limits and supplementary all-target Clippy failures remain open. No mid-normalization
cancellation claim or new semantic exception. No GPU/desktop checks, dependency or
toolchain changes, raised budgets, security changes, branch deletion or release.

## Reproduction and preservation

Use the existing locked/offline toolchain. Host overrides:
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`, jobs four,
`CARGO_NET_OFFLINE=true BEND_GPU=off`. These differ from Bend's C `-O3` optimization.

```sh
cargo test --locked --offline --features extended_observability --lib raw_apostrophe_tests
cargo test --locked --offline --features extended_observability --test cpu_raw_apostrophe
cargo clippy --locked --offline --features extended_observability --test cpu_raw_apostrophe -- -D warnings
cargo build --locked --offline --features extended_observability
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The compact receipt includes the bounded runner; substitute its explicit roots for
local checkouts/evidence. Each differential child is limited to 30 seconds and the
full gate to 1200 seconds. Disk floors stay 11 GiB prebuild, 10 GiB active workspace,
1 GiB temporary storage. Gate minima: 12.167 GiB workspace /
1.702 GiB temporary. No cleanup was needed. The initial
targeted Clippy failed on a test struct/field naming lint; it was corrected before
the clean tested commit, and the failed log remains preserved.

Unique source and evidence remain local with verified Git bundles and preservation
archives; generated binaries and large outputs are not in Git. Earlier cleanup
restoration records and the old denial are retained. The user's explicit renewed
approval separately resolved the previous main restriction: reviewed `878c0bb`
was verified on main before this work's publication. This checkpoint follows
independent review, full validation, feature preservation and a non-force expected-SHA
main update; remote identities are recorded separately after the documentation commit.
Library delivery remains blocked without a connection change; no retry was made.

Next bounded candidate: contextual empty-character/quote-count diagnostics, preserving
raw loader contracts and first-error ordering. The acceptance repair here does not
waive those diagnostic differences or establish full CPU parity.
