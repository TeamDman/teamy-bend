# Quantitative-meet diagnostic precedence

Tested source: `9d13eba51d51b2de7f5ab415770c355f86fc0e58`. Upstream remains
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Full CPU parity remains incomplete under the [acceptance criterion](cpu-parity-acceptance.md).

## Correction and contract

The unchanged `check/meet_stuck_pending_ref.bend` correctly rejected the false
proof while its referenced law was pending. Its expected reflexivity endpoint was printed as
`(q <&> &1)` instead of pinned upstream's `q <&> &1`.

Pinned [term_show Min](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1334)
wraps the expression only at precedence greater than 1. Its
[diagnostic entry points](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1440)
start at precedence -1. The port's ordinary term formatter always wrapped meets.

A private diagnostic presentation mode now follows that pinned rule through
existing printer helpers. It keeps parentheses on both nested-meet operands and
on function heads; it omits unnecessary outer parentheses and parentheses inside
Kind/constructor arguments. It preserves lexical scope through binders. This is
formatting of the existing normalized diagnostic terms, not additional reduction,
string stripping, graph rewriting or a change to their semantic types.

Public Term Display and normal-form output retain their prior text. Public
KernelError message, Display and equality retain their prior values. No signature
or Err/Ok boundary changes: these already-formatted CLI failures still return
Ok(CliOutput) with exit_code 1, and callers must inspect it. Derived Debug includes
diagnostic metadata and can change. External invoke consumers remain unaudited.
Acceptance, pending-law handling, quantity charging, evaluation, sharing, work/depth
limits and cancellation remain unchanged. No mid-normalization cancellation claim.

## Qualification

Standard gate: **1260 passed / 0 failed / 49 ignored**, exit 0 in 1021.035 seconds. Format,
strict production Clippy, all-feature build and real Node TCP/UDP provider checks
pass. [Exact commands, environment and hashes](cpu-meet-diagnostic-precedence-validation.json).

Focused runs: 2 printer/state tests; 46 integration tests; 2 renewed integration
tests after adding context-kind and observed-side cases; 62 diagnostic controls;
targeted strict Clippy passes. The 112 successful test invocations overlap each
other and the full gate; they are not 112 additional unique tests.

The [bounded matrix](cpu-meet-diagnostic-precedence-results.json) records:

**272 commands**, **130 deterministic repeated pairs**, **16 generated C/JS executions**, **8 pure normalizations**, **180 exact negative comparisons**, **40 separate phase probes** and **4 baseline artifact matches**.

No unexpected outcome or timeout occurred. Both positive controls are pure
proofs: run_main is normalization, while generated C at O3 and generated JS are
actual executable observations. No native IO execution is claimed for this batch.
Four emitted port artifacts remain byte-identical to the baseline. Invalid-source
execution remains unrun; all ten negatives fail checking and emit no artifact.
Exact diagnostics are separate from phase-only probes.

The [fixtures](../tests/fixtures/cpu-meet-diagnostic-precedence/README.md) retain two
unchanged Apache-2.0 originals. Eight bounded diagnostic differences close; meet
operand charging and the false closed-partial proof remain exact unchanged
rejections. Ten pre-fix public error goldens remain byte-identical after the fix.
Seventeen syntax printer cases preserve complete Debug structure, root identity,
ownership and legacy Display while matching pinned diagnostic precedence. They
cover nested operands, application head/argument, Kind, constructors, datatypes,
equality, annotations, arrows, lambdas, matches, cons, Nat tails, rewrite and let.
These printer observations do not assert the arbitrary terms are well typed.
Their pinned oracle is `term_show(term_lower(term_higher(parse_term(p))))`,
using a fresh empty book/parser state for each saved `source` string. Higher/lower
conversion refreshes lexical levels without reduction. Legacy strings were
measured through the baseline Rust `parse_term` and public Display. The saved
JSON keeps both results independently; it does not rewrite either expectation.

