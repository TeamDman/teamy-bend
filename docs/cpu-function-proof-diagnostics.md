# Lambda rejection diagnostics

Tested source: `286741ec7c8bce954ef25de6e6ed5420acff44de`. Target remains
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Full CPU parity remains incomplete under the [acceptance criterion](cpu-parity-acceptance.md).

## Cause and correction

The unchanged `proof/no_funext_000.bend` rejects a lambda supplied as a proof of an
equation between functions. The port already made the correct decision, but emitted
an internal type mismatch with host context instead of the pinned expected equation,
`non-inferrable term 'x => add_zero(x)'`, and the lambda's Bend source window.

Pinned [check-lam](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3423)
weak-head reduces the goal and rejects a non-function type using the original
goal, lambda occurrence and outer context. The port now attaches its existing
structured diagnostic on precisely that existing refusal branch. Original
`mismatch(ty, value)` remains the error. Rendering uses the remaining-budget
clone and falls back to that original error if normalization or presentation fails.
The lambda body is not newly typechecked; any diagnostic normalization occurs
only after rejection on the bounded display clone.

No function-extensionality rule is introduced. The unchanged, misleadingly named
`check/no_eta_conversion.bend` is a positive control: the pin already supports its
definitional eta equality by reflexivity. Accepting that proof does not permit a
lambda to inhabit an equation type. Ordinary valid lambdas, aliased function types,
closed applications and reflexivity proofs retain their checking and execution.

## Host-stack regression caught and corrected

The initial clean source `e0aceb2636fce3bf01bddfbad35b8254f20041e0` failed its
standard gate after 925.768 seconds. Completed groups reported 932 passes and
48 ignores; `kernel_core` then aborted with a host stack overflow in the existing
90-level nested-application regression. That target produced no final result and
later targets did not run. The saved test binary reproduces SIGABRT directly.

The new failure-only diagnostic locals enlarged the unoptimized recursive
`check_inner` frame from **11,416 to 11,848 bytes**, including successful paths
that never reject a lambda. Independent source/disassembly review confirmed this
432-byte increase. A private `#[inline(never)]` helper now contains the same
mismatch construction and diagnostic fallback. This restores the frame to
**11,416 bytes**, retaining the original rejection point, sharing and live budget
accounting. All 19 existing kernel-core tests then pass, including the unchanged
nested positive, type-error and quantity-error controls. No host stack or Bend
limit was raised; no failed test was weakened. The initial failure, rejected
candidate and earlier passing controls remain historical evidence.

## Public boundary and invariants

Public kernel error message, Display and equality remain unchanged. This adds no
signature, parser-error or loader-error change. Newly formatted sourced CLI
failures use the existing `Ok(CliOutput)` with `exit_code() == 1` boundary instead
of their prior unstructured host `Err`. Process failure status remains 1. The
binary reads that status; direct callers must also inspect it. Derived Debug
includes attached metadata and can change. External invoke consumers remain
unaudited; this is an explicit compatibility caveat, not a downstream audit claim.

Proof rejection, source-hole refusal, inference order, semantic types, evaluation,
termination, sharing and safety limits remain unchanged. Expected/rejected term
identities and original source provenance are preserved. Diagnostic clone work
cannot consume or replenish the live engine's allowance. There is no new
mid-normalization cancellation claim.

## Qualification

Full standard gate: **1265 passed / 0 failed / 49 ignored**, exit 0 in 1037.374 seconds. Format,
strict production Clippy, all-feature build and real Node TCP/UDP provider checks
pass. [Validation receipt and exact commands](cpu-function-proof-diagnostics-validation.json).

Final focused runs: **19 kernel-core tests and three diagnostic state tests passed**;
targeted strict Clippy passes. These overlap the renewed full gate. The initial
candidate's 48 integration and 65 diagnostic controls passed, but did not catch
the host-stack regression subsequently exposed by the full gate. An earlier synthetic
state test failed because the empty engine's initial fresh identifier 1 collided
with a manually inserted alias 1. An intermediate seed-2 run passed but still reused
an authored binder ID. Independent review required the final seeds 3, 3 and 130,
strictly above authored maxima 2, 2 and 129. The final three-test state run and
renewed full gate cover those corrected fixtures. Initial failure and intermediate-pass receipts remain separate;
no production change or budget increase was made to accommodate the test harness.

The [bounded matrix](cpu-function-proof-diagnostics-results.json) records:

**320 commands**, **151 deterministic repeated pairs**, **24 generated C/JS executions**, **12 pure normalizations**, **198 exact negative comparisons**, **44 separate phase probes** and **6 baseline artifact matches**.

No unexpected matrix outcome or timeout occurred. All three successful controls
are pure programs: run_main contributes normalization evidence; generated C-O3
and generated JS are actual executions. There is no native IO claim for this batch.
Six positive port artifacts remain byte-identical to the baseline. All eleven
negative cases reject during checking with no executable artifact; invalid-source
execution remains unrun. Exact diagnostic agreement is distinct from phase probes.

The [fixtures](../tests/fixtures/cpu-function-proof-diagnostics/README.md) retain
three unchanged Apache-2.0 originals. Nine bounded diagnostic gains include the
original false-funext proof plus scalar/equality/aliased goals, outer contexts,
nested matches, imported/template locations and shadowing. Single-use lambda
consumption and the false closed-partial proof retain their exact prior refusals.
Eleven measured pre-fix public error goldens remain identical after the fix.

