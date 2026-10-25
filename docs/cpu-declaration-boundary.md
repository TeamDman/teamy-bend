<!-- SPDX-License-Identifier: MPL-2.0 -->
# Declaration-boundary diagnostics at the frozen pin

Clean tested source: `27204f16e87e57f4285610dc4cb0dab23b622988`. This checkpoint fixes nine demonstrated
source-load diagnostic mismatches. It changes no acceptance, evaluation, argument
ordering, safety limit or public parser error. Full CPU parity is still incomplete.
The `nat_first` C ordering choice remains pending and unwaived; compact-pattern
research remains unadopted.

## Selection from the recorded census

The last full census remains historical at `d224e79d33b4a0ab26f701b57f448056b63b790a`:
300 differences outside the 385 historical strict-IO exclusions, plus six upstream
stack failures. The earlier baseline 302 classified 151 diagnostic candidates,
109 acceptance disagreements and 42 other rejection differences. Two earlier
repairs reduced that measured subset to 149 / 109 / 42 at the historical source.
These are observational categories, not 300 independent executable defects.
Numeric/foreign proof-name exclusions and explicit guards are not silently expanded
into scope or waived. No full-census renewal occurred in this checkpoint.

We selected five constructor-result annotation negatives and four adjacent global
declaration negatives: four historical diagnostic candidates and five parser-boundary
differences. Current checking outcomes were reproduced before changing code. All nine
load-rejected in both implementations; their diagnostics differed. Their unchanged
originals, source hashes, expected stderr and compact pre-fix failures are in
[the fixture directory](../tests/fixtures/cpu-declaration-boundary/README.md).
Authored positive sources required Base/reflexivity/operator-annotation corrections
before qualification; their initial failures remain separate private evidence.

## Cause and correction

Pinned [constructor parsing](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2572)
stops at a non-name-head token. The outer parser then reports
[`'def', 'type' or 'law'`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2630)
at that untouched token. For `Mk{} : U32`, the port instead reported `a name` at
`:`. Existing global declaration refusals reported a host-context error.

The constructor-row call now passes the outer category to existing `name_expected`;
the declaration rejection attaches the same optional diagnostic metadata. The
parser cursor, invalid-name predicate, public `ParseError`, partially built terms
and successful paths are untouched. No new constructor-result syntax or GADT
support is introduced. Keyword cursor behavior and whole-file lexer precedence
remain outside this fix. No kernel, compiler or runtime implementation changed.

Independent reviewers inspected the pinned grammar, source change, all nine
original hashes, fourteen negative goldens, default-feature snapshot and completed
bounded matrices. They found no blocking issue and ran no builds themselves.
Root executed the checks below. The small unsafe-decorator control is parser-only;
it is not a strict-proof or execution claim.

## Validation

| Check | Result |
| --- | --- |
| Standard gate at the clean source above | **1,178 passed / 0 failed / 49 ignored** |
| Focused unit and parser/checker/provenance targets | 8 + 66 passed |
| Strict Clippy for the changed integration target | Passed |
| Negative matrix: 9 originals + 5 authored controls | **252 exact diagnostics; 84 load-phase probes** |
| Three valid controls, outputs 7 / 5 / 5 | **30 executable runs** across native, port C/JS and upstream C/JS |
| Pinned interpreter / strict port evaluation / strict checking | 6 / 6 / 6 successful observations |
| Generated C/JS / C builds | 12 generations / 6 successful builds |
| Positive phase probes | 18 accepted |
| Before/current generated code | All six C/JS files byte-identical for these controls |

Negative comparisons run twice across upstream CLI/C/JS generation and the six
port entries: check, eval, run, executable C/JS/native compilation. Every outcome
has exit 1, empty stdout, exact upstream stderr and no artifact. Negative input
never executes. Successful generations/builds are distinct from execution.
Positive values retain complete constructor-graph regression checks for strict
normalization and data evaluation, separately from printed output. Every repeated
available outcome was deterministic; no matrix timeout occurred. Upstream C uses
the pinned CLI's C11/O3 profile; port C adds strict warnings. No O0 claim is made.

The full gate renews existing sharing, packed-word, work/depth, cancellation,
source-hole, default-stack and false-proof regressions. It does not add
mid-normalization cancellation. Supplementary all-target Clippy and all older
oracle/host/resource/F32.read/foreign/hole differences remain as previously
recorded; none is cleared by a passing standard gate. Hardware/profiler ignores
remain unrun. [Commands and exact bounded outcomes](cpu-declaration-boundary-results.json)
and [validation provenance](cpu-declaration-boundary-validation.json).

Formatted CLI refusals use the existing `Ok(CliOutput)` with status 1 boundary.
Repository production dispatch reads `exit_code()` before emitting the result;
the actual six CLI failure statuses are tested. External consumers were not
audited and must inspect that status. No public API signature or plain loader
error changed.

## Reproduction and remaining decisions

Use the existing frozen environment. Host overrides are
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_BUILD_JOBS=4`,
`CARGO_INCREMENTAL=0`, `CARGO_NET_OFFLINE=true`, `BEND_GPU=off`; `RUST_MIN_STACK`
was unset. These do not alter Bend execution profiles or safety budgets.

```sh
cargo test --locked --offline --features extended_observability --lib syntax::diagnostics::tests
cargo test --locked --offline --features extended_observability \
  --test cpu_pattern_locations --test cpu_template_occurrences --test kernel_core --test parser
cargo clippy --locked --offline --all-features --test cpu_pattern_locations -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

For each original/control in `cases.json`, compare the pinned CLI and `-o` C/JS
forms with each port entry listed above, twice. Goldens and all concrete commands
are in the results file. [Phase-probe instructions](../scripts/checking-census/README.md)
apply to strict and executable loading separately. Matrix child timeout: 30 seconds;
integration child timeout: 20 seconds; full gate: 1,200 seconds. Workspace preflight
requires 11 GiB; stop floors remain 10 GiB workspace and 1 GiB temporary storage.
No source, unique historical evidence or caches were removed this turn.

Nine exact checking receipts bring linked inventory coverage to **438 / 1,302**;
**864 remain unlinked**: 327 checker negatives, 523 compiled candidates,
13 interpreter profiles and one checking-only module. Links do not establish
universal CPU parity, and unlinked entries are not all runtime tests. No subtraction
from or relabelling of the historical 300-difference census is justified by this
bounded renewal.

No new semantic decision is needed for this fix. The exact pending `nat_first`
decision remains in [the prior report](cpu-failure-order-renewal.md). No extra
ordering exception was introduced or implicitly approved. The next independent
measured candidate is numeric lambda-arm diagnostic context: unchanged
`check/erased_match_numeric` and `flatten/statement_beta` still load-reject in
both implementations, but disagree on expected syntax/location. Eight repeated
CLI observations plus eight independent phase probes reproduce this gap. They
remain unqualified, with no acceptance/evaluation correction claimed here.

Pins, dependencies, toolchains, guards and budgets remain frozen. Compact durable
failure/qualification receipts are in Git; raw logs, full receipts and compressed
binaries remain local and are not preserved by publication. Feature-branch
publication is not a main merge or release. No installs, desktop or GPU work.
