# Contextual reflexivity rejection diagnostics

Tested source: `366c7487fa795685227c72fcdb58a1af4ab3f5df`. Target remains
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Full CPU parity remains incomplete under the [acceptance criterion](cpu-parity-acceptance.md).

## Cause and correction

The unchanged `check/default_arm_family.bend` and `check/neq_reflexivity.bend`
correctly failed checking, but the port emitted an internal type-mismatch report.
Pinned upstream instead reports the normalized expected type, the non-inferrable
`{==}` term, the current context, and the exact proof occurrence. In the inequality
case the expected type is `@_:{0 == 0 : U32} -> Empty`, rather than the alias `neg`.

Pinned [check-rfl](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3571)
first weak-head reduces the expected type, then rejects its non-equation shape.
The port had the same decision but omitted optional presentation metadata on this
one branch. It now attaches the existing structured diagnostic using the original
expected-type and proof identities. Rendering normalizes types and context on the
existing bounded clone, after rejection; exhausted rendering retains the original
error. No acceptance, evaluation, sharing, termination, guard or budget rule changes.

Public `KernelError` message, Display and equality remain unchanged. Its derived
Debug representation can change because it includes the attached diagnostic. These
tracked failures now use the existing CLI `Ok(CliOutput)` with `exit_code() == 1` boundary
instead of a host `Err`; callers must inspect the status. The binary does so in
`src/lib.rs`, and CLI check/eval/run/compile tests assert nonzero status and no
artifact. There is no new public signature, parser-error or loader-error change.
External invoke consumers remain unaudited; this is not a claim that all downstream
callers safely handle status-bearing output.

## Evidence

Full standard gate: **1256 passed / 0 failed / 49 ignored**, 185 result groups, exit 0 in 991.664 seconds. Format, strict production Clippy, all-feature build and real Node TCP/UDP provider checks pass. Focused state/integration/boundary checks: **51 / 0 / 0**; targeted strict Clippy passes. [Validation receipt](cpu-contextual-proof-diagnostics-validation.json).

The [matrix](cpu-contextual-proof-diagnostics-results.json) records **340 commands**: 336 matching observations and four strict-IO observations. It includes **216 exact negative comparisons**, **48 separate negative phase probes**, **26 actual executions** (24 generated C-O3/JS and two native IO), **eight pure normalizations**, and **161 repeated pairs with identical status/streams**. No unexpected outcomes or timeouts. All 12 negatives reject before artifact generation; nine diagnostic differences close and three earlier/equality refusals remain unchanged. Invalid-source runtime execution is unrun, not a passing runtime cell.

The [fixtures](../tests/fixtures/cpu-contextual-proof-diagnostics/README.md) preserve
both original files byte-for-byte under their Apache-2.0 attribution. Authored
controls cover scalar/function expectations, nested matches, imported and template
sources, repeated binder names and an earlier unannotated-term failure. True and
false equality proofs, including the closed-partial-call regression, keep their
previous behavior. Pure normalization, native IO and generated execution remain
separate observations. Six emitted port artifacts are compared byte-for-byte with
the baseline; that does not assert repeated generation of arbitrary programs.

State tests retain the internal error, source/type identities, shared ownership,
aliases, fresh identifiers and exact live work charges. Earlier semantic budget
and depth failures remain errors; rendering failure falls back without resetting
allowances or changing rejection. Existing cancellation/untracked-source and
writer/flush controls now include both originals. Pure execution cancellation is
still only before normalization and before output, not mid-normalization.

## Reproduction and storage

```sh
source <tools>/activate.sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib contextual_reflexivity -- --nocapture
cargo test --locked --features extended_observability --test cpu_contextual_proof_diagnostics --test cpu_comptime_rebind --test kernel_normalization --test kernel_open_closures
cargo test --locked --features extended_observability --lib cli::checker_diagnostic::tests
cargo clippy --locked --features extended_observability --lib --test cpu_contextual_proof_diagnostics -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Commands and raw stream/artifact hashes are in the matrix and validation receipts.
Host debug-information/incremental overrides are temporary Cargo settings, not
Bend runtime optimization settings. Differential C execution uses the pinned CLI's
`-O3` profile; port C retains strict warnings. Repository tests retain their own
unchanged compiler settings. Process comparisons are bounded at 30 seconds,
the gate at 1,200 seconds. Workspace build preflight is 11 GiB; stop floors remain
10 GiB workspace and 1 GiB temporary storage.

Initial Cargo-cache relocation preserved 198,876,287 bytes but observed only 618,496 bytes of extra workspace space because the cache largely lived in the environment base layer. Relocating one inactive generated executable, retaining its original path and a verified lossless archive, then reclaimed 100,347,904 bytes net. No unique source or evidence was removed. Gate preflight had 11.093 GiB workspace free; minima were 11.070 GiB workspace and 3.146 GiB temporary storage. No timeout or disk-floor stop occurred.

Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, Clang 19.1.7,
PowerShell 7.5.4 and docs pin `50e89d6897ce2a65167ef65b17c5fa83fe849158` stay frozen.
Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

## Remaining requirements

The full [checking census](cpu-contextual-proof-diagnostics-census.json) measured all **1,302** originals: **366 both accept / 380 exact rejections / 494 acceptance disagreements / 56 different rejections / six oracle-host failures**. Only the port stderr streams for the two target originals changed; both now match upstream exactly. All acceptance statuses, failure phases, upstream streams and the other 1,300 port streams are unchanged. There is no oracle-variation adjustment.

All **385 historical IO members renew exactly**; the separate 386 missing-IO symptoms do not enlarge that cohort. Outside it, the measured residual is **165 = 109 acceptance disagreements + 56 rejection differences**, down from 167 by these two diagnostic closures. Historical triage labels remain 109 acceptance/rejection mismatches, 27 diagnostic candidates and 29 other rejection differences. No remaining observation is waived.

Original receipt linkage is **484 linked / 818 unlinked**; this is applicability-led coverage, not completed parity. The unlinked categories are 283 checker rejections, 521 interpreter/compiled candidates, one checking-only module without main, and 13 interpreter-only fixtures. The next independent measured gap is `check/meet_stuck_pending_ref`: port expected-type text `(q <&> &1)` differs from upstream `q <&> &1`, while status/source and observed type agree. No correction is claimed for it here.

Base callback coordinates (390 versus 1017), `nat_first` ordering, unadopted compact
Nat/string lowering and the external invoke audit remain unresolved. Existing
strict-proof exclusions, resource/foreign restrictions, printer/host failures,
F32.read and hole disagreements remain unwaived. Supplementary all-target Clippy,
GPU hardware, desktop validation and newer upstream adoption were not run.

Compact pre-fix and current receipts are committed. Bulk raw logs, generated
artifacts, historical binaries and cache archives remain environment-local and
are not preserved by this Git push. No unique historical evidence was discarded.
