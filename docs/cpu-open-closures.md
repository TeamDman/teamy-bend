# Open closures, computed calls and function-domain printing

Tested source: `cbfe84fb6f76c17fcb3022fc86d3e62f01a3cafa`. Upstream remains
[pinned Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The [three originals](../tests/fixtures/cpu-open-closures/README.md) retain upstream
bytes and Apache-2.0 attribution. New tests and wrappers use MPL-2.0.

## Demonstrated presentation defect

The port omitted parentheses around function-valued domains in the refusal for
`open_closure_pipe`. Upstream requires this type:

```text
@f:(@_:U32 -> U32) -> @g:(@_:U32 -> U32) -> @_:U32 -> U32
```

The old string was `@f:@_:U32 -> U32 -> @g:@_:U32 -> U32 -> @_:U32 -> U32`,
which loses the domain grouping. The pinned
[All printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1338)
prints domains at precedence 2 and parenthesizes an All term above precedence 1.
The production fix is one condition in [term.rs](../src/kernel/term.rs): the
existing precedence-2 operand formatter now also groups `Term::All`. Codomains
remain right-associated. It does not normalize, substitute, rebuild or mutate
terms, affect sharing, or change checking, demand, cancellation or work accounting.

Six fully bound syntax-printer cases cover nested domains, a nested function
codomain, dependent outer type references, erased/reusable quantities and an
ordinary arrow chain. These are syntax/structure controls, not fabricated checked
proofs. Their local comparator preserves quantities, forall domain/body structure
and bound-variable distances. The shared normal-form comparator is unchanged.
An initial attempt to extend it failed two existing refusal tests; the final
candidate restores those tests and keeps the new vocabulary local. The original
failed source `8728dbd3ddb1350bffc25a73fbad30757fa9a1aa` and its receipt are retained.

The [CLI regression](../tests/cli_compile_executable.rs) checks all three port
targets: exit 1, empty stdout, exact pinned stderr and no output artifact. The
external matrix independently renews actual upstream/port printer outcomes.
No public signature or result-layout change is made; consumers of textual types
will see the required parentheses. Earlier downstream API caveats remain open.

## Original applicability, graphs and binding

| Original / observed definition | Exact pinned and port normalized text | Other contracts |
| --- | --- | --- |
| `open_closure` / `pipe` | `f => g => x => g(f(x))` | IO main executes; strict proof book rejects IO |
| `open_closure_pipe` / `main` | `f => g => x => g(f(x))` | Strict normalization; function executable printers refuse |
| `stuck_split_mint` / `main` | `n => probe.at(dbl(n))` | Strict normalization; function executable printers refuse |

[Kernel regressions](../tests/kernel_open_closures.rs) compare exact text and
Facet-decoded graphs separately. Composition has three distinct binder IDs;
the outer application references `g`, the inner one references `f`, and its
argument references `x`. The blocked call retains the `probe.at` and `dbl`
references with the original bound argument. Parsed-text round trips preserve
structure. No printer substitution is used to hide a normalization difference.

The IO helper is compared inside the existing private execution-normalization
test module, with the existing engine reset and budgets. Strict checking and
`eval --entry pipe` still reject `main: undefined name IO`; no proof API or
executable-to-proof conversion is added. The external paired graph matrix uses
the two pure originals and `partial_app_snf`. The private IO comparison is not
relabeled as an external public proof call. All ten named-call regression cases
and their sharing, guard and cancellation controls remain in the full gate.

The unchanged IO original prints exactly `12\n`, exit 0 and empty stderr,
in native execution and upstream/port generated C and JS. Direct upstream Node
execution still exits 1 with empty stdout and
`TypeError: require is not a function\n`, from the pinned
[import.meta.require runner](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1274).
No Bun installation or host workaround is attempted. The two function originals
refuse executable printing with matching status/streams and no artifact.

## Closed applications and the open aggregate discrepancy

Each composition helper receives identity, increment and double in both function
positions, with U32 inputs 0, 5 and 4294967295: **27 individual calls per helper**.
The noncommuting pair distinguishes `double(increment(5)) = 12` from
`increment(double(5)) = 11`; boundary inputs check U32 wrapping. Five Nat inputs
0, 1, 2, 7 and 16 applied to the blocked-call function produce complete Bool values
`True{}`, `False{}`, `False{}`, `False{}`, `False{}`. Native kernel assertions
compare complete results separately from the executable wrapper's U32 projection.
All **59 individual calls** agree with pinned observations. Each call receives
the existing fixed budget; this does not grant their aggregate a larger budget.

The two unchanged [27-result wrappers](../tests/fixtures/cpu-open-closures/wrappers)
produce the expected full list under upstream interpretation and upstream/port
generated C and JS. The port's pure CLI/driver normalizes that entire list under
one budget and rejects both with `kernel evaluation budget exhausted; proof not
accepted`, exit 1 and empty stdout. These four repeated observations remain
**resource compatibility failures**, not passing output matches. The private
runner completes collection but exits 1. Native IO execution, pure normalization
and generated execution are separate lanes; no alternate evaluator closes the gap.
The persistent compiler test qualifies generated aggregates and the original IO
and small recursive wrapper natively; it does not claim native aggregate success.

Bounded prefix diagnostics preserve the full 27-result source and its hashes.
They are diagnosis only; a shorter prefix cannot qualify the full wrapper. The
23-result prefix passes and the 24-result prefix fails; the added result is
`increment(double(4294967295)) = 4294967295`, which passes individually:

| Result count | Lane | Exit | Outcome |
| --- | --- | --- | --- |
| 9 | port | 0 | pass |
| 9 | upstream | 0 | pass |
| 18 | port | 0 | pass |
| 18 | upstream | 0 | pass |
| 21 | port | 0 | pass |
| 21 | upstream | 0 | pass |
| 22 | port | 0 | pass |
| 22 | upstream | 0 | pass |
| 23 | port | 0 | pass |
| 23 | upstream | 0 | pass |
| 24 | port | 1 | resource_gap |
| 24 | upstream | 0 | pass |
| 27 | port | 1 | resource_gap |
| 27 | upstream | 0 | pass |

Preliminary isolated controls also preserve increment-at-max, double-after-double
at max and the noncommuting 5 input. They complete on the preceding source with
0, 4294967292 and 12 respectively. Current-source individual tests renew those
results. This locates the failure in aggregate work, but does not prove that all
accounting or repeated reduction is unavoidable. That question remains next work;
limits are unchanged and this is not a new accepted policy exception.

## Validation and provenance

- Full gate: **983 passed, zero failed, 49 ignored**, **124 result-bearing
  targets**, **377.57 seconds**. Format, strict Clippy, all-feature build
  and real socket-provider checks pass.
- Final focused run: **300 passed, zero failed, one ignored**. Initial failed
  source: **280 passed, two failed, one ignored**; remaining targets were unrun.
  The initial failures were the shared comparison-vocabulary expansion, now reverted.
- Open/applicability: **54 observations**. Twelve pure normal-form texts, 20 exact
  printer refusals, six individual-application probes, six type/phase probes,
  four strict IO exclusions, two IO-helper texts, two pinned IO-helper graphs
  and two syntax-printer probes. Result lanes repeat twice.
- Paired graph matrix: **12 exact observations**, including `partial_app_snf`.
- Execution collection: **72 observations**, **42 exact outputs**, **24 builds/
  generations**, **four resource failures**, **two host blocks**; runner exits 1.
  No timeout or observed result nondeterminism in the final matrices.
- All **1,302 checking statuses** are unchanged: 439 expected negatives reject;
  other cases remain 366 accepted / 497 rejected. **One** checking stream changed;
  the affected diagnostics have **2** renewed exact upstream
  observations. The changed original, `check/capture_soundness.bend`, also has two
  repeated port checks and two upstream checking-phase refusals. Its false
  equality is still rejected; only function-type parentheses change. Any other
  nonexact diagnostic remains unqualified. This is not a claim
  of universal upstream checking or error-text parity.
- Fourteen positive controls pass; 18 observations retain three hole/TODO cases.
  Private-request wildcard refusal and the approved upstream C exception are
  unchanged. The 49 ignores remain unrun: 47 GPU hardware, two optional profilers.

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test kernel_open_closures --test compiler_open_closures --test cli_compile_executable --test kernel_wildcard_values --test kernel_normalization --test executable_demand --test runtime_effects --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$BEND_REFERENCE/bend2/main.ts" "$ORIGINAL"
"$BEND_BINARY" run "$ORIGINAL"
"$BEND_BINARY" --output-format json eval "$PURE_ORIGINAL"
node "$BEND_REFERENCE/bend2/main.ts" "$WRAPPER" -o "$WORK/up.c"
clang-19 -std=c11 -O3 "$WORK/up.c" -lpthread -lm -o "$WORK/up"
"$WORK/up" --threads 1 --gpu off
"$BEND_BINARY" compile "$WRAPPER" --executable --target c --output "$WORK/port.c"
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$WORK/port.c" -pthread -lm -ldl -o "$WORK/port"
"$WORK/port" --threads 1 --gpu off
```

Rust 1.96.0, formatter nightly 2026-05-24, Node 24.19, Clang 19.1.7 and PowerShell
7.5.4 remain frozen. Rust debug/incremental overrides affect host build storage,
not Bend budgets or C optimization. External C uses pinned O3; persistent C tests
retain strict warnings without an explicit optimization flag. Deadlines remain
30 seconds per comparison, 60 per audit child/private probe build, 20 per persistent
generated child and 1,200 per gate. Exact commands, hashes, streams, statuses and
exceptions are retained privately; the [compact ledger](cpu-open-closures-evidence.csv)
contains no private paths or raw audit diagnostics.

The build preflight remains 11 GiB; the reserve remains 10 GiB. Inactive focused
executables were reclaimed before each build; the gate minimum was
**10.164 GiB**. Source and unique historical evidence remain,
including the failed focus and aggregate probes. Only inactive rebuildable Cargo
executables and two alternate-feature metadata caches were removed; the candidate binary has a verified lossless gzip copy.
Final free space is **11.007 GiB**. Emergency workspace/
temporary floors remain 2/1 GiB and were not reached. No new work is pushed.

## Remaining acceptance

Source `cbfe84fb6f76c17fcb3022fc86d3e62f01a3cafa` qualifies the unchanged `eval/open_closure`,
`eval/open_closure_pipe` and `eval/stuck_split_mint` originals under separate
normalization, printer and IO contracts. Function-domain diagnostics now retain
the pinned parentheses. Graphs/text and binder identities agree; 54 individual
composition calls and five recursive Nat applications preserve complete results.
The IO original prints `12` in native and upstream/port generated C and JS.
Its direct upstream Node lane remains host-blocked and strict proof checking
still excludes IO. [Report](cpu-open-closures.md).

Full gate: **983 passed, zero failed, 49 ignored**; focused controls:
**300 passed, zero failed, one ignored**. The 54-observation open matrix and
12-observation graph matrix pass, including renewed `partial_app_snf`. The
72-observation execution collection has **42 exact outputs, 24 generation/build
successes, four aggregate work-budget failures and two direct-Node host blocks**.
It exits 1; it is not a passing full runtime matrix. Both unchanged authored
27-result composition wrappers exhaust the single pure-normalization budget,
although individual calls and generated C/JS aggregates agree. This new required
resource gap remains open; no evaluator switch, larger budget or policy exception
is used to qualify it.

All 1,302 checking statuses are unchanged; all 439 expected negatives reject.
Only `check/capture_soundness.bend` changed its checking stream. It now matches
exact repeated upstream status/streams and rejects during checking. Phase-only
probes remain separate. Fourteen positive controls pass;
18 observations retain three unwaived hole/TODO differences. Existing source-hole,
sharing, named-call, demand, cancellation and work/depth/structural controls pass.
The initial test-comparator expansion broke two refusal controls; it was reverted
and a type-only comparison was kept local. The failed receipt remains historical.

Detailed receipts link **339 originals**; **963** lack detailed receipts:
**357 checker negatives, one checking-only module, 68 interpreter cases and 537
compiled candidates**. Two interpretation rows, one original-IO runtime row and
one exact checking row are added; earlier 149 interpretation, 94 runtime and
92 checking rows retain source identities. Linked evidence includes partial/blocked cells, not universal
CPU parity. Aggregate work, imported-printer/JS-timeout, erased-hole/TODO,
host/resource/foreign/F32.read/error-order/API gaps and the incomplete 688-vector
exploration remain unwaived.

Next bounded work: account for the observed **23-result pass / 24-result budget
failure** boundary, separating genuine accumulated work from avoidable repeated
reduction. Preserve the exact 27-result reproducers, individual and generated
results, fixed budgets and guards. The printer correction is independent of this
resource investigation. Frozen dependencies, toolchains and reference pins;
no installations, desktop, GPU work or new pushes. Previously published branch
heads remain at the earlier approved checkpoints.

The [acceptance criterion](cpu-parity-acceptance.md) remains unchanged.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `e7f6c9be58b0cb7162402a15d98f835ab2d5d77893ef847801fc038f4715b092` |
| Frozen Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| snapshot.json | `9c4c70eb956ae10ae9b683710a0af5257dadb715b1cf38d924a00024185c12c9` |
| snapshot/teamy-bend.gz | `d0ec0ffcdd36c6885829af858fa610009beb3a18dfb1f2b2ea80345794fac774` |
| focused.json | `ba2bfcf7a99238defb9feea1bae9ad5d07896398b5d09c76125ef12ca7bbb3f5` |
| focused-final.json | `649e74a70727b7f0c9b45e03eaf7fe1f5dd2e627c9d6bffdc18e738d748b4983` |
| full-gate-final.json | `533a3865978df23780e2ab83dece3d7d4fe7ab93800647a082be0641553f401b` |
| full-gate-final.log | `ee9123e92518dcf51ce20ae065a8c8851c8d9499b5efb9c445dcafa0ff895d64` |
| preflight.json | `6dcee9d03ad119aedafbdf681588ab451158e4be7841bcc470bcddab9053aea6` |
| open-matrix/receipt.json | `11fd13564257833f920c5a10531f8c3c850d3a7e29cb8912519ab7aca6f6254e` |
| graphs-matching/receipt.json | `289d7b7546fdaa2a04a8c08f9fa38da135c185a72394b8d88269c7e91ab77963` |
| closed-runtime/receipt.json | `adef5367b7e53723d065467958b129733d19f10fc76d98b3bdd441532d7099db` |
| budget-prefixes/receipt.json | `43fbc2788007a39c243ed38a43874747288f8074bbb7e44c7d13756520eaae1c` |
| budget-boundary/receipt.json | `dc165c1eff2b21f614fe9b319e7807836039f1a9ab1628d404a4a05c76164844` |
| audit/receipt.json | `5c01f737cae48477a1ed81711b73d903bd36afafd8a1ff2ed7cb4375232b78ca` |
| audit-delta.json | `69a12ad87db607c3b2b9fc1f87860bc67d9df936ad4035ffb37bd13af2579a8d` |
| audit-diagnostics.json | `56eb6eec50b6c737e9eb60c3453bbd9c2960aafa5faaa6f8b46a8d27775119fd` |
| capture-soundness.json | `9870054ef7e1c6922d62785ae6218da6de831b4cfaf017fcfb388d92cff206ff` |
| controls.json | `47a2e5f95e66f37dbcffe5046b58a4f6d01aa82dcee382c4529b1f01d95aefb6` |
| coverage-summary.json | `1db4a63028c327a83ffa2270910564e7445ff31d183e3807bf82dd7760d9215a` |
| inactive-test-cache-reclaimed.json | `029626555975f7716f9782205adafba210e4f08e457524d9e8fe4933cc6bccd2` |
| alternate-library-cache-reclaimed.json | `9f39cfed4f1a5bdf24a241225ccd77c3d347ac83479eed86c9513f44a787c0dc` |
