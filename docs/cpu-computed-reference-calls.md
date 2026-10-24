# Computed reference calls: bounded CPU checkpoint

Tested source: `8deca787667b9d01d40a865ca71fba6d527fdfdd`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The six original files are byte-for-byte copies, with attribution in the
[fixture directory](../tests/fixtures/cpu-computed-ref-calls/README.md).
This batch adds regression coverage and evidence; **no production implementation
or semantic policy changed**. CPU parity remains incomplete.

## What was compared

These fixtures pass checking and are interpreter-only under the pinned gate.
Their mains are functions. Exact normalization compares the pinned interpreter's
text against both pure CLI output and strict eval; the Rust regression observes
actual checked core and compares its structure with the parsed oracle form.
The CLI writer's core is not separately exposed or claimed as observed.

| Originals | Application inputs | Exact constructor results | Scalar observation |
| --- | --- | --- | --- |
| `_000`, `_001` | Nt depths 0, 1, 2, 3 | `S{Z{}}`, `Z{}`, `Z{}`, `Z{}` | `[1, 0, 0, 0]` |
| `_000_f`, `_001_f` | Nt depths 0, 1, 2, 3 | `T{}`, `F{}`, `F{}`, `F{}` | `[1, 0, 0, 0]` |
| `_000_step`, `_001_step` | `T{}`, `F{}` | `S{Z{}}`, `Z{}` | `[1, 0]` |

This is 20 distinct fixture/input pairs. The pinned direct-application adapter
loads and checks the unchanged book, applies constructor arguments, and uses
`term_snf`, `term_lower` and `term_show`. Two repetitions yield 40 oracle argument
results. The Rust checked evaluator asserts each expected value and actual core
structure. Typed wrappers independently exercise the same arguments.

Pure wrappers project Bl to a discriminant and W to its full depth, then return a
list. IO wrappers print that list and are asserted to have `ExecutableEntry::Io`.
This matters because pure `run_main` uses normalization, while IO invokes the
native executable runtime. Both profiles are regression-tested through native,
C and JavaScript; external comparisons add pinned C/JS and the interpreter.
Constructor projection does **not** qualify imported-name printing.

## Applicability and exact failure boundaries

Unchanged originals accept checking but C/JS generation reports exit 1, empty
stdout and `Error: a build needs import Base\n`. Port C/JS/native emitters match,
with no output artifact. Separate Base-enabled adapters return the original
function. They accept upstream checking, then all five emitters agree byte for
byte on exit 1 and the unprintable-main diagnostic. Its type names reflect the
adapter's import spelling; exact bytes are retained privately. These are emitter
refusals, not runtime failures or executable passes for the original mains.

All available concrete output lanes finish with exit 0, the exact list above plus
newline and empty stderr. Direct upstream IO under installed Node instead returns
exit 1, empty stdout and `TypeError: require is not a function\n`; it needs
`import.meta.require`. Twelve such calls remain **host-blocked**, not passes.
The six IO adapters are deliberately excluded from strict proof checking, while
accepted executable checking is established by native execution and code emission.
No Bun or replacement host was installed.

## Validation and provenance

| Check at tested source | Result |
| --- | --- |
| Standard gate | 936 passed, 0 failed, 49 ignored; 102 result-bearing targets |
| Format / strict Clippy / all-feature build / TCP-UDP provider | Passed |
| Original normalization | 36 process observations; strict checked-core regression also passes |
| Direct oracle application | 12 invocations, 40 returned values, 20 distinct vectors |
| Original / Base-function refusals | 120 invocations; exact exit/streams and artifact absence |
| Independent accepted-source phase probes | 24 |
| Pure outputs / pure strict eval | 72 / 12 exact observations |
| Native IO plus generated IO outputs | 60 exact observations |
| Direct IO oracle / strict IO proof | 12 host blocks / 6 deliberate exclusions |
| Generation and C build | 72 successful observations |
| Complete matrix | 426 process observations; two repetitions of result lanes; no timeouts or nondeterminism |
| All checking statuses | 1,302 unchanged: 439 expected negatives rejected; others 366 accepted / 497 rejected |
| Positive / hole controls | 14 pass; 18 observations preserve three unresolved disagreements |

