# Reserved-name loader diagnostic parity

Tested source: `de64abf0a7f4c844d2de1b0898af852f1af93d8d`, based on published checkpoint
`d1773a46cbabb7ed740d2c85a036647cb1dcb740`. The upstream target remains
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; documentation remains
`50e89d6897ce2a65167ef65b17c5fa83fe849158`.

## Change and pinned contract

Nine unchanged originals previously failed during load with a generic port
`expected a name` host error. They now match the pinned keyword message,
raw following character and source excerpt: `parse/keyword_reserved`,
`reserved_assert_name`, `reserved_ctor_field_000`, `reserved_ctor_field_001`,
`reserved_def_name_001`, `reserved_def_name_002`, `reserved_pi_binder`,
`reserved_type_name` and `reserved_type_param`.

Pinned [parse_name](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1580)
reads the lexeme before refusing a reserved keyword. Its diagnostic therefore
observes the raw following character, including whitespace, comments, newline,
CRLF or EOF. For example `law law : Type` now reports
`a name (got the keyword 'law')`, observed `' '`, and the source line.
The port reuses existing source-diagnostic metadata for that location without
advancing its actual token cursor or changing its public error.

Private `NameContext` distinguishes ordinary and constructor name slots from
term parsing. The latter has a [separate pinned contract](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1726)
and is deliberately unchanged. The sixteen-keyword set is unchanged; `as`,
keyword prefixes and qualified names retain their existing resolution.

No acceptance, evaluation, termination, sharing, fresh-ID, work-accounting,
stack-size, cancellation, guard or budget changes. Public `ParseError` fields,
message, Display and loader signatures remain unchanged. Successful tracked and
untracked loads produce equal complete books. Newly formatted CLI failures use
the existing `Ok(CliOutput)` with `exit_code = 1` boundary where the old host
error used `Err`; external `invoke` consumers must inspect status and remain
unaudited. Source-aware load failures now carry the new optional metadata, so their derived Debug output may change. No new public error variant or field is introduced.

## Evidence and validation

The [fixtures](../tests/fixtures/cpu-reserved-name-diagnostics/README.md) preserve
all eleven upstream files byte-for-byte. Nineteen exact negatives comprise the
nine original gaps, nine authored raw-character/import/template controls and the
already matching `reserved_def_name_000` duplicate-Base-U32 control. U32 is a
library name, not a reserved keyword. Eighteen bounded diagnostic gains do not
mean eighteen independent semantic fixes. Compact [pre-fix streams](../tests/fixtures/cpu-reserved-name-diagnostics/before.json)
and twenty-two public loader errors are retained.

Four pure positives cover free `as`, prefixed and qualified names, fields and
imported template replay. Results are respectively `7`, `9`, `12`, `7`.
Complete normalized shape/text, pure run_main normalization, and generated C/JS
execution are tested separately. Eight port artifacts are byte-identical to
baseline emission; baseline emission alone was not counted as execution.

The [matrix](cpu-reserved-name-diagnostics-results.json) records **522 commands; 249 deterministic repeated pairs; 32 generated C/JS executions; 16 pure normalizations; 342 exact negative stream comparisons; 76 negative phase probes; 8 baseline-byte-identical port artifacts**.
All repeated status/stdout/stderr observations agree; no timeout or unexpected
result. C uses the actual pinned `-std=c11 -O3` profile with `-lpthread -lm`;
port C additionally passes `-Wall -Wextra -Werror -pedantic`. Invalid sources
produce no executable artifact and are not executed. No native IO claim is made
for these pure programs. Phase probes remain distinct from exact diagnostics.

The [separate retained controls](cpu-reserved-name-diagnostics-retained-controls.json)
record thirty commands: lambda-binder, expression-keyword and earlier-depth
failures remain byte-identical to baseline port behavior, deterministic, and
non-exact versus upstream. The earlier 64-level parser refusal still wins over
a later reserved name. These are neither matches nor new policy exceptions.

Full repository gate: **1270 passed / 0 failed / 49 ignored**, completed in 1040.684 seconds.
Focused controls: **69 passed, zero failed** (three state/provenance unit tests
and sixty-six integration/regression tests). Targeted strict Clippy passed.
Full gate renews existing host-stack, shared-reference, aggregate work-accounting,
closed-partial proof, writer/flush, cancellation and guard controls. This does
not add mid-normalization cancellation. Independent read-only source and
matrix/census reviews are included in the [validation receipt](cpu-reserved-name-diagnostics-validation.json).

