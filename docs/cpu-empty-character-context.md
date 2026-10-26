# Contextual empty-character diagnostic checkpoint

Tested source: `15568847ae21f8e917024edc0f39181fe21396dc`, following reviewed checkpoint
`79d4109f436510bf580310e35e1100c1427c0fe9`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. This validation is separate from the preceding reviewed main update.

The existing empty-character rejection now reports the immediate source character
where pinned upstream requires its closing quote. The raw public parser error,
acceptance, evaluation and all guards remain unchanged. One diagnostic mechanism
closes sixteen measured negative controls, not sixteen independent semantic defects.

Pinned [Char parsing](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1930)
consumes the second quote of `''` as code 39 through
[parse_char](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588),
then requires another quote immediately. The new private helper is reached only
at the existing cardinality refusal. It validates two quotes in the original
source and observes the next character, retaining EOF, LF/CRLF, tabs, spaces and
comments. Another apostrophe withholds metadata because longer quote runs have
a different parse history. Astral successors also withhold metadata: pinned
parse_fail observes one UTF-16 unit. The lexer, literal decoder, semantic terms,
sharing, evaluation, termination, work/depth/input budgets and cancellation paths
are untouched. The helper reuses existing source-excerpt rendering.

## Measured validation

- Standard gate: **1338 passed / 0 failed / 49 ignored**,
  197 groups, 1043.739 seconds. Strict Clippy, build and real TCP/UDP
  provider checks passed. Source/fixtures stayed clean during the gate.
- Focused tests: two new diagnostic unit tests, 21 literal regressions and two CLI/runtime
  integration tests passed. Corrected targeted strict Clippy passed.
- New cohort: 22 load refusals, five accepted executable controls; **acceptance unchanged**.
  Diagnostics improve **2 exact/20 different → 18 exact/4 different**.
  All 44 strict/executable raw error observations remain byte-identical to baseline.
  CLI check/eval/run and C/JS/native emitters preserve failure status 1 and create no
  refused artifact. Existing formatted diagnostics can make `invoke` return `Ok`
  with `exit_code=1` instead of `Err`; external callers must inspect status and remain unaudited.
- Clean-source matrix: **432 observations / 74 deterministic repeat pairs / zero timeouts**.
  **40 successful generated executions /20 exact corresponding upstream comparisons**.
  Port C additionally passes strict Clang warnings; primary C oracle uses pinned `-O3`.
  Ten C/JS artifacts are byte-identical to baseline. Four pure native outputs and one
  actual native IO output are unchanged. Strict IO proof exclusion remains explicit.
  The ordinary integration helper uses its default unoptimized C profile separately.
- Fresh 96-observation overlap renewal: earlier character-quote cohort **26 exact/5 different**
  (historically 25/6); raw-apostrophe negatives **8 exact/9 different** (historically 3/14).
  Overlaps are not additional diagnostic closures. Historical receipts retain their actual source identities.

[Validation receipt](cpu-empty-character-context-validation.json) and
[compact streams and runners](cpu-empty-character-context-results.json.gz) retain
commands, source/artifact hashes, statuses, timing and classifications. Public
copies substitute explicit roots and preserve raw-stream hashes. Raw clean matrix
SHA-256: `a05beb0248dcc4e0f6bf923c87b8580fc3a794b2b77bd8c4edfb46215e619f00`. Full-gate log SHA-256:
`3c367b4c0f0906e98a983a13e8cfa03cb241c35539428d274b052a5e6b0ca1ef`. The gate executable matches the final matrix binary.
The matrix's initial raw API probe was built at 71b08b3 with identical production
source; a fresh 15568847 probe independently reproduces all 54 matrix API observations.
That bridge is explicit, not an assumed binary identity or hidden feature substitution.

## Limits and provenance

Four differences remain unwaived in this cohort: earlier grammar, empty-followed-escape,
four-quotes and raw-before-grammar. They need their own grammar/lexical-context analysis;
this patch does not guess a new semantic policy. No full upstream normalized-graph
qualification or whole-census rerun was performed. The last 140 non-IO census differences
at c5edea8 (109 acceptance/31 rejection), six upstream stack failures and 516/1302 linked
originals remain historical. Linked receipts include failures/partials and are not full CPU parity.

`nat_first` remains pending; compact Nat/string lowering remains unadopted. Base
coordinates, F32.read, external invoke audit, foreign/Bun/host/oracle/source-hole/resource
limits and supplementary all-target lint failures remain open. No desktop/GPU execution,
new dependencies/toolchains, raised guards, security changes or release.

The initial 71b08b3 matrix observed CRLF working bytes that Git had normalized in one
committed fixture. The expected stderr was already byte-exact. A scoped `-text`
attribute plus explicit index refresh corrected the fixture; the complete matrix and
full gate were renewed at 15568847. Two mistaken preliminary identity assertions and
a missing-runner setup failure are retained, not counted as passes. The first API
bridge attempt also stopped before executing a case because its result filename
collided with the wrapper receipt; exclusive-create protection prevented overwrite,
and a distinct result filename resolved it. The initial test
Clippy also rejected unused C-helper functions; meaningful accepted-program execution
controls now use them, and the renewed lint passes. Intentional fixture/golden trivia
produces whitespace warnings from `git diff --check`; source-code checks remain clean.
The first 432-observation matrix is historical working-byte evidence, not the clean-source qualification.

## Reproduction and preservation

Frozen host overrides: `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0`, four jobs; `CARGO_NET_OFFLINE=true BEND_GPU=off`. These host
settings are separate from the Bend C `-O3` oracle profile.

```sh
cargo test --locked --offline --features extended_observability --lib empty_character
cargo test --locked --offline --features extended_observability --lib literal
cargo test --locked --offline --features extended_observability --test cpu_empty_character_context
cargo clippy --locked --offline --features extended_observability --test cpu_empty_character_context -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The compact receipt includes bounded replay scripts. Differential children use 30-second
timeouts; the full gate uses 1200 seconds. Disk floors remain 11 GiB prebuild, 10 GiB active
workspace and 1 GiB temporary. Gate minima: 12.165 GiB workspace /
1.604 GiB temporary. No cleanup was needed. Source bundles,
raw evidence and unique failure receipts remain preserved locally with verified archives;
large generated binaries are excluded from Git and separately fingerprinted.

This continuation began locally while the earlier `878c0bb → 79d4109` main update
was denied. The user then explicitly authorized continuing reviewed updates to main.
That pending update was retried once with the new approval, succeeded and was verified.
This newer checkpoint requires final review before feature preservation and a non-force
expected-SHA main update; later remote identities are recorded separately. No force push,
branch deletion or release is authorized. Library delivery remains blocked without
a connection change. Next bounded gap: context-sensitive lexical/grammar diagnostics
for the four retained cases, preserving raw errors and first-error order.
