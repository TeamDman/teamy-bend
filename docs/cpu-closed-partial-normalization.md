# Closed partial-application normalization

Tested source: `cd7cabbda37192a7d401d1b3b1a3b9caf37c9f7a`. Upstream remains
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Full CPU parity remains incomplete under the [acceptance criterion](cpu-parity-acceptance.md).

## Demonstrated defect and correction

The unchanged [original](../tests/fixtures/cpu-comptime-rebind/positive/local.bend)
previously normalized to `Nat.add(11n, 1n)` rather than `12n`. An annotation-only
control, `{Nat.add(11n) : Nat -> Nat}(1n)`, and an ordinary returned partial call
reproduced the same defect. A custom three-argument constructor function was also
stuck after two pending arguments. All now reduce to their complete expected graphs. The port incorrectly rejected a true proof
that the Nat call equals `12n`; a false proof remains rejected.

The pinned [weak-head machine](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2807)
keeps pending application frames while reducing a computed head. The port reduced
that head separately, rebuilt its now-saturated named application, then stopped
because no lambda or match had consumed an argument. Strong normalization only
normalized the children of the residual application.

The [reducer](../src/kernel/reduce.rs) now retries the rebuilt named spine.
The scan only borrows references; existing argument processing is unchanged and
the retry pays the existing tail-reduction charge. Direct named dispatch still
stops on underapplication, undefined/unfilled references and blocked matching.
There is no printer substitution, Nat special case, blanket fixed-point loop,
new dependency, public signature change or safety-limit increase.

## Evidence and applicability

Standard gate: **1252 passed / 0 failed / 49 ignored** across 184 result groups, exit 0, 970.330 seconds. Format, strict production Clippy, all-feature build and real Node TCP/UDP provider checks pass. Corrected focused checks: **9 / 0 / 0**; targeted strict Clippy passes. Earlier harness failures are retained in the [validation receipt](cpu-closed-partial-normalization-validation.json).

The [matrix](cpu-closed-partial-normalization-results.json) records **298 commands**, **110 identical repeated status/stream pairs**, **74 actual executions** (72 generated C-O3/JS, two native IO) and **44 pure-normalization observations**. No unexpected outcomes or timeouts. All 12 generation refusals match exactly across upstream/port C/JS: the custom constructor fixture lacks Base, and open/partial function values have no executable printer. Execution is unrun for those lanes. Three negative sources retain exact rejection diagnostics; the true-proof control now accepts and prints `{==}`. Two strict-IO phase observations remain exclusions, not proof passes.

Emission occurs once per artifact: repeated execution determinism does not establish repeated generation or byte identity against baseline. Independent review verified source/test behavior, completed matrix, exact refusals and census reconciliation.

Kernel tests compare complete constructor/application/binder graphs separately
from exact text, including normalization and checked evaluation of the original.
Open and partial function observations remain distinct from closed applications.
Pure `run_main` uses the normalizer; only the explicit IO wrapper exercises the
native IO runtime. Generated C and JavaScript are separate executed lanes.

[Compact pre-fix streams](../tests/fixtures/cpu-closed-partial-normalization/before.json)
retain the actual baseline binary/source identity and all authored fixture hashes.
Historical reports remain historical; the current expected value is corrected
only after the underlying graph reduces. The full gate renews existing blocked
named-call, complete 27-result aggregate, sharing, stack and guard controls.
New controls retain lazy unused cycles, demanded error ordering, exact work
charges, fuel exhaustion, nesting refusal and memo/depth cleanup. Pure execution
cancellation remains checked before normalization and before output; this does
not claim mid-normalization cancellation.

## Reproduction and host profile

```sh
source <tools>/activate.sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test kernel_normalization computed_
cargo clippy --locked --features extended_observability --lib --test kernel_normalization --test cpu_comptime_rebind -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

These are temporary host Cargo profile overrides, not Bend runtime optimization
settings. C oracle execution uses the pinned CLI's `-O3` profile; port C retains
strict warning flags. Individual comparisons have 30-second process bounds;
the repository gate has a 1,200-second bound. Workspace build preflight is 11 GiB,
with stop floors of 10 GiB workspace and 1 GiB temporary storage. Preflight workspace free space was 11.023 GiB; observed minima were 11.000 GiB workspace and 3.513 GiB temporary storage. No timeout, floor stop, cache removal or relocation occurred. Existing debug/dependency-cache links and debug-information overrides were retained.

Frozen Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, Clang 19.1.7,
PowerShell 7.5.4 and docs pin `50e89d6897ce2a65167ef65b17c5fa83fe849158` remain.
Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

## Remaining requirements

The renewed [1,302-fixture checking census](cpu-closed-partial-normalization-census.json) is exactly unchanged from `d0176c3a104688fbe21bb79d9e05330bdc9ce08f`: **366 both accept / 378 exact rejections / 494 acceptance disagreements / 58 different rejections / six oracle-host failures**. Every port and upstream stream, status, phase and fixture hash is unchanged. All 385 historical IO members renew exactly. Outside that cohort, **167 differences remain: 109 acceptance disagreements and 58 rejection differences**. The separate 386 missing-IO symptoms do not expand the historical cohort. Historical labels remain 109 acceptance/rejection mismatches, 29 diagnostic candidates and 29 other rejection differences, without waiver.

The corrected authored normalization/proof cases are outside that inventory census. Linkage remains **482 linked / 820 unlinked**, not completed parity. The next independent measured gap is contextual non-inferrable-proof rejection in `check/default_arm_family` and `check/neq_reflexivity`; their port host errors still differ from pinned normalized type/source diagnostics. No correction is claimed for them here.

Keep `nat_first` diagnostic ordering pending and unwaived, compact Nat/string
lowering unadopted, Base callback coordinates (390 versus 1017) unresolved, and
the external invoke-caller compatibility audit outstanding. Existing strict-IO
proof exclusions, resource/foreign limitations, C/JS printer/host failures and
unrun lanes retain their previous classifications. Supplementary all-target
Clippy failures were not renewed by targeted or standard Clippy success. GPU
hardware, desktop checks and newer upstream adoption were not attempted.

The compact receipts are committed. Raw logs, captured binaries and historical
large evidence remain in environment-local storage outside Git; pushing this
branch does not preserve those files. No unique historical evidence was removed.