Dedicated shadowing and input-depth controls preserve the 128-level boundary;
previous diagnostic work/depth/fallback, source provenance, cancellation/writer,
sharing, stack and normalization controls are renewed by focused and full gates.

## Commands and storage

```sh
source <tools>/activate.sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib diagnostic_meet -- --nocapture
cargo test --locked --features extended_observability --test cpu_meet_diagnostic_precedence --test cpu_contextual_proof_diagnostics --test cpu_comptime_rebind --test kernel_normalization --test kernel_open_closures
cargo test --locked --features extended_observability --test cpu_meet_diagnostic_precedence
cargo test --locked --features extended_observability --lib diagnostic
cargo clippy --locked --features extended_observability --lib --test cpu_meet_diagnostic_precedence -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Temporary host Cargo debug/incremental overrides are distinct from Bend compiler
settings. Differential C uses the upstream CLI's O3 profile; port C also uses
strict warnings. Repository runtime tests retain their own compiler settings.
Process comparisons use 30-second timeouts; the gate uses 1,200 seconds.
Build preflight remains 11 GiB workspace; stop floors remain 10 GiB workspace and
1 GiB temporary storage. Gate minima: 11.045 GiB workspace / 3.085 GiB temporary storage. Existing cache links remain;
no new cache reclamation or unique-evidence removal was needed.

Frozen: Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, Clang 19.1.7,
PowerShell 7.5.4, docs `50e89d6897ce2a65167ef65b17c5fa83fe849158` and Cargo.lock
SHA-256 `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

Code whitespace checks pass. The unfiltered diff check reports spaces in an
unchanged upstream fixture and exact diagnostic source windows; those bytes are
preserved intentionally. An authored malformed import was corrected before final
qualification; its exploratory rejection remains local evidence. Two task-local
script initializations refused existing temporary paths before writes; corrected
new destinations preserve historical binaries and artifacts. Neither was a
product-test failure.

## Remaining coverage

The renewed [1,302-fixture census](cpu-meet-diagnostic-precedence-census.json)
measures **366 both accept / 381 exact rejections / 494 acceptance disagreements / 55 different rejections / 6 oracle-host failures**. Changed port observations: check/meet_stuck_pending_ref.bend.
Only that port stderr stream changes. All acceptance statuses, failure phases,
upstream streams and the other 1,301 port streams are unchanged; there is no
oracle-variation adjustment. The residual outside the exactly renewed historical
385-member IO cohort is **164 = 109 acceptance disagreements + 55 rejection
differences**, down from 165 through this one diagnostic closure. Historical
triage labels among those rejections remain 26 diagnostic candidates and 29 other
rejection differences. Oracle-host failures remain separate. Existing IO symptoms
do not enlarge the historical exclusion cohort.

Receipt linkage, not completed parity: **486 linked / 816 unlinked**. Unlinked applicability: 521 interpreter/compiled candidates, 281 checker rejections, 13 interpreter-only fixtures and 1 checking-only module without main.

Base callback coordinates, nat_first ordering, unadopted compact Nat/string
lowering and external invoke auditing remain outstanding. Other strict-proof,
resource/foreign, printer/host, F32.read and hole disagreements remain unwaived.
Supplementary all-target Clippy, desktop/GPU and newer upstream adoption were not
rerun; historical supplementary failures remain unwaived. No new exception, dependency, install, merge or release.

Next independent measured candidate: `proof/no_funext_000.bend` rejects in both implementations, but the port still emits a host-context type mismatch instead of the pinned non-inferrable-lambda diagnostic and Bend source window. No acceptance or function-extensionality change is proposed, and this candidate is not fixed here.

Compact pre-fix and current receipts are committed. Raw bulk logs, generated
artifacts, binary snapshots and cache archives remain local, outside Git
preservation. Reviewed publication uses an isolated feature branch. The final checkpoint adds
receipts/documentation and corrects one stale test comment after the gate; no
executable Rust tokens, fixtures or assertions change after the tested source.
