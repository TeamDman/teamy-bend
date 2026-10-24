# Named calls blocked on constructor fields

Tested source: `c123b3eaafe12f9ef40880aaa9922c22ec5465bc`. The target remains
[e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
This checkpoint fixes the demonstrated open-term normalization defect. Full CPU
parity remains incomplete under the [acceptance criterion](cpu-parity-acceptance.md).

## Cause and correction

The unchanged [partial_app_snf original](../tests/fixtures/cpu-blocked-named-calls/partial_app_snf.bend)
normalizes a small lambda-calculus interpreter under its rebuilt binders. The
[previous checkpoint](cpu-context-scope.md) established a structural difference:
the port exposed nested matches where the pinned interpreter retained named calls.
This was not a printer abbreviation or alpha-renaming difference.

The pinned [weak-head contract](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2807)
retains a stuck named application. Its
[Ref/Lam/Mat machine](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2864)
carries `lhs` through demanded constructor fields and reconstructs the named call.
Previously the port's generic `consume` recursively reduced the selected arm
applied to its fields and lost that named-call boundary.

The [reducer](../src/kernel/reduce.rs) now tracks pending formal arguments and
constructor fields for named definitions. A blocked field returns the original
shared application. Entering another call hands off all remaining arguments
together, allowing an underapplied callee to become saturated. Anonymous matching
continues through the existing path. Annotations, simultaneous lets, proved
rewrites and constructor fallbacks retain their existing interpretation. No
printer changes are used to hide a residual match.

For example, the [minimal case](../tests/fixtures/cpu-blocked-named-calls/stuck_field.bend)
now normalizes exactly to `x => f(Box{x})` instead of exposing the inner match.
The original produces exactly:

```text
TLam{x => TLam{x => snf9(4n, Sp{wnf(1n, Ap{x, wnf(1n, Ev{x^0})})})}}
```

| Graph observation | Pinned nodes | Current port nodes | Prior port nodes |
| --- | ---: | ---: | ---: |
| Original partial application | 27 | 27 | 166 |
| Blocked nested field | 5 | 5 | 9 |
| Fuel plus blocked nested field | 9 | 9 | 16 |
| Blocked first argument | 6 | 6 | 6 |
| Function-returning control | 6 | 6 | 6 |

These are tree-node occurrences in the same bounded comparison vocabulary, not
heap allocations or a benchmark. Comparison preserves references, applications,
constructors, match arms and bound-variable distances. It neither evaluates away
differences nor normalizes printer names to force equality. All ten selected
cases have exact graphs and text. The external Rust probe links the gate-built
library with the captured CLI's exact feature/profile identity; it is not a
stale all-feature artifact. Persistent [normalization tests](../tests/kernel_normalization.rs)
compare both checked normalization and strict evaluation against the pinned graph.

## Guard, sharing and closed-result controls

The production change is limited to reduction of named formal patterns.
Existing 2,000,000-work and 128-depth limits remain. Every pattern action charges
work, and pending constructor-field work retains an owned depth frame. Returns
and errors unwind those frames without resetting fuel, aliases or fresh IDs.

Three [unit regressions](../src/kernel/check.rs) cover shared-pointer identity,
unchanged input/alias ownership, blocked or unselected later cyclic fields,
first-demanded-error ordering, exact finite-work exhaustion and depth-limit
refusal/unwinding. Adjacent integration controls retain rejection of source holes,
false closed proofs and nondecreasing recursion. Existing executable demand,
cleanup and cancellation regressions pass; no native execution implementation,
sharing policy or approved strict live-argument contract changes here.

Five [closed wrappers](../tests/fixtures/cpu-blocked-named-calls/wrappers) separately
observe concrete results. Three minimized functions at Nat depths 0, 1 and 2
give `[0, 0, 1]`; the fuel-limited one gives `[0, 0, 0]`. Applying the original's
two rebuilt lambdas to `TNum{7}` and `TNum{9}` gives `[1, 7, 9]` (application
tag, function payload, argument payload). Full constructor-graph regressions
complement those projections. These bounded samples are not a proof of arbitrary
closed-value equivalence.

The [compiler regression](../tests/compiler_blocked_named_calls.rs) runs all five
observations through the native executable driver, strict generated C and generated
JavaScript, asserting exit 0, exact stdout and empty stderr. The external matrix
separately compares pinned interpretation, port pure CLI and upstream/port C and
JS. Pure CLI normalization is not relabeled as native executable evidence.

Actual original C/JS emitters and port C/JS/native emitters all refuse:
`Error: main's type Trm cannot be printed (a function, a Type, an erased or dependent field)`
with exit 1, empty stdout and no artifact. The nine authored function originals
first refuse the missing-Base precondition; separate Base-importing aliases reach
the actual function printer and refuse exactly. Original bytes remain unchanged.
There is no fabricated original runtime pass.

## Validation and identities

- Standard gate: **966 passed, 0 failed, 49 ignored**, 116 result-bearing targets.
  Formatting, strict Clippy, all-feature build and real socket-provider checks pass.
- Focused pre-closed run at `15d3ce7477fde6fa9afc490b7023c159ab46a9f4`:
  **308 passed, 0 failed, 1 ignored**. The two new closed tests then pass at the
  tested source above, and all run again in its full gate.
- Graph matrix: **40 observations**, ten cases × two implementations × two repeats.
- Open/applicability matrix: **270 observations**: 60 exact normalized results,
  20 type/checking-phase probes, 100 original emission refusals and 90 separate
  Base-enabled function-printer refusals. Result lanes repeat twice; type/phase
  probes run once per original. [Compact evidence](cpu-blocked-named-calls-evidence.csv).
- Closed matrix: **90 observations**: 60 exact outputs and 30 generation/build
  successes. O3 is explicit in both external C lanes. The persistent Rust test's
  strict C helper uses no explicit optimization flag; these are distinct profiles.
- Diagnostic renewal: **180 observations**, 162 exact failures and 18 checking-phase
  probes. The contextual suffix correction and earlier false-proof/constructor/
  rewrite/matched-value diagnostics remain exact. No output artifacts appear.
- Checking audit: **1,302 unchanged exit statuses**: 439 expected negatives rejected,
  plus 366 accepted and 497 rejected other fixtures. Only two stderr streams change,
  explained below. This is status coverage, not universal exact upstream parity.
- Fourteen positive controls pass; 18 observations preserve three unwaived hole/TODO
  differences. The 49 ignores remain unrun: 47 GPU hardware tests and two optional
  profilers. No new dependencies or unavailable host are required by this batch.

`proof/shift_left_mask` and `proof/shift_right_mask` formerly exhausted work at
`shln_compose`/`shrn_compose`. They now get through that work and reach the existing
strict-proof exclusion, `main: undefined name IO`. Twenty repeated observations
compare historical/current proof checks, pinned checking phase and upstream/port
JS emission. Pinned checking and both executable emitters succeed. No generated
shift program is executed; no runtime ledger row or diagnostic exception is added.
The changed normal form avoids the earlier expansion without raising a limit.

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test kernel_normalization --test executable_demand --test runtime_effects --test cli_checker_diagnostics --quiet
cargo test --locked --features extended_observability --test kernel_normalization blocked_call_closed --test compiler_blocked_named_calls --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$BEND_REFERENCE/bend2/main.ts" "$ORIGINAL"
"$BEND_BINARY" run "$ORIGINAL"
"$BEND_BINARY" --output-format json eval "$ORIGINAL"
"$BEND_BINARY" compile "$ORIGINAL" --executable --target c --output "$WORK/refused.c"
node "$BEND_REFERENCE/bend2/main.ts" "$WRAPPER" -o "$WORK/up.c"
clang-19 -std=c11 -O3 "$WORK/up.c" -lpthread -lm -o "$WORK/up"
"$WORK/up" --threads 1 --gpu off
"$BEND_BINARY" compile "$WRAPPER" --executable --target c --output "$WORK/port.c"
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$WORK/port.c" -pthread -lm -ldl -o "$WORK/port"
"$WORK/port" --threads 1 --gpu off
```

Host Rust debug overrides only control build storage; they do not change Bend
budgets or the C O3 profile. Toolchains remain Rust 1.96.0, formatting nightly
2026-05-24, Node 24.19, Clang 19.1.7 and PowerShell 7.5.4. Bounds remain 30 seconds
per external comparison, 60 per audit child/probe build, 20 per generated child
in the persistent compiler test, and 1,200 per gate. Completed final comparisons
have no timeout or observed result nondeterminism. Exact commands and complete
streams are retained privately with source, binary and artifact hashes.

Earlier failed attempts remain preserved: three unit tests initially used an
empty engine without registered definitions; the fixture now supplies a checked
engine. One guard assertion expected different established wording. The first
closed wrapper imports omitted `.bend`. An authored anonymous-match oracle needed
an expected type annotation. Finally, a private applicability adapter used invalid
comment syntax; the corrected attempt uses a typed Bend wrapper. These were test/
harness corrections, with separate failed receipts, not waived production failures.

The full gate took **321.39 seconds**, starting with
**11.012 GiB** free and reaching **10.281 GiB**.
The 11 GiB build preflight and 10 GiB reserve remain. After binary capture, 112
inactive integration executables and one inactive unit executable were reclaimed,
with hashes/inodes recorded. Source and unique historical evidence remain.
Final free space is **11.019 GiB**. This batch's private
evidence is about 12 MiB, using a losslessly verified compressed binary snapshot.
The large retained storage component is earlier evidence under the tools root
(about 19 GiB); the recurring build peak here added about 0.73 GiB of rebuildable
test binaries. Recheck headroom before the next build. Emergency stop floors remain
2 GiB workspace and 1 GiB temporary storage.

No public API changes are introduced. Prior external-consumer implications of
structured KernelError/status-bearing invocation and emitter APIs remain open;
this reducer change does not establish an external-consumer audit.

## Remaining coverage

Source `c123b3eaafe12f9ef40880aaa9922c22ec5465bc` corrects named-call reconstruction when a constructor-field
match blocks. The unchanged `eval/partial_app_snf` now agrees with pinned upstream
in exact normalized text and structure: **27 graph nodes on both sides**, replacing
the earlier port's 166-node residual match. Nine authored adjacent cases cover
first/nested/second fields, fuel, fallback, computed constructors, callee handoff,
partial applications and anonymous matches. No printer workaround, raised limit,
new semantic exception or public API is introduced.
[Report](cpu-blocked-named-calls.md).

Clean-source gate: **966 passed, zero failed, 49 ignored**. Forty repeated graph
observations agree. The 270-observation open/applicability matrix separates exact
normalization from type/phase probes and actual emitter refusals. Ninety closed
observations agree; persistent tests separately cover native execution and both
generated targets. The original's function-valued `Trm` still refuses executable
printing, exactly as upstream; wrapper execution is not an original runtime pass.

The 180-observation diagnostic renewal passes. All 1,302 checking statuses remain
unchanged, including rejection of all 439 expected negatives. Two shift proofs
now get beyond former budget exhaustion to the existing strict-proof `IO`
exclusion; 20 separate observations confirm pinned checking and upstream/port JS
emission succeed. No shift program execution is claimed. Fourteen positive
controls pass; three hole/TODO differences remain unwaived.

Detailed receipts link **326 originals**; **976** lack detailed receipts:
**358 checker negatives, one checking-only module, 68 interpreter cases and 549
compiled candidates**. Only one interpretation row is added; prior 140 interpretation,
93 runtime and 92 checking rows retain historical identities. Linked receipts
include partial/blocked cells and do not establish CPU parity. Imported-printer/
JS-timeout, erased-hole/TODO, host/resource/foreign/F32.read/error-order/API gaps
and the unfinished 688-vector exploration remain unwaived. Frozen dependencies,
references, toolchains, guards, budgets, cancellation and approved demand policy;
no installations, pushes, desktop or GPU work.

Next independent source-reviewed batch: `eval/eta_expand_print`,
`eval/toplevel_ref_print` and `eval/hoas_binder_refresh`, distinguishing a retained
bare reference from eta-expanded and shadowed-binder values. Actual printer
applicability and bounded applications still require qualification; no new
execution credit is assigned to those three sources.

Historical receipts retain their actual source identities. In particular, the
preceding contextual diagnostic report's graph mismatch and closed observations
are historical evidence; they are not overwritten or relabeled as this run.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `44d358d8fa0e52881e85d82dfc39cb5bf0fdefca1e5c03c8060bc6d11f33ed32` |
| Unchanged original source | `707039643ac5c59a8944b5029ce9a71551e6b56501b91b83e93804ba2f144cbc` |
| Frozen Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| snapshot.json | `4285245cd8874dad8f39f5a3e628342e70fdb3f8da10964c97ebae625f262e8d` |
| snapshot/teamy-bend.gz | `557162e11a3573d064685ff7de22429b4716b0d716544343d5af57e85944a132` |
| full-gate-final.json | `def545d2ca55ab2eb521a202c724b0bef35075482cd07b08e6c8951ae8b446d8` |
| full-gate-final.log | `f38ba86e137d5b1d96a69f324107c9486ac71696f29133d7f9ddc196008c9fed` |
| focused-budget.json | `87f8a3e9bf9ba26ef664a0cda2637b3aa58a059ef8bfe1e26179995af1214f6c` |
| focused-budget-corrected.json | `f5e8a5e5ece3f188072b053982aaa04346da18c56fb371497cb2d101d451f16c` |
| focused-open.json | `17607502a010b491fb7a5ce3f4552ab65940cb3e92c3043f8a494a012b869a41` |
| focused-regressions.json | `2a6c600a4673ae785a0fc76ffe09834ddc89d099879b5a76f618bdc767e1e3d2` |
| focused-closed.json | `c195769d42623d162595ffc29ef03fbc38f5b23c5d603abdcfd39af66b6c7639` |
| focused-closed-corrected.json | `249e8eba2a31ee95b6dc5460fc996b411b2296c9cfefaa01c3047e4e4e5fc3e8` |
| graphs-matching/receipt.json | `b7f23933024f4f245a4ec06f3329df5c0e1fe140f43881959c75204755141c49` |
| open-matrix/receipt.json | `cf402f210a9da36e6ce45f47320eab7d166d7075a464add5ea4b1d0b145a4888` |
| open-matrix-corrected/receipt.json | `da8189a69b3ff59d8b8dc7796f81b2f152fb366f6dc51facbf9b8d2536d9f077` |
| closed-runtime/receipt.json | `e33a0590a128b42ec52c71afd2dce2210f4fcaec3fb1003ad60de524f0643b4c` |
| diagnostic-renewal/receipt.json | `3f4a65ae3828a1d33f2f0ee7fb1c3ec4360e9ee9e61c2a822fb8f088678774fa` |
| audit/receipt.json | `ee54f265bfecbeceb93aa0b85ecc4af469be373d07cf9ff80ec193e044d76548` |
| audit-delta.json | `e7b8daf512cb539a29f504225bd31f05d2687640dab35c86f86d9b131c6f34e4` |
| audit-stream-probes/receipt.json | `d830e564d63ede6a181a01360df3639f2e4ca6b9cfb9661b15246edebb933dbb` |
| controls.json | `8f206d7f48b288affd949adb902a8f603b8d729f0d61bcceedaf589825f434f6` |
| coverage-summary.json | `7cb7f1733ce7e375a65fa32ffe89128cddea755740b554b1a83cbfdc3184649e` |
| inactive-test-cache-reclaimed.json | `96a47bdfa897d6b0c7bed2845d631dc928bc3dc58e4c78be92d358b6b61e6455` |
| final-unit-cache-reclaimed.json | `6966bf6e6333e05fc6ef7cb1b221fdcec0d9036841d37af17f2736b1f0560ef1` |