The 49 ignores are unrun: 47 hardware tests and two optional profilers. Existing
regressions cover the guard, cancellation, false-proof and no-main boundaries;
the complete gate was renewed. New focused coverage is in
[kernel_computed_reference_calls](../tests/kernel_computed_reference_calls.rs) and
[compiler_computed_reference_calls](../tests/compiler_computed_reference_calls.rs),
three tests executed within that gate. Generic emitter failures remain covered by
the existing CLI compilation tests. No separate redundant Cargo test run is claimed.

The initial comparison used the prior `623c6bacfc82993e731c4c124306924c1d68d8eb`
binary and is retained as baseline evidence only. Initial test commit
`b062536b49a6495a04a5f52d019be4961ccea82a` had a copied fixture-name loop;
review caught it and stopped its gate after 39.87 seconds, exit 143. The log has
mixed checkout identity and supplies no gate pass. The corrected clean commit
above passed the full gate in 295.79 seconds; no test or source failure
is hidden by the interruption. Final matrix and controls use its verified binary.

## Reproduction

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$BEND_REFERENCE/bend2/main.ts" "$ORIGINAL"
"$BEND_BINARY" run "$ORIGINAL"
"$BEND_BINARY" --output-format json eval "$ORIGINAL"
node "$BEND_REFERENCE/bend2/main.ts" "$WRAPPER" -o "$WORK/up.c"
clang-19 -std=c11 -O3 "$WORK/up.c" -lpthread -lm -o "$WORK/up"
"$WORK/up" --threads 1 --gpu off
"$BEND_BINARY" compile "$WRAPPER" --executable --target c --output "$WORK/port.c"
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$WORK/port.c" -pthread -lm -ldl -o "$WORK/port"
"$WORK/port" --threads 1 --gpu off
```

The primary C matrix follows the pinned Linux [O3 profile](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253).
The Rust regression helper separately uses strict C flags with no explicit
optimization switch; its result is not relabeled O3. Host Rust debug overrides
do not alter Bend optimization or budgets. JS uses upstream `.js` generation or
port `--target javascript`, then Node. Matrix children have 30-second bounds,
audit children 60, compiled Rust-test children 20 and the gate 1,200. Exact
commands, streams, statuses, generated hashes and adapter sources are recorded
in the private receipt identified below; the public CSV keeps compact hashes.

## Storage and remaining acceptance

The initial free space was 10.001 GiB. Coalescing byte-identical inactive
historical executable caches preserved all 18 paths, SHA-256 values, modes,
ownership and modification times while reclaiming about 1.75 GiB across two
batches. Manifests record the before/after inode and link counts. This removes
redundant blocks, not unique evidence. The corrected gate began with 11.139 GiB
free and never fell below 10.434 GiB. After the gate,
98 inactive rebuildable integration-test executables were retired; source,
receipts and the exact verified lossless binary snapshot remain. Current
headroom is 11.035 GiB. Build-start preflight now
requires 11 GiB, a full GiB above the 10 GiB reserve; stop floors remain 2 GiB
workspace / 1 GiB temporary storage. Recheck before another build.

Source `8deca787667b9d01d40a865ca71fba6d527fdfdd` qualifies six unchanged `flatten/computed_ref_call_000/001`
originals and their `_f`/`_step` companions through exact normal forms and **20
bounded concrete applications**. Checked-core structure and text, pure CLI
normalization, and native IO execution are separate observations. The original
builds require Base; separate Base-enabled function adapters are unprintable.
Concrete pure/IO observation wrappers pass the available generated C/JS lanes.
No production behavior change was needed. [Report](cpu-computed-reference-calls.md).

The clean-source full gate passed **936 tests, zero failed, 49 ignored**. The
repeated matrix has **426 process observations**, including 12 direct upstream
Node IO host blocks and six strict-proof exclusions. All 1,302 checking statuses
are unchanged; 14 positive controls passed, and 18 observations preserve the
three unwaived hole/TODO disagreements. The interrupted initial gate is not a
candidate receipt. Historical source and receipt identities remain unchanged.

Detailed receipts now link **293 originals**, with **1,009** lacking detailed
receipts: **360 checker negatives, one checking-only module, 95 interpreter cases
and 553 compiled candidates**. Six interpretation rows are new; the 104 other
interpretation, 93 runtime and 90 checking rows retain historical identities.
These counts include partial/blocked cells and do not establish CPU parity.

The imported-constructor spelling disagreement and pinned JS printer timeout
remain separately open at their earlier tested source. Erased-hole/TODO,
host/resource/foreign/F32.read/runtime-error-order/API requirements and the
unfinished 688-vector exploration remain unwaived. Next independent batch:
six `stacked_computed_match_000/001` originals and `_nat`/`_not`/`_sel` companions,
with a proposed 39-vector grid for two/three computed columns. Source and
applicability were reviewed; none of that new grid was executed or qualified.
Dependencies, references, toolchains, guards, budgets, cancellation and approved
demand semantics remain frozen. No installations, pushes, desktop or GPU work.

The earlier [imported-printer evidence](cpu-ragged-rows.md#open-imported-constructor-printer-discrepancy)
remains linked to source `623c6bacfc82993e731c4c124306924c1d68d8eb`: interpreter,
upstream C and pure native normalization print `helper.Box{7}`; port C/JS print
`Box{7}`; pinned JS was killed at a three-second bound, with the larger probe
also retained at 30 seconds. This batch did not rerun or fix it and adopts no
exception. [CPU acceptance](cpu-parity-acceptance.md) still requires every
applicable lane, API and failure/resource contract, with each gap closed or an
explicitly approved exception; linked receipts and a passing gate are insufficient.

| Private evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `14fe9f4d4efb4f56db51e153014cf7e090e9d74a7170b271fdcb8a7407af16d5` |
| snapshot.json | `8fd983da4dd89100275963349472594311d3542053a486551b8e1497e3935a7d` |
| snapshot/teamy-bend.gz | `19bbb5562e83284399309ef0fa5954cdbc4946ef6d9c2debea4bc74f6b69d921` |
| full-gate-corrected.json | `ac58a1ba0219e9ed831941761fd6fbb8d2ed4acd7a0e85ef864f4ab214bb0991` |
| full-gate-corrected.log | `efb98abc4d59d4ce3e7bbf1440b7762b3f0400597a73b4366db7648e28b00648` |
| qualified/receipt.json | `aad3fe1e9f825691c7e370679fef389ce6fd6330832a4b1b4e00ebd016f9accb` |
| audit/receipt.json | `bd10bde63533bb602a48bf888e1cb0aedd0ebf6fc4cd74dcd27caf70d4110840` |
| controls.json | `a908594883a7b544147642cfda7393524c4a0ef1408d386b2ca411273fc44b4e` |
| coverage-summary.json | `58592de212810d1ad935cb6b0308368d02bb2d5b5992b42685bc4384785b7338` |
| duplicate-cache-coalesced.json | `312d2b2fdadfb7516eb41a90f397cbcfae29a2f2dbad750efa04e1ef63d63513` |
| additional-cache-coalesced.json | `d7ec132b8b7aa41189e9123ac3a3faa9b74a9ed8449e176477add27367b118bd` |
| inactive-test-cache-reclaimed.json | `23faa073f1a738e3238553f05d037cd5a7d7ec057df79db60bfc8a05282fc5e3` |
| qualify.py | `77ba3b7a67748dd73cee6ff8bc316c4678cd009fe115beaef9a71922c229c0f1` |
