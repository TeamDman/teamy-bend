# U32-backed literal readback checkpoint

Canonical non-scalar character escapes now load as the same U32-backed data as
explicit `Chr` constructors. The pinned parser and Base representation establish
this as a host-representation defect; no new semantic policy or exception was
needed. CPU parity remains incomplete.

Tested source: `087a646a33cac73307fd4adf3135fc3186553b53`. Preserved remote baseline:
`53e452d4c5d8641292aae3b66116071a8ec5ce02`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

[Validation](cpu-nonscalar-literal-validation.json),
[compact outcomes](cpu-nonscalar-literal-results.json.gz),
[review](cpu-nonscalar-literal-review.json),
[controls](../tests/fixtures/cpu-nonscalar-literal).

## Contract and correction

Pinned [Base](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L60)
defines `Char` as `Chr{code: U32}`. Its
[escape parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588)
accepts one to eight ASCII hexadecimal digits, case-insensitive `u`/`U`, without
scalar validation. [Literal lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1932)
builds the same numeric constructors. Both the
[interpreter printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1280)
and [C printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L5527)
preserve non-scalars as braced hex. The guide shows Char/String literal syntax
but adds no scalar-only restriction to this representation.

The port previously decoded through `Vec<char>`, refusing values that were
already legal through `Chr{55296}`. The private decoder now retains `Vec<u32>`.
Newly accepted non-scalars must satisfy the pinned one-to-eight-digit grammar.
Noncanonical lowercase non-scalars such as `\u{+D800}` and `\u{00000D800}`
retain their prior refusal. Existing lowercase scalar syntax discrepancies remain
unchanged. No public Rust signature or error-structure layout changes.

For example, this now checks and preserves code 55296:

```bend
import Base
def main() -> Char: '\u{d800}'
```

| Observation | Pinned upstream | Port |
| --- | --- | --- |
| Checking | Accepts | Now accepts; previously load refusal |
| Pure normalization | `'\u{d800}'` | `Chr{55296}`; full value graph agrees |
| Executable C | `'\u{d800}'` | Exact bytes/status/stderr match |
| Executable JS | Exit 1, scalar refusal | Same refusal |

Pinned [JS char_new](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L567)
validates scalars at its runtime boundary. That behavior remains unchanged.
Native/C raw IO retain their existing byte encoding, including non-UTF-8 output
for non-scalars; these bytes never become an unchecked Rust `char` or `String`.
Numeric text conversion still performs checked scalar validation. Non-Unix path
conversion remains checked; Windows execution was not performed.

The literal limit of 128 codes, expression/pattern and kernel-input depth guards,
work accounting, cancellation and runtime budgets are unchanged. Adjacent
surrogate escapes remain two codes, never an implicit surrogate-pair conversion.
False equality proofs still reject; both selected and default pattern arms are
covered. IO controls preserve the `before|` prefix before JS's runtime refusal.

## Measured results

- Standard gate: **1317 passed / 0 failed / 49 ignored**,
  195 groups, 973.382 seconds on the clean tested source.
- Focused checks: 12 literal/public-error tests, one numeric scalar-boundary
  test, nine integration tests; strict targeted Clippy passed. The earlier
  seven-test run is retained separately, not added as new coverage.
- Main matrix: **890 observations / 210 deterministic pairs / 470 singletons**,
  zero unexpected outcomes or timeouts, across 44 authored controls.
- All **120 corresponding C/JS observations match**: 78 successes and 42 JS
  refusals. There are 128 upstream execution observations; the extra eight are
  resource-guard cases (four C successes and four JS refusals). Port emission
  refused those programs, so port execution remains unrun.
- All **23 canonical C-output readbacks** check and normalize. Integration
  controls compare complete literal/constructor graphs, including surrogate
  boundaries, U32 maximum, both escape cases and quote forms.
- Forty comparable historical cases demonstrate 18 checking acceptance fixes
  and 19 native-status improvements: 18 pure-normalization cases and one actual
  native IO case. Four later selected-pattern/proof controls have no fabricated
  baseline. True proofs also emit and execute successfully in both C/JS lanes;
  false proof emission refuses before producing an artifact.
