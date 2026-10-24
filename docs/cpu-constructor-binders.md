# Constructor continuations and generated match binders

Tested source: `0cabee21d28aa6e8fb96164b860249db29510210`. Upstream target remains
[e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
All three [original fixtures](../tests/fixtures/cpu-constructor-binders/README.md)
retain upstream bytes and Apache-2.0 attribution. Only tests, fixtures and evidence
change; the added internal kernel code is in an existing `#[cfg(test)]` module.
There is no production implementation or public API change.

## Exact normal forms and binder identity

| Original / observed entry | Pinned and port result | Applicability |
| --- | --- | --- |
| `printer_fixpoint` / `main` | `EPrint{1n, x => EDone{1n+x}}` | Strict normalization; function-field executable printer refuses |
| `printer_fresh_binder` / `pick` | `\{Succ: \{Succ: (x => 2n); (_4 => 0n)}; (n => 0n)}` | Execution-context helper normalization; original main executes IO |
| `printer_fresh_binder_pick` / `main` | Same nested match above | Strict normalization; function executable printer refuses |

The old source comment mentions `_6`; the actual pinned normal form uses `_4`.
Tests follow measured pinned output and graph identity, not that stale comment.
The pinned [nested-pattern lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2723)
mints names from fresh indices. The comparison retains match constructor names,
arms, fallbacks, lambda nesting, references, payloads and bound-variable distances.
It performs no eta conversion, unfolding repair or textual substitution.

[Kernel tests](../tests/kernel_constructor_binders.rs) compare exact text and
Facet-decoded pinned graphs separately. They also parse the printed terms back
and compare structure. The continuation's leaf refers to its own lambda ID.
[Shared binder assertions](../tests/support/constructor_binder_assertions.rs)
verify the `x`, `_4` and `n` branch lambdas have three distinct IDs. The two nested
matches have equal structure even though their entries live in different programs.
This is core binding evidence, not a claim that printed names alone prove identity.

The IO original cannot be accepted as a strict proof program: `check` and
`eval --entry pick` still refuse `main: undefined name IO`. Its typed executable
program is accepted. The existing private
[execution-normalization test module](../src/kernel/executable_normalization_tests.rs)
clones and resets its bounded engine to compare `pick` with the pinned graph and
text. It explicitly checks that strict proof checking still refuses IO. No public
normalization API, executable-to-proof conversion or safety exception is added.
The external graph matrix pairs the two pure originals and `partial_app_snf`;
it does not mislabel the private IO unit assertion as an external proof API call.

Named-call reconstruction remains covered by the full gate's ten blocked-call
cases and budget/sharing regressions, plus renewed external `partial_app_snf`
graph agreement (27 nodes on both sides). Existing demand, first-error ordering,
structural/work/depth, source-hole and cancellation controls remain intact.

## Bounded applications and actual execution

Four continuation applications use Nat inputs 0, 1, 2 and 7. The new MPL-2.0
[wrappers](../tests/fixtures/cpu-constructor-binders/wrappers) preserve the original
constructor tag/text and the returned constructor tag/value. Their rows are
`[1, 1, 0, 1]`, `[1, 1, 0, 2]`, `[1, 1, 0, 3]` and `[1, 1, 0, 8]`:
EPrint tag, original text, EDone tag, returned Nat represented as U32.
The pinned probe independently applies the original continuation and observes
complete `EDone{1n}`, `EDone{2n}`, `EDone{3n}`, `EDone{8n}` values.

Each nested-pattern function is applied to 0, 1, 2, 3 and 7, producing
`[0, 0, 2, 2, 2]`. These inputs cover the outer default, inner default and selected
successor arm with multiple unused payload values. Persistent kernel tests compare
complete result graphs, separately from the executable wrappers' projections.
The IO helper's five applications are execution-context observations; the pure
companion's five are checked through the strict public API. These 14 bounded
samples are not a proof of arbitrary-depth equivalence.

The unchanged IO original runs and prints exactly `2\n` in native execution,
upstream C/JS and port C/JS, with exit 0 and empty stderr. The
[persistent compiler test](../tests/compiler_constructor_binders.rs) separately
checks all three wrappers and this IO original through the native driver and
both generated targets. Pure CLI normalization is not relabeled as native runtime.

Direct upstream execution under Node 24.19 still exits 1 with empty stdout and
`TypeError: require is not a function\n`. The pinned
[dynamic JS runner](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1274)
passes `import.meta.require` to its generated function. This is recorded as the
existing direct-Node host limitation, not an exact runtime match or a new semantic
exception. No Bun installation or host workaround was attempted. Generated JS
under Node succeeds and remains a separate lane.

Actual upstream C/JS and port C/JS/native emitters all refuse the two pure originals
because their values contain functions. Each returns exit 1, empty stdout, exact
stderr and no artifact. The original type is `Eff` for the constructor continuation
and `@n:Nat -> Nat` for the pure nested match. Successful wrapper execution does
not create an original runtime pass for either.

## Validation and reproduction

- Full gate: **973 passed, 0 failed, 49 ignored**, 120 result-bearing targets.
  Formatting, strict Clippy, all-feature build and real socket-provider checks pass.
- Focused lib/new tests/normalization/demand/effects run: **286 passed, 0 failed,
  1 ignored**. All new tests pass at the tested source on their first run.
- Open/applicability matrix: **52 observations**: 12 pure normal-form outputs,
  20 actual printer refusals, six application probes, six type/phase probes,
  four strict IO exclusions, two IO-helper text observations and two pinned
  IO-helper graph observations. Result lanes repeat twice; type/phase probes once.
- Paired graph matrix: **12 observations**, three sources × two implementations
  × two repeats. The two new pure originals and `partial_app_snf` agree exactly.
- Runtime matrix: **72 observations**, comprising 46 exact successful outputs,
  24 generation/build successes and two direct-Node host blocks. External C uses
  the pinned O3 profile, one thread and GPU off. The persistent compiler test uses
  strict warnings without an explicit optimization flag; profiles remain distinct.
- All **1,302 checking statuses and streams** match the preceding port receipt:
  439 expected negatives reject; other fixtures are 366 accepted / 497 rejected.
  This is status/stream regression evidence, not universal upstream diagnostic parity.
- Fourteen positive controls pass; 18 observations retain three hole/TODO
  discrepancies. The 49 ignored tests remain unrun: 47 GPU hardware and two optional
  profilers. Final comparisons show no timeout or observed result nondeterminism.

No shift program ran in this checkpoint. Earlier shift proof/executable-emission
receipts keep their original identity. Existing external-consumer API caveats remain
open; this batch adds no public API changes or downstream compatibility claim.

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test kernel_constructor_binders --test compiler_constructor_binders --test kernel_normalization --test executable_demand --test runtime_effects --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$BEND_REFERENCE/bend2/main.ts" "$ORIGINAL"
"$BEND_BINARY" run "$ORIGINAL"
"$BEND_BINARY" --output-format json eval "$IO_ORIGINAL" --entry pick
node "$BEND_REFERENCE/bend2/main.ts" "$IO_ORIGINAL" -o "$WORK/up.c"
clang-19 -std=c11 -O3 "$WORK/up.c" -lpthread -lm -o "$WORK/up"
"$WORK/up" --threads 1 --gpu off
"$BEND_BINARY" compile "$IO_ORIGINAL" --executable --target c --output "$WORK/port.c"
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$WORK/port.c" -pthread -lm -ldl -o "$WORK/port"
"$WORK/port" --threads 1 --gpu off
```

Toolchains remain Rust 1.96.0, formatting nightly 2026-05-24, Node 24.19, Clang
19.1.7 and PowerShell 7.5.4. Rust debug overrides control host build storage, not
Bend policies or the C profile. Deadlines remain 30 seconds per external comparison,
60 per audit child/private probe build, 20 per persistent generated-child execution
and 1,200 per gate. Full commands, source/binary/artifact hashes, status and streams
are retained privately; [compact evidence](cpu-constructor-binders-evidence.csv)
contains no private paths or raw diagnostics.

The disk preflight prevented the first gate launch: removing five inactive focused
integration executables left 10.991 GiB free. The gate did not start. Removing one
verified inactive unit-test executable restored 11.009 GiB, and the gate then ran
for **324.89 seconds**, with a minimum **10.223 GiB** free.
The 11 GiB build preflight, 10 GiB reserve, 2 GiB emergency workspace floor and 1 GiB
temporary floor are unchanged. After capture, cleanup retired 116 inactive integration
executables, one unit-test executable and one inactive alternate-feature CLI cache.
The captured CLI is excluded by hash/link identity. Source, dependencies and unique
historical evidence remain; the current binary also has a losslessly verified gzip
snapshot. Final headroom is **11.023 GiB**. Recheck before builds.

## Remaining coverage

Source `0cabee21d28aa6e8fb96164b860249db29510210` qualifies the unchanged `eval/printer_fixpoint`,
`eval/printer_fresh_binder` and `eval/printer_fresh_binder_pick` originals under
separate applicable contracts. Exact graphs/text preserve the constructor's
continuation, generated `_4` binder and three distinct match-arm identities.
Fourteen bounded applications retain complete values. No production defect or
implementation change was needed. [Report](cpu-constructor-binders.md).

Clean-source gate: **973 passed, zero failed, 49 ignored**; focused controls:
**286 passed, zero failed, one ignored**. The open/applicability matrix has 52
observations. Twelve paired graph observations include renewed `partial_app_snf`;
the IO helper's execution-context graph comparison is separately covered by the
private unit test, without exposing a proof API. The 72-observation runtime matrix
contains 46 successful exact outputs, 24 generation/build successes and two known
direct-Node IO host blocks. The unchanged IO original prints `2` in native and
upstream/port generated C and JS. The two function-containing originals refuse
executable printing exactly; their wrappers do not fabricate original runtime passes.

All 1,302 checking statuses and streams match the preceding port receipt; all
439 expected negatives reject. Fourteen positive controls pass; 18 observations
retain three unwaived hole/TODO cases. Strict IO checking remains excluded,
separate from successful executable checking. No shift programs are executed.

Detailed receipts link **332 originals**; **970** lack detailed receipts:
**358 checker negatives, one checking-only module, 68 interpreter cases and 543
compiled candidates**. Two interpretation rows and one runtime row are new;
earlier 144 interpretation, 93 runtime and 92 checking rows keep their historical
identities. Linked receipts include partial/blocked cells and do not establish
CPU parity. Imported-printer/JS-timeout, erased-hole/TODO, host/resource/foreign/
F32.read/error-order/API gaps and the unfinished 688-vector exploration remain
unwaived. Frozen dependencies, references, toolchains, guards, budgets,
cancellation and approved demand policy; no installs, pushes, desktop or GPU work.

Next source-reviewed batch: `eval/underscore_print`, `eval/wildcard_chain_print`
and `eval/efq_arm_value`. These distinguish fresh wildcard binders and a selected
empty-match value from a genuinely blocked named call. No inhabitant of `Empty`
may be fabricated to claim a closed runtime test. Only source/inventory review
is complete; actual applicability and results remain unqualified.

The [acceptance criterion](cpu-parity-acceptance.md) remains unchanged. Linked
receipts include historical and blocked cells; they do not establish full CPU parity.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `be71a73e25dac2e64dee50e86e0205f5e19dfbcaa64e922da066c6a356c82812` |
| Frozen Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| snapshot.json | `ba47a425b4ccc8f087dfb6fe8658157c0c2f3a1255ad01903db1ff9b39bb5a9f` |
| snapshot/teamy-bend.gz | `eab91565761bef0244e9b6ce0d3a9a294c836bbdb9a087e7916ef2127a34527f` |
| full-gate-final.json | `ba276be3451f6a5e5dcf11fd698b1d62e591b267fa34b5bd7da32f1b6f789889` |
| full-gate-final.log | `11b7d1e0f9258c09c53a8920711496723e275f13a7d1b5df69444a17297665e5` |
| focused.json | `a8423fd2d812749c4f36d3c2617bf1bbd5f06424ae81b3a7b7e6cc7f06268d66` |
| applicability-preflight.json | `b3ca78631f7c29d334cfd60aa140b25d213998f63fef26e6c816e69c461195be` |
| wrapper-preflight.json | `eaada56705bbf74138505e847556f41d450c770fb31d3d8fb25f06d63b188e43` |
| graphs-matching/receipt.json | `90768a2f024aa262eb2ddae8bb05bd1c8a1ac813da61390fe21fd25e0aca2a5f` |
| open-matrix/receipt.json | `6aa25ee99b74071a174be4f73352895ee73fe74bd2ec8e5265c7af82ffb06fb3` |
| closed-runtime/receipt.json | `a571edbee3a4879d108ab4a5eafb5ca94369991fee1029a394623e39451bbff9` |
| audit/receipt.json | `1acc6c62254395f3ec1cdc90b7df9680bf394f39ea1ecb2d9cd72a74f48160f8` |
| audit-delta.json | `cbb5d2b1a74966868faf8dc428e17ce0a31247bdb1ee66946ad324126855cc41` |
| controls.json | `c9268f9450001d8b9f9d25cd07c53865b43fe5389d6648073d12903de86a4432` |
| coverage-summary.json | `039a1dcb01e00178a3ede6389f5e3ec84dd375575ef2e35b771774a9593a79cc` |
| gate-preflight-block.json | `0cab8accedf2fa527eb5ae095fe8ab5834895abe2d625a03c64be9ddb3973dd2` |
| pre-gate-unit-cache-reclaimed.json | `a620680b342b2707f2edf8288c840cc0cb14f300abf4f1c450f9f89c03d48ae5` |
| inactive-test-cache-reclaimed.json | `e6bce9f72b7f867a3cda9fe62ec85947992155dd2e207b3c340e501c739d873b` |
