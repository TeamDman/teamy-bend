# Malformed uppercase escape diagnostic checkpoint

Malformed uppercase escapes now receive the pinned Bend diagnostic when the
preceding literal grammar establishes that this is the first upstream failure.
Acceptance, raw parser errors and runtime behavior are unchanged. CPU parity
remains incomplete.

Tested source: `a28efb73ec5d78cf8f0125854799c19c51c9163a`. Preserved baseline:
`84a5652595fea158fa520a04530ff43ad4d72f6e`. Upstream remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

[Validation](cpu-uppercase-escape-diagnostics-validation.json),
[compact outcomes](cpu-uppercase-escape-diagnostics-results.json.gz),
[review](cpu-uppercase-escape-diagnostics-review.json),
[fixtures and original baseline](../tests/fixtures/cpu-uppercase-escape-diagnostics).

## Contract and correction

Pinned [parse_char](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588)
accepts one to eight hexadecimal digits in braced `u`/`U` escapes. If that grammar
fails, it consumes the escape letter and reports the next UTF-16 code unit.
The [Char parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1932)
then requires a closing quote after one code; Strings consume successive codes.

The existing optional diagnostic formatter excluded every malformed `U`.
It now tracks a sticky pinned-prefix flag during the existing decoder pass and
formats eligible refusals. An accepted noncanonical lowercase escape clears the
flag, preventing a later error from being presented as the first upstream error.
A Char failure is eligible only at its first code. Existing ASCII/BMP constraints
remain, and multiline lexical refusals are not remapped. No parser replay,
new term, sharing change or evaluation policy is introduced.

For `def main() -> String: "\U{}"`, both implementations reject during loading;
the port now reports the expected escape alternatives, observed `{` and the
original Bend source excerpt. It previously emitted the host error wrapper.
Canonical U32 codes, including surrogates and values above the Unicode scalar
range, remain accepted as data. Existing JavaScript scalar refusals and native/C
raw IO bytes remain unchanged.

## Results and limits

- Standard gate: **1319 passed / 0 failed / 49 ignored**, 195
  groups, 994.551 seconds, on the clean tested source.
- Focused: 13 unit tests and 13 integration tests passed; targeted strict Clippy
  passed. Two initial exploratory test failures are retained: one first exposed
  the cross-line lexer boundary, and the labeled rerun identified its case.
  The case remains unresolved; no lexer or guard change was made to pass it.
- Main matrix: **605 observations / 104 deterministic pairs / 397 singletons**,
  zero unexpected outcomes or timeouts. Forty rejection inputs comprise 39
  authored controls and one byte-identical pinned unknown-escape original.
- Authored diagnostic subset: **5 exact / 34 differing before**, now **28 exact /
  11 differing**: 23 improvements, no previously exact diagnostic regression,
  no changed acceptance status. Both sides refuse all 39 at load time.
- The pinned unknown-escape original remains exact. Rejection checks cover
  checking, strict evaluation, native-run entry and C/JS emission, plus port
  native emission; refused commands write no artifact. These are refusal tests,
  not execution of malformed programs.
- Six accepted controls produce **24 exact corresponding C/JS executions**:
  16 successes and eight retained JS scalar refusals. Five native pure outputs
  and one actual native IO output are unchanged; 12 port artifacts are
  byte-identical to the baseline. Strict checking still excludes the IO control.
- Full graph, literal/input-depth, false-proof, selected-pattern, cancellation
  before effects, sharing and work-accounting regressions remain in the affected
  tests/full gate. No new mid-normalization cancellation guarantee is claimed.

The 11 authored residuals are two cross-line lexer diagnostics, an astral observed
UTF-16 unit, an earlier function-parameter grammar error, three later-code Char
`U` errors, two errors after noncanonical lowercase prefixes and two analogous
`q` controls. Their exact differing streams remain in the receipt. In particular,
`'a\U{}'` must first report a closing quote upstream, and `"\u{+41}\U{}"` fails
at the earlier lowercase escape upstream. No new exception is adopted.

