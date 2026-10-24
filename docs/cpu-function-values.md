# Function-valued normal forms and applications

Tested source: `19b35fa85c6e6a510df7863f0a6991c7c4bfc3f8`. Target:
[e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The three [original fixtures](../tests/fixtures/cpu-function-values/README.md) are
unchanged Apache-2.0 upstream bytes. This checkpoint adds bounded regression and
differential evidence; no production code, dependency, budget or policy changes.

## Different open forms, separately tested applications

| Original | Exact normalized text | Exact graph shape | Closed sample |
| --- | --- | --- | --- |
| `eta_expand_print` | `x => id(x)` | Lambda containing application of reference `id` to bound distance 0 | Nat 0, 1, 2, 7 |
| `toplevel_ref_print` | `id` | Bare reference `id` | Nat 0, 1, 2, 7 |
| `hoas_binder_refresh` | `x => x => x` | Two lambdas, leaf bound distance 0 | All nine ordered pairs from U32 0, 7, 4294967295 |

The first two functions give the same sampled results but have different normal
forms. The comparator performs no eta conversion, unfolding, name substitution
or textual repair. This follows the pinned
[weak-head treatment of underapplied references](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2807)
and [strong normalization](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3033).
The original eta fixture explicitly distinguishes conversion from printed normal
form. Bound-variable distances preserve identity independently of display names.

[Kernel regressions](../tests/kernel_function_values.rs) compare pinned graphs
with both `normalize_definition` and strict `evaluate`, repeat both operations,
and compare exact text separately. The shadowing test also inspects actual core
IDs: outer and inner IDs differ, and the leaf refers to the inner ID. The graph
oracle uses existing Facet decoding; no dependency or comparison vocabulary is
added. These are value-graph observations, not claims about identical heap layouts.

The external graph probe uses the gate-built library with the captured CLI's
matching Cargo feature/profile identity. All three graphs match twice. It also
renews the unchanged `partial_app_snf`: both implementations still have its exact
27-node graph. The full gate retains all ten blocked-call graph cases and the
sharing, work/depth, false-proof and structural controls introduced in the
[preceding checkpoint](cpu-blocked-named-calls.md).

The 17 applications are separate evidence. Both Nat functions return each sampled
Nat unchanged. All nine shadowing pairs return the second U32, including unequal
inputs and the maximum value. Checked application tests compare complete graph
structure and text, so the executable observation's small Nat-to-U32 projection
cannot hide a different Nat payload within this sample.

The public checked API accepts declared parameters; these originals have
zero-parameter `main` definitions returning functions. Therefore the new MPL-2.0
[Bend adapters](../tests/fixtures/cpu-function-values/wrappers) declare typed
`apply` parameters and apply `H.main()` in Bend. They preserve the original files
and the existing API arity check. Their main functions collect outputs:

```text
eta_expand_print:    [0, 1, 2, 7]
toplevel_ref_print:  [0, 1, 2, 7]
hoas_binder_refresh: [0, 7, 4294967295, 0, 7, 4294967295, 0, 7, 4294967295]
```

The upstream application probe applies the original returned value directly;
separate wrappers then agree across upstream interpretation, port pure CLI, and
upstream/port C and JS. The [persistent compiler test](../tests/compiler_function_values.rs)
separately runs the native executable driver and both generated targets, checking
exit 0, exact stdout and empty stderr. These bounded results do not establish
arbitrary-input equivalence, nor equate the distinct open forms.

Actual upstream C/JS and port C/JS/native emitters refuse each function-valued
original at the printer boundary: exit 1, empty stdout, exact stderr and no output
artifact. The type is `@n:Nat -> Nat` for the first two and
`@x:U32 -> @y:U32 -> U32` for shadowing. The refusal begins
`Error: main's type` and ends
`cannot be printed (a function, a Type, an erased or dependent field)`.
Types/checking phases are independently observed. No original runtime pass is
fabricated from wrapper success.

## Validation and reproduction

- Full gate: **969 passed, 0 failed, 49 ignored**, 118 result-bearing targets.
  Formatting, strict Clippy, all-feature build and socket-provider checks pass.
- Focused lib/new tests/normalization/demand/effects run: **285 passed, 0 failed,
  1 ignored**, at the same source as the full gate.
- Open/applicability matrix: **60 process observations**: 18 exact normalized
  outputs, six application-grid observations, six type/phase probes and 30 actual
  emitter refusals. Result lanes repeat twice; type/phase probes run once.
- Graph matrix: **16 observations**, four sources × two implementations × two
  repeats. Three are newly qualified; `partial_app_snf` is a regression renewal.
- Closed matrix: **54 observations**, 36 exact outputs and 18 generation/build
  successes. External C builds use the pinned O3 profile. The persistent C test
  uses strict warnings without an explicit optimization flag; profiles are distinct.
- All **1,302 checking statuses and streams** match the preceding port receipt:
  439 expected negatives reject; other fixtures are 366 accepted / 497 rejected.
  This audit is not universal exact upstream diagnostic parity.
- Fourteen positive controls pass; 18 observations retain three hole/TODO
  differences. The 49 ignores remain unrun: 47 GPU hardware and two optional
  profilers. No timeout or result nondeterminism appears in final comparisons.

The earlier shift proofs remain strict-proof `IO` exclusions. Their prior
upstream/port executable-emission successes remain historical evidence only;
no shift program ran here, and this batch adds no shift runtime credit. No public
API changes are made; earlier external-consumer compatibility caveats remain open.

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test kernel_function_values --test compiler_function_values --test kernel_normalization --test executable_demand --test runtime_effects --quiet
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

Toolchains remain Rust 1.96.0, formatting nightly 2026-05-24, Node 24.19,
Clang 19.1.7 and PowerShell 7.5.4. Host Rust debug overrides control build storage,
not Bend demand policy, safety limits or the C optimization profile. Bounds are
30 seconds per external comparison, 60 per audit child/private probe build,
20 per generated child in the persistent test, and 1,200 per gate.

The first focused run at `c0ef926f9461efcbf5a00c9215f51ddce843e59e` failed one new
test because it passed a returned function's arguments to zero-parameter `main`
through the checked API. The test correctly received `main needs 0 arguments;
got 1`. Typed Bend adapters corrected the harness at the tested source above;
the public API and its arity guard are unchanged. The failed receipt is retained,
and tests not reached by that failed run are not counted as executed there.

The gate took **319.61 seconds**, starting at
**11.001 GiB** free and reaching
**10.251 GiB**. The build preflight remains 11 GiB, reserve
10 GiB, emergency workspace floor 2 GiB and temporary floor 1 GiB. Only inactive
rebuildable test executables were reclaimed after completed runs; source and
historical evidence remain intact. Integration-cache retirement removed 114
executables but stopped at its 11 GiB postcondition (10.994 GiB free). Removing
one verified inactive unit-test executable restored 11.012 GiB before external
comparisons. The failed postcondition and completed actions have separate receipts.
The captured binary is retained as a losslessly verified compressed snapshot. Final headroom is **11.009 GiB**;
preflight again before another build.

[Compact lane evidence](cpu-function-values-evidence.csv) records source and
receipt hashes. Full commands, outputs, exit codes and build identities remain
in private receipts, without copying host paths or raw diagnostics into this report.

## Remaining coverage

Source `19b35fa85c6e6a510df7863f0a6991c7c4bfc3f8` qualifies the unchanged `eval/eta_expand_print`,
`eval/toplevel_ref_print` and `eval/hoas_binder_refresh` originals. Exact normalized
structure distinguishes the eta-expanded lambda/application from bare `id`, while
the shadowed result refers to the inner binder. Seventeen bounded applications
preserve complete values. No production defect or implementation change was needed.
[Report](cpu-function-values.md).

Clean-source gate: **969 passed, zero failed, 49 ignored**; focused controls:
**285 passed, zero failed, one ignored**. The open matrix records 60 observations,
the graph matrix 16 (including renewed `partial_app_snf`), and the closed C/JS/
interpretation matrix 54. Native executable observations have separate persistent
tests. The function-valued originals refuse actual executable printing exactly;
wrapper execution is not an original runtime pass. Named-call reconstruction,
structural/work/depth guards, demand and cancellation remain under regression.

All 1,302 checking statuses and streams match the preceding port receipt; all
439 expected negatives reject. Fourteen positive controls pass; 18 observations
retain the three unwaived hole/TODO cases. Strict `IO` proof exclusions remain
separate from executable checking. The previous shift-program evidence remains
checking/emission only; those programs were not executed in this checkpoint.

Detailed receipts link **329 originals**; **973** lack detailed receipts:
**358 checker negatives, one checking-only module, 68 interpreter cases and 546
compiled candidates**. Three interpretation rows are added; earlier 141 interpretation,
93 runtime and 92 checking rows retain their actual identities. Linked rows include
partial/blocked cells and do not establish CPU parity. Imported-printer/JS-timeout,
erased-hole/TODO, host/resource/foreign/F32.read/error-order/API gaps and the
unfinished 688-vector exploration remain unwaived. Dependencies, references,
toolchains, budgets, guards and approved demand policy remain frozen. No installs,
pushes, desktop or GPU work.

Next source-reviewed batch: `eval/printer_fixpoint`, `eval/printer_fresh_binder`
and `eval/printer_fresh_binder_pick`, covering a constructor continuation and
fresh binders generated by nested patterns. The middle original uses IO, so its
strict-proof exclusion, actual host applicability and executable results must
remain distinct. Only source/inventory review is complete; no execution credit
is assigned to that proposed batch.

The [acceptance criterion](cpu-parity-acceptance.md) remains unchanged. Historical
rows retain their tested commits, and linked receipts are not universal parity.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `6c8064d0d0025fc00b2d659adb88ac5f29efe995068a9723b50b6356406c2a39` |
| Frozen Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| snapshot.json | `71584edc0c7aeb8f8f24af3d5cdaee4e6fa75b780fce64ec64ce53c22ea3cd96` |
| snapshot/teamy-bend.gz | `383d8f508c33bc0ecb0e7c12d6255fc6c5760ed4c25647908e927627894a0ddf` |
| full-gate-final.json | `1d008b3af0085ccde759b0b9e2bc6df4ddc1b704d14b30a114125397c02be37c` |
| full-gate-final.log | `826c5bf763ab76896a6dd30528758f91c7bfed01c790f14118df43c8bf4c80fd` |
| focused.json | `62b0edc94617b7c6e2c98eacd4ac8830866771340aec5b24d48627d432a1a775` |
| focused-corrected.json | `c19fcdc5e3178bfa24f3756f5469503b2fe5f3158df65810533e79284f5adfa1` |
| applicability-preflight.json | `359dbfdcc63dabcfd25b30276b6d75a13320d25df552011b46fa84ec99679079` |
| graphs-matching/receipt.json | `20f668f8e1ea6d70c6a0ab5f461d61e1b759eabe5a65413b73bf3479d648abdd` |
| open-matrix/receipt.json | `de7464543d39a59a28e3f1991cfc083a17349fbbe7780fbc7d74cdb85fbbd598` |
| closed-runtime/receipt.json | `2d014462613a1dcbe0ba0b06484b7e7089fd5af0f60287f038f89e4dba5fe6b4` |
| audit/receipt.json | `455071da7d3c6ce377a03ece67e30f1cd7f52012646dfd4134514e75a56f3f81` |
| audit-delta.json | `f8f8fc5149b33ccf3017934998078346631bce456945fcdc7b509ffb30619beb` |
| controls.json | `a7e9677e56da68333ced20e43af408bf2c8f819f8806fa7ade9978f0c27ea7b6` |
| coverage-summary.json | `8ba7a9a34a538614d57ae42a045cb172d77674b96db2b8324c772f25d22bdf1b` |
| inactive-test-cache-reclaimed.json | `8b84f2e4e8746141040b3530b583f5300dae06cbd78ee0275fe376c787cea2d5` |
| final-unit-cache-reclaimed.json | `7e96b14aa6871b8563ac60fc6cbd7a053628d2604d66d6a394aa86037b03022f` |
| cache-postcondition.json | `0f2492e808c64238042787b42c6526a9a102227910c0f63a3a8e05dd85ad7bcd` |
