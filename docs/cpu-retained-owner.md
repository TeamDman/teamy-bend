<!-- SPDX-License-Identifier: MPL-2.0 -->
# Retained owners at the legacy C boundary

The exact reusable-U32 reproducer now compiles and runs in port C, returning
`6\n` with exit 0 and empty stderr. The owner-slot validation remains intact.
Clean tested source: `208832a8282a7e1688aa725341cdf9ea0b5447c0`.
Frozen upstream: `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

```bend
import Base
def main() -> U32:
  +n = {3 : U32}
  (n + n : U32)
```

[`original.bend`](../tests/fixtures/cpu-retained-owner/original.bend) is byte-identical
to the authored source retained in the previous checkpoint. No upstream fixture
or reference source was edited. The [compact before/after receipt](../tests/fixtures/cpu-retained-owner/before.json)
records the actual earlier binary `0aef62717e9587a21b7266c55e659ecdd6e65712`, whose
production source matches the starting checkpoint `4da922d`.

## Root cause and ownership contract

Pinned [quantities](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/guide/GUIDE.md#L197)
allow `+x` reuse when its type is Data. The pinned
[statement forms and runtime description](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/guide/GUIDE.md#L557)
distinguish reusable lets, parallel lets, inline words and shared heap values.
The concrete expected value is also confirmed by upstream C and JS execution.
These language contracts do not dictate the port's particular frame layout.

In the port, `flat_constructor` delegates native scalars to `flat_legacy`.
`Generator::expression` can return an inline constant such as `UINT64_C(3)`.
Previously `flat_legacy` returned that constant as a boxed flat value. A reusable
local could therefore reach `flat_take` or `owned_use` without a writable retained
slot. `duplicate_owned` correctly refused it: duplication may rewrite the original
owner to a shared reference, and resumable work needs that authoritative owner to
persist across a yield. An inline expression is not such an owner.

The four-line production hunk holds the legacy result in `tb_values` before it
enters the flat-value path. This repairs the invariant **before partial takes**,
including closure capture and indirect fork transfers. Merely holding the later
`OwnedLocal` insertion would miss a partial `flat_take` that duplicates first.
Other flat producers already retain their words; independent review checked them.

Ownership transfer flags and remaining-use counts are unchanged. The held copy
is the transferred owner; old scratch aliases are not additional owned roots.
Resume labels skip prior evaluation and initialization, so a duplication rewrite
continues to target the same retained slot. The existing validation in
`duplicate_owned`, ordinary duplication algorithm, cleanup, task ABI and
cancellation logic are unchanged. No bypass, warning suppression, special case
for the fixture or public API change is added.

The hold costs one existing **compiler generation charge and frame slot per
legacy boundary**. It does not change Bend runtime tick sites or increase any
limit. Observed registered frame-slot sums rise 14→15 for the one-use scalar,
255→260 for the boxed parallel control and 28→30 for the overflow control.
These are sums of declared capacities, not simultaneous live allocation or a
memory benchmark. [Capacity evidence](cpu-retained-owner-capacity.json).

## Execution and checking evidence

The [bounded differential matrix](cpu-retained-owner-results.json) contains
**374 commands**, all with retained exact outcomes and deterministic repetitions:

| Evidence | Count | Result |
| --- | ---: | --- |
| Executions | 146 | 126 successes; 20 intended Nat-overflow failures |
| Original pure normalization | 4 | Port and upstream each twice, exact `6` |
| C/JS generation and C builds | 84 | All succeed; emitted artifacts are actually executed |
| Positive/runtime-negative checking and phase probes | 112 | Executable checks accept; strict IO exclusions remain distinct |
| Linear-reuse and false-proof CLI negatives | 28 | Exact rejection status/stderr, no artifact |

The 146 executions include:

- The unchanged original in upstream C, port C, upstream JS and port JS, twice
  each: eight successful executions.
- Eleven positive IO companions in those four lanes plus the native runtime,
  twice each: 110 successful executions. Controls cover one/two/multiple uses,
  partial transfer, zero Nat, U32 wraparound, F32 addition, captured closures,
  dynamic fork children, heap strings, erased code and an unselected error branch.
- Boxed/string and scalar-fork C controls with four workers in both C lanes,
  twice each: eight further successes. Regression assertions verify actual join
  construction and two `FID_CLO_APPLY` children rather than assuming the source
  still selects the indirect fork path.
- Two IO overflow controls in five lanes, twice each: 20 exact failures. The
  ordering control prints `before\n`, emits the Nat overflow diagnostic, exits 1
  and never prints `after`.

Native `run_main` normalizes pure entries; it executes IO entries through the
native runtime. Thus the original port `run` result is normalization evidence.
The explicitly authored IO companion keeps the same value expression and calls
`IO.print(U32.show(value()))`; it establishes native **runtime** output `6\n`.
This clarifies earlier shorthand calling pure CLI results “native execution”;
historical commands and receipts are preserved, not rewritten.

Before/after generation probes contain 78 and 81 commands. All **18 retained-owner
C-generation refusals** across the nine affected pure controls and their IO
companions are corrected. The 52 overlapping non-C checking/CLI observations are
unchanged. Additional pure observations are not promoted to runtime parity:
F32 and string pure CLI outputs retain their existing core/residual forms, and
the pure overflow program still reaches the normalizer depth refusal. Its IO
companion instead reaches the runtime Nat bound. Those mode/resource distinctions
remain explicit; this patch changes none of them.

## Regression and unresolved-contract controls

Standard clean-source gate: **1,190 passed / 0 failed / 49 ignored**. The targeted pre-commit run passes
**59 tests**, including existing continuation, reclamation, parallel-worker,
Nat/Word runtime, sharing, work-budget and cancellation regressions. Targeted
strict Clippy passes. [Exact commands and provenance](cpu-retained-owner-validation.json).

New C success tests inspect live words, blocks, frames, continuations and depth
after return; bulk teardown cannot hide leaked successful owners. The existing
worker-failure/cancellation tests remain the error-cleanup controls. New negatives
retain linear-use and false-proof rejection, overflow output ordering and an
intentionally reduced `BEND_MAX_STEPS=1` failure with the unchanged diagnostic
`evaluation budget exhausted`. Default safety limits were not changed. A fresh
normal-budget invocation succeeds afterward. No GPU execution or
mid-normalization cancellation coverage is claimed.

The primary upstream-O3 [demand renewal](cpu-retained-owner-demand.json) remains
**92 exact observations, six previously approved C-wildcard observations and two
unresolved C diagnostic-order observations**. The required-parity runner exits
**1**. Its port-C builds use the existing strict flags without an explicit
optimization flag; those commands are distinct from this family’s O3 matrix.
`nat_first` remains pending; no exception or order policy was added.

The last full checking census remains at `a1f8f4d`: 227 differences outside the
385 renewed historical IO members, comprising 110 acceptance / 117 rejection,
with five separate oracle failures. “Non-IO” was shorthand, not a proof that
every residual is unrelated to IO: the census records 386 missing-IO candidates,
and `reg/cop_let_descent_escape` remains inside the 227. No additional exclusion
is adopted. Its unstable
`dead_param_fork` observation and historical six-failure cohort remain as recorded
in [the census reconciliation](cpu-nat-pattern-census.json). That census was not
rerun or relabelled here. This compiler-only correction does not claim to resolve
checker acceptance differences. Coverage links remain **445 / 1,302**, with
857 unlinked entries classified by applicability; authored controls do not add
upstream inventory coverage. Full CPU parity remains incomplete.

Nat/string compact lowering remains unadopted. Existing F32.read, source-hole/TODO,
foreign, host, resource, oracle, external-API and supplementary all-target Clippy
limitations remain unwaived. The next concrete independent family in the retained census is unfilled-law
rejection diagnostics: `check/assert_fill_scope`, `check/axiom_use`, imported and
forward-reference companions. Both implementations reject live use, but the port
loses the pinned filled-definition diagnostic, Bend context and source location.
Renew those observations before correcting metadata; preserve refusal semantics.
Nat/string architecture and Nat-first order questions retain their separate review.

## Reproduction and preservation

Use the installed frozen tool activation, then:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --features extended_observability \
  --test compiler_executable_c_continuations --test compiler_executable_c_parallel \
  --test compiler_executable_c_reclamation --test runtime_review --test runtime_numeric
cargo clippy --locked --features extended_observability \
  --test compiler_executable_c_continuations -- -D warnings
"$BEND" compile tests/fixtures/cpu-retained-owner/original.bend \
  --executable --target c --output "$RUN/original.c"
"$TEAMY_BEND_CC" -std=c11 -O3 -Wall -Wextra -Werror -pedantic \
  "$RUN/original.c" -lpthread -lm -o "$RUN/original"
"$RUN/original" --threads 1 --gpu off
"$BEND" run tests/fixtures/cpu-retained-owner/io/original.bend
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 \
  -Reference "$UPSTREAM" -OutputDirectory "$FRESH_DEMAND" \
  -OracleOptimization O3 -Suite demand
```