Public `ParseError` fields, loader rejection and type layouts are unchanged.
The optional diagnostic sidecar becomes available for qualified `U` cases and is
withheld for the two previously misleading `q` controls. Consequently public
command `invoke` calls may change **Err → Ok(exit_code=1)** for new formatted
failures, or **Ok(exit_code=1) → Err** for those narrowed controls. Process status
remains 1. Repository dispatch propagates both, and `src/lib.rs` reads
`CliOutput::exit_code()` before `src/main.rs` exits; external consumers have not
been audited. Internal diagnostics remain available through tracing.

No whole census was rerun. Historical 140 non-IO differences (109 acceptance,
31 rejection) plus six upstream stack failures remain measured at c5edea8,
not inferred current totals. Coverage linkage stays 516/1302, including failed
receipts; 786 unlinked entries remain 256 checker rejection, 517
interpreter/compiled candidates, 12 interpreter-only and one no-main module.

## Reproduction and preservation

Use the existing activated environment and repository working directory:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off PYTHONDONTWRITEBYTECODE=1
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=4
cargo test --locked --features extended_observability --lib literal_ -- --nocapture
cargo test --locked --features extended_observability --test cpu_malformed_literal_diagnostics --test cpu_nonscalar_literal --test cpu_c_escape_printer --test cpu_uppercase_unicode -- --nocapture
cargo clippy --locked --features extended_observability --lib --test cpu_malformed_literal_diagnostics --test cpu_nonscalar_literal --test cpu_c_escape_printer --test cpu_uppercase_unicode -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

C uses the pinned `-std=c11 -O3 -lpthread -lm` profile; port C also uses
`-Wall -Wextra -Werror -pedantic`. Programs run with `--threads 1 --gpu off`;
Node executes JS. Matrix commands have 30-second timeouts. Host Cargo debug
settings are separate from Bend optimization and safety budgets.

Exact commands and lossless outputs are retained with source, stream and artifact
hashes. Published paths use explicit root substitutions. Raw matrix SHA-256:
`8db8bb72928e8f10e39e5ac919cfd735bc08bc1a2b1840d2710af2e2ac140bf6`; full-gate log SHA-256: `7191ad9824cbb001d0a47010998d10d986415951de68a79e4f2abaeed65c4654`.
The final gate executable is byte-identical to the compressed tested snapshot
(the verified delta has zero changed bytes). A further 58 observations
renew diagnostics/native outputs/generated source. An earlier identity attempt
stopped after 43 observations because the relocated snapshot has no adjacent Node
provider, while the Cargo-directory binary packages its existing provider.
That raw failure is retained. Renewal verifies the destination-dependent JSON
and provider bytes separately; all 12 generated programs remain byte-identical.
The packaging difference is recorded, not normalized away as an exact CLI-output match.

Compact source/receipts are in Git. A verified private archive retains 102
records (769245 logical bytes, 82926 archive bytes), SHA-256
`9f5a269f13aaabb19139c0ccbcc955a8d687f1690d541a35c455390f3dc8db2b`. Generated programs and binary snapshots remain local, outside Git.
All prior cleanup restoration records and backups were verified unchanged.
The older lost focused log remains unrecovered. No Library retry occurred.

Gate storage minima: 12.172 GiB workspace / 2.039 GiB temporary; unchanged
floors are 11 GiB before builds, 10 GiB active workspace and 1 GiB temporary.
Pins, dependencies, toolchains, semantic guards and budgets are unchanged.
No desktop, GPU, installation, main merge or release occurred.

Next substantive gap: existing lowercase `\u{+41}` and `\u{000000041}` controls
still accept/normalize to 65 in the port but reject upstream. Eight fresh
checking/pure-normalization observations confirm this; executable qualification
and correction were not attempted in this batch. Existing Base coordinates,
`nat_first` ordering, unadopted compact lowering, external invoke audit,
supplementary all-target lints, F32/foreign/source-hole/resource and oracle
failures remain open. No new semantic decision was made.