- Separate 21 renewals cover three unchanged upstream originals and four authored
  constructor controls: seven native pure outputs remain unchanged and 14 C/JS
  artifacts are byte-identical to the prior printer checkpoint. These renewals
  alone do not claim new executable runs; affected integration tests run them.

Current checking subset: **28 both accept / 2 exact rejections / 10 differing
rejections / 4 acceptance differences**. The four are two strict-IO exclusions,
one 129-code literal refusal and one 95-code kernel-input-depth refusal. The ten
remaining diagnostic differences cover malformed/noncanonical escapes,
multi-character Char syntax and the false proof. Their phases and exact streams
remain in the receipt; matching rejection is not exact diagnostic parity.

No inventory census was rerun. The last measured 140 non-IO residuals (109
acceptance, 31 rejection) and six oracle failures remain historical at c5edea8.
Coverage linkage stays 516/1302, including failures, with 786 unlinked entries:
256 checker rejection, 517 interpreter/compiled candidates, 12 interpreter-only
and one no-main module. This authored batch does not add upstream inventory rows.

## Reproduction and preservation

Use the existing activated environment:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off PYTHONDONTWRITEBYTECODE=1
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=4
cargo test --locked --features extended_observability --lib literal_ -- --nocapture
cargo test --locked --features extended_observability --lib numeric_string_decoder -- --nocapture
cargo test --locked --features extended_observability --test cpu_nonscalar_literal --test cpu_c_escape_printer --test cpu_uppercase_unicode -- --nocapture
cargo clippy --locked --features extended_observability --lib --test cpu_nonscalar_literal --test cpu_c_escape_printer --test cpu_uppercase_unicode -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The C oracle uses its pinned `-std=c11 -O3 -lpthread -lm` profile. Port C adds
`-Wall -Wextra -Werror -pedantic`; both run with `--threads 1 --gpu off`.
Node executes JS. Individual matrix commands have 30-second timeouts. Host Cargo
debug overrides do not change Bend optimization or resource policies.

Exact commands, source/stream/artifact hashes and retained differences are in
compact receipts. Invalid raw bytes remain lossless base64; root substitutions
are disclosed separately from raw stream hashes. Case-only draft fixture names
were replaced by portable `lower`/`upper` names; the explicit mapping preserves
all historical baseline source bytes and paths.

The exact baseline executable was not separately retained before Cargo replaced
its disposable target. Its source, observed hash and all 158 baseline outcomes
are retained; the earlier printer snapshot has equivalent production sources.
The new clean executable has its own verified lossless snapshot. The final gate
binary differs in 23 bytes under the same source/features/profile; six native
outcomes and 12 emitted artifacts were renewed identically. Both hashes remain
explicit. A verified 23-byte delta preserves that final binary from the existing
snapshot without duplicating it. The earlier
lost historical focused log also remains unrecovered; no recovery is claimed.

Matrix SHA-256: `fee913642d828fdaebb93e9c357fc80a8b6fd58fcef939a4aab3c53bdbf55abb`. Compact results SHA-256: `31dd1e4df8de64a0a73167cae8faaf4a9a4584ccebd8bbdafd38099b684c9b9d`.
Final gate log SHA-256: `f13630843f1cfc6b81108f7bdb683a29154e8da4db95c1ce44f98ea2300a4225`.
A verified private archive retains 139 evidence records,
855934 logical bytes, SHA-256 `d066ce7bc074a22663f88399cf6fb9c622a6cad7862cfa4796c879fec4845167`. Raw generated artifacts and
binary snapshots remain local, outside Git preservation. Old cleanup records and
backups remain unchanged. No Library retry occurred.

Workspace floors remain 11 GiB before builds and 10 GiB during work; temporary
storage stays above 1 GiB. Gate minima: 12.173 GiB workspace / 2.121 GiB
temporary. Dependencies, toolchains, upstream and docs pins remain frozen.
No desktop, GPU work, installation, main merge or release occurred.

Next independent gap: malformed uppercase-escape diagnostic context, beginning
with `\U{}` and overflow, against the pinned parser. Existing lowercase scalar
acceptance discrepancies also require separate qualification. Native constructor
presentation, `nat_first`, compact lowering, Base coordinates, external invoke
audit, supplementary lints, F32/foreign/source-hole/resource and older oracle
issues remain open; this change adopts no exception for them.