Exact primary commands, with pre-existing tools activated:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib reserved_name -- --nocapture
cargo test --locked --features extended_observability --test cpu_reserved_name_diagnostics --test cpu_function_proof_diagnostics --test cpu_duplicate_declaration --test cpu_unknown_template --test kernel_core --test kernel_normalization --test kernel_open_closures
cargo clippy --locked --features extended_observability --lib --test cpu_reserved_name_diagnostics -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host build-profile overrides control Rust debug information/cache size. They do
not change Bend optimization, safety limits or the generated C O3 profile.
Rust 1.96.0, Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4 remain installed and
unchanged. Every differential command and its raw-stream hashes appears in the
matrix receipt; machine roots are replaced with placeholders. The existing
`scripts/checking-census/run.py` command is recorded in the validation receipt.

## Reproducibility corrections and storage

An early authored positive omitted explicit U32 operator typing; upstream
correctly refused the resulting Nat/U32 mismatch. Both lanes were corrected
before qualification. An initial Python receipt comparison translated raw CR;
subsequent comparisons preserve bytes. Both failures remain local evidence.

Git initially normalized the deliberate CRLF fixture. A fixture-scoped
`.gitattributes` and explicit index renormalization preserve raw CRLF input and
stderr on checkout. All committed fixture blobs were then compared with their
working bytes. An initial gate at `f631f43` was stopped after 20.006 seconds because
it had started despite the byte-equality assertion failing; it is interrupted,
not passed. The renewed clean gate and binary identity belong to `de64abf0a7f4c844d2de1b0898af852f1af93d8d`.
No history or failed evidence was discarded.

Build preflight requires 11 GiB workspace; active-run stops are 10 GiB workspace and
1 GiB temporary storage. Measured full-gate minima: **10.998 GiB workspace / 2.846 GiB temporary storage**.
Installer cache relocation retained identical bytes and paths via symlinks but
freed no workspace blocks because originals were in the base layer. A lossless
`git repack -a -d --keep-unreachable` recovered 44,003,328 bytes by packing duplicate
storage, with all 7,732 object IDs and all refs verified identical. No object
pruning, source/evidence deletion, tool reinstall or limit reduction occurred.

## Current census and remaining work

The renewed 1,302-fixture checking census is **366 both accept, 391 exact rejections, 494 acceptance disagreements, 45 different rejections, 6 oracle/host failures**.
Outside the exactly renewed 385-member historical IO cohort there are
**154 residuals: 109 acceptance disagreements, 45 different rejections**. The [census receipt](cpu-reserved-name-diagnostics-census.json)
records actual before/after streams and hashes, classifications, phases and any
upstream variation separately. Both-accept is checking-status agreement, not
runtime parity. Only the nine named port stderr streams changed; no status, phase or upstream stream changed. The residual fell from 163 to 154: 109 acceptance disagreements, 29 inherited other-rejection differences and 16 diagnostic candidates. Those labels do not prove diagnostic-only semantics. All six historical oracle host failures remain. The 386th missing-IO symptom does not enlarge the 385-member historical cohort.

Receipt linkage is **499 linked / 803 unlinked; unlinked categories: 269 checker rejection, 520 interpretation and compiled candidates, 1 checking without main, 13 interpretation only**. This includes partial and blocked observations;
unlinked entries are categorized by applicability, not uniformly missing runtime
tests. Full CPU parity remains incomplete under the existing acceptance criterion.

The next small demonstrated diagnostic candidate is `parse/reserved_lambda_binder`:
both loaders refuse it, but the port emits its generic one-name-binder error
instead of the pinned `a lambda binder (one name: k => body)` diagnostic with the raw following space and source window. Its distinct parser
path needs its own correction and controls; this checkpoint does not qualify it.
The expression-keyword difference remains separate.

External invoke auditing, Base callback coordinates, pending nat_first ordering,
unadopted compact Nat/string lowering, strict-proof/foreign/resource boundaries,
F32.read, source-hole and upstream host/printer failures remain outstanding.
Supplementary all-target Clippy was not rerun; its historical failures remain
open. No dependency, toolchain or upstream update; no desktop/GPU, installs,
main merge or release.

Compact expected streams, current results, failed controls and census deltas are
in Git. Raw logs, generated artifacts, binary archives, phase sidecars and
initial runner/authoring attempts remain local outside Git preservation. The
reviewed feature checkpoint adds documentation/receipts after tested source.