Successful controls compare complete normalized value graphs and text before
executing C/JS. Unit tests retain expected/value identities, aliases, owner counts,
fresh identifiers and exact live work charges. Earlier goal-reduction failures
retain precedence; exhausted expected-type or lambda-body rendering falls back
without changing rejection. The original is also included in cancellation,
untracked-source and writer/flush failure controls. Prior stack, sharing,
work-accounting and closed-partial proof regressions remain in the renewed gates.

## Reproduction and storage

```sh
source <tools>/activate.sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib function_goal -- --nocapture
cargo test --locked --features extended_observability --test cpu_function_proof_diagnostics --test cpu_contextual_proof_diagnostics --test cpu_meet_diagnostic_precedence --test cpu_comptime_rebind --test kernel_normalization --test kernel_open_closures
cargo test --locked --features extended_observability --lib diagnostic
cargo clippy --locked --features extended_observability --lib --test cpu_function_proof_diagnostics -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The state command has both historical attempts and a final corrected run,
recorded separately. The broad integration/diagnostic commands above describe
initial focused evidence; the renewed full gate covers those tests at the corrected
source. The final focused kernel-core command is:

```sh
cargo test --locked --features extended_observability --test kernel_core
```

Per-process comparisons are bounded at 30 seconds; gate timeout remains 1,200 seconds. Host debug/incremental
settings above are temporary Cargo overrides, distinct from the pinned upstream
C O3 profile. Port C also uses strict warning flags; repository tests keep their
own unchanged compiler settings.

Frozen: Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, Clang 19.1.7,
PowerShell 7.5.4, docs `50e89d6897ce2a65167ef65b17c5fa83fe849158` and Cargo.lock
SHA-256 `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Build preflight stays 11 GiB workspace; stop floors stay 10 GiB workspace and
1 GiB temporary storage. Gate minima: 10.990 GiB workspace / 2.980 GiB temporary storage. Existing cache links were
retained; no new cache reclamation or unique evidence removal was necessary.

Code whitespace checks pass. Spaces in unchanged upstream fixtures and exact
stderr source windows are retained intentionally and reported by the unfiltered
diff check. A baseline-helper invocation initially lacked the activation environment;
it launched no compiler and was rerun after activation. This is distinct from the
retained synthetic test failure. One snapshot taken during Cargo's feature-binary
replacement was rejected by its identity assertion; the final snapshot was taken
after test execution started and verified against the matching feature artifact.
No mismatched snapshot was used for comparisons. No frozen dependency, toolchain or security setting
was changed.

## Remaining coverage

The full [1,302-fixture census](cpu-function-proof-diagnostics-census.json)
measures **366 both accept / 382 exact rejections / 494 acceptance disagreements / 54 different rejections / 6 oracle-host failures**. Changed port observations: proof/no_funext_000.bend.
The reconciliation preserves exact status/phase/stream differences and any oracle
variation, without inferred acceptance gains. The residual outside the exactly
renewed historical 385-member IO cohort is **163**; oracle-host failures
remain separate. New IO symptoms do not enlarge that historical cohort.

The residual is **109 acceptance disagreements + 54 different rejections**.
Only `proof/no_funext_000.bend` changed on the port side, from different to exact
rejection. The port's checking status and phase are unchanged for all 1,302 cases.
All upstream statuses, phases and streams in this final run match the preceding
published baseline. This closes one measured diagnostic gap, with no acceptance gain.

The intermediate census at rejected candidate `e0aceb2636fce3bf01bddfbad35b8254f20041e0`
had 164 residuals and five oracle failures: upstream `reg/borrow_fork_hold.bend`
accepted in that run. It again reported its historical host-stack failure in the
final run, while the port's Nat-literal resource refusal remained byte-identical.
This unstable oracle outcome is preserved separately; it is not a port improvement.
All six historical host failures remain unqualified. The 386th IO symptom does not
enlarge the exactly renewed 385-member historical cohort.

Inherited triage labels within the residual are 109 acceptance mismatches,
29 other rejection differences and 25 diagnostic candidates. Those labels are
not waivers or proof that all candidates are diagnostic-only.

Receipt linkage, not whole-parity acceptance: **489 linked / 813 unlinked**. Unlinked applicability: 520 interpreter/compiled candidates, 279 checker rejections, 13 interpreter-only fixtures and 1 checking-only module without main.

The next independently identified bounded candidate is the nine related
reserved-name loader diagnostic differences: `parse/keyword_reserved`,
`parse/reserved_assert_name`, `parse/reserved_ctor_field_000` and `_001`,
`parse/reserved_def_name_001` and `_002`, `parse/reserved_pi_binder`,
`parse/reserved_type_name`, and `parse/reserved_type_param`. Current census
observations show both sides rejecting during load, with upstream reserved-keyword
source diagnostics versus a generic port expected-name host error. This is a
candidate for reproduction and correction, not a completed qualification.
`parse/reserved_lambda_binder` has a distinct failure path and remains separate.

Base callback coordinates, nat_first ordering, unadopted compact Nat/string
lowering and external invoke auditing remain outstanding. Existing strict-proof,
resource/foreign, printer/host, F32.read and hole disagreements remain unwaived.
Supplementary all-target Clippy was not rerun; its historical failure remains
outstanding. No desktop/GPU work, newer upstream adoption, installs, merge or release.

Compact pre-fix and current receipts are committed. Raw logs, generated artifacts,
phase sidecars, binary snapshots and initial-harness evidence remain local outside
Git preservation. The final checkpoint adds documentation/receipts only after the
clean tested source. Reviewed publication uses an isolated feature branch.