`BEND` is the binary built at the tested source. The matrix receipt stores every
first command plus verified repetition equality and exact artifact hashes.
`$REPO`/`$UPSTREAM` denote the frozen checkouts; `$RUN`/`$EVIDENCE` are disposable
output and probe directories. Build the matching phase probe as documented in
[checking-census](../scripts/checking-census/README.md). Comparison children have
30-second limits; the established demand runner uses 60 seconds. Primary upstream
C uses its actual O3 profile; port C additionally retains strict warning flags.
Host Rust debug/incremental overrides manage build-cache size, not Bend semantics.

Minimum observed headroom was **11.285 GiB workspace / 4.734 GiB temporary**. Preflight remains 11 GiB workspace, stop floors 10 GiB workspace /
1 GiB temporary. No cache deletion, dependency/toolchain/reference/lockfile change,
installation, desktop/GPU use, main merge or release occurred.

Independent source and evidence reviews precede publication. The first targeted
run failed only because the authored low-step diagnostic expectation said “step”
instead of the existing “evaluation”; the draft Clippy failure required
`unwrap_err`. Earlier malformed authored controls were corrected after pinned
checking rejected them, with draft sources/receipts retained locally. None is
reported as a passing final-source run.

Compact failure receipts, fixtures and reproducible commands are in Git. Raw
logs, initial drafts, full source-bearing outputs, generated executables and
lossless binary snapshots remain local and **are not preserved by Git pushing**.
The reviewed feature branch is a checkpoint, not whole-parity acceptance.
