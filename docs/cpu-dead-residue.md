# Dead-residue matching and standalone build qualification

Tested source: `dcf26ff64a9c093cd5a5b123b3b5c6df599e5bc5`. Starting checkpoint:
`0d78280cdf4547a0c90615037438f46d3c53701f`. Target:
[pinned Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The final evidence checkpoint changes only repository documentation and ledgers
after this tested source; its commit identity is in the handoff receipt.

## Applicability and correction

The four [originals](../tests/fixtures/cpu-dead-residue) are byte-identical,
Apache-2.0 copies of pinned `tests/flatten/dead_residue_elision_000`, `_001`,
`_002` and `_004`. Derived oracles retain attribution; new adapters, wrappers
and tests use MPL-2.0. `_000` has a function main; the other three have concrete
results. All four lack Base and top-level IO. Upstream C/JS refuse them with
exit 1, empty stdout and exactly `Error: a build needs import Base\n`.

At baseline source `eeb7b2814ae9fa21cf68b8c0c06cf236c419b75b`, the port incorrectly
accepted all three concrete mains in C, JS and native-target compilation:
**nine acceptance mismatches**. The full 64-observation baseline and generated
counterexamples are retained separately from an initial partial collector and
an artifact-collision collector failure. The correction is not a matching fix
or a new evaluation rule.

Pinned [`show_main`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
tests whether the exact top-level `IO` declaration exists. It does not require a
particular import spelling, declaration kind or Base provenance. Both
[`compile_book`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2820)
and [`js_book`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L3138)
apply this gate before lowering. The port now uses the same exact-name presence
test through a shared structured CompileError preflight. It performs no term
evaluation or mutation and grants no Base/foreign capability. Existing missing-main
Display/equality behavior remains intact; strict compilers and pure interpretation
still accept appropriate no-Base programs.

Base-importing adapters are separate sources. The `_000` function adapter reaches
the exact function-printer refusal; the three concrete adapters generate artifacts.
The renewed collection includes 50 exact refusals without artifacts, 30 successful
artifact generations, six auxiliary native-target C executions and separate
normalization, application, type and checking-phase observations. Native-target
compilation packages generated C; it is not the native Rust interpreter.

[Boundary regressions](../tests/compiler_dead_residue.rs) cover missing and
qualified-only IO, unprintable Type precedence, ordinary IO definitions/ADTs,
existing-file preservation under `--force`, absent provider artifacts and source
foreign refusal. Existing custom Nat/U32 shadow tests retain their original
checked meanings and exact constructor output, using an ordinary IO declaration
without importing Base. Independent review of `dcf26ff64a9c093cd5a5b123b3b5c6df599e5bc5` found no blocking
production or structural-comparison concern; validation is recorded separately.

API compatibility: `compile_executable_c` and `compile_executable_javascript`
can now return Err for inputs previously returning source, and the early preflight
can change failure precedence. Repository production callers in CompileArgs
propagate the structured rejection to status-1 CliOutput before output/provider
creation; the entry point consumes exit_code. **External consumers were not
audited.** No public type or signature changed in this batch.

## Bounded behavior and complete structure

| Original/worker | Inputs | Behavior covered |
| --- | ---: | --- |
| `_000.main` | 3 enum values | All constructors covered; unreachable default residue retained consistently in normal form |
| `_001.f` | 9 pairs from Nat depths 0, 1, 3 | Later-column partial row and complete constructor coverage |
| `_002.f` | 9 pairs from Nat depths 0, 1, 3 | Default row reconstructs the entire first matched value |
| `_004.f` | 8 state/counter pairs; depths 0, 1, 2, 4 | Leading zero-counter catch-all retains priority in either state |

[Kernel tests](../tests/kernel_dead_residue.rs) compare all 29 direct applications
with complete constructor text and structure. Four original mains, three worker
functions, four scalar wrappers, four full imported-value lists and three concrete
adapters have full graph/text checks. Alpha-canonical graphs preserve all nodes,
ordered fields/arms and binding relationships; they are not physical Rc identity.
The prior explicit-stack constructor JSON helper retains complete shape and
escaping/deep-output/rejection controls without changing Bend evaluation or guards.

The eleven executable mains run through native Rust Pure, strict C and Node JS
in repository tests. External comparisons separately repeat upstream interpretation,
port pure CLI, upstream C O3, port strict C O3, upstream JS and port JS. All four
scalar wrappers and the nullary-result `_000` full-value list match all six lanes.
The other six full-value/adapter sources match every available completing lane.

## Results and unresolved observations

| Check | Result |
| --- | --- |
| Full standard gate | **1,018 passed / 0 failed / 49 ignored**, 495.341 seconds |
| Clean focused gate | **83 / 0 / 0**, 41.582 seconds |
| Normalization/applicability | 132 observations; exact refusal and successful-generation categories kept separate |
| Complete graphs | 72 matching observations, two implementations twice |
| Runtime matrix | 120 exact executions; 66 successful generation/build steps; 12 upstream-JS timeouts |
| Checking audit | 1,302 unchanged statuses and streams; 439 expected negatives reject |
| Additional controls | 14 positive passes; 18 observations preserve three hole/TODO disagreements |

The six upstream-JS field-bearing imported-value printers each time out twice at
the diagnostic three-second bound, with empty streams. Generated values use local
field-bearing tags while descriptors name qualified constructors, reproducing the
[imported-printer discrepancy](cpu-imported-printer.md). Nullary `_000` results use
matching tags and finish. All seven generated controls, including failing programs,
are retained with hashes. Only owned timeout processes are killed and reaped;
that is not an upstream language exit status. The runtime runner exits **1**,
`full_required_matrix_passed=false`; no oracle exception or full parity is claimed.

Ordinary IO-only scaffolds pass the upstream declaration gate and JS generation.
Upstream C then fails because its hard-coded runtime ADTs are absent, with
`TypeError: Cannot read properties of undefined (reading 'c')`. This is an
unresolved upstream compiler failure, not a missing installed host dependency.
Port scaffold execution tests pass, but they do not establish full upstream C
or JS runtime parity for these scaffolds.

The synthetic `def main() -> Tag: import "missing.js"` is rejected during checking
by both implementations, before the emitter. The pinned
[foreign checker](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3714)
requires actual Base IO directly. Upstream prints expected/observed/source context;
the port retains the host-context diagnostic. Status/phase agreement is recorded,
**exact diagnostic parity remains open**. Neither a new foreign policy nor an
exception was introduced.

Initial failed receipts remain explicit. The focused run at `9bc6a099` exits 101
because a new test incorrectly expected that foreign source to reach emission;
the corrected test at the tested source checks the existing refusal. Independent
review also caught a missing ordinary-IO scaffold in the JS custom-Nat test,
fixed before the clean focused/full runs. An external boundary collector repeated
the mistaken upstream-emitter expectation; its partial receipt is retained and
the four renewed foreign observations record actual checking failures. A separate
open collector compared successful CLI reports for different requested output
paths. Renewal uses identical paths for both repetitions, retaining exact raw
stream comparisons; the failed receipt and artifact remain intact.

Aggregate sharing/work/native/C/JS/demand/cancellation regressions pass in the
full gate. The 23/24/27-result work counts remain **634,793 / 861,655 / 1,338,629**
under the unchanged 2,000,000-step and 128-depth guards. Cancellation remains at
existing before/after-normalization/output boundaries; no mid-normalization
interruption or general resource equivalence is claimed. The external 72-observation
aggregate receipt remains historical at `936d299`, not renewed by this task.
Formatting, strict Clippy, all-feature build and Node provider checks pass.
The 49 ignored checks are 47 GPU hardware tests and two optional profilers.

## Commands, identities and storage

After activating the existing tools:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_dead_residue --test compiler_dead_residue --test compiler_executable_c --test compiler_executable_javascript --test cli_compile_executable --test compiler_imported_printer --test executable_demand --test runtime_effects --quiet
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug overrides do not change Bend runtime optimization. Matrix commands
use the unchanged originals or explicitly identified adapters/wrappers:

```sh
node "$BEND_REFERENCE/bend2/main.ts" "$fixture"
teamy-bend run "$fixture"
teamy-bend --output-format json eval "$fixture"
node "$BEND_REFERENCE/bend2/main.ts" "$fixture" -o upstream.c
teamy-bend compile "$fixture" --executable --target c --output port.c
clang-19 -std=c11 -O3 upstream.c -lpthread -lm -o upstream
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic port.c -pthread -lm -ldl -o port
./upstream --threads 1 --gpu off
./port --threads 1 --gpu off
node "$BEND_REFERENCE/bend2/main.ts" "$fixture" -o upstream.js
teamy-bend compile "$fixture" --executable --target javascript --output port.js
node upstream.js
node port.js
teamy-bend compile "$fixture" --executable --target native --output native-adapter
./native-adapter --threads 1 --gpu off
teamy-bend --output-format json check "$fixture"
```

Primary external C qualification uses O3. Auxiliary CLI-native-target packaging
retains the port's O2 profile and is a separate observation. Exact argument arrays,
raw streams, exit statuses, hashes, cleanup and timing are retained privately.
Two execution repetitions; ordinary matrix deadline 30 seconds, diagnostic JS
printer deadline three seconds, checking child deadline 60 seconds, outer bound
1,200 seconds. Tools remain Rust 1.96.0, fmt nightly-2026-05-24, Node 24.19.0,
Clang 19.1.7 and PowerShell 7.5.4. Docs pin remains
`50e89d6897ce2a65167ef65b17c5fa83fe849158`.
Cargo.lock SHA-256: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured tested binary SHA-256: `220e02a46da8765b212d55dd8dd614c369fdacbca67b4013295ab6e2ed13e9f9`.

Prebuild free-space requirement is 11 GiB, one GiB above the 10 GiB stop floor.
The full gate started with 11.089 GiB and reached a minimum
of **10.086 GiB**. Completed inactive first-party build caches
were reclaimed only after the exact current executable was captured and verified.
The matching comparison library was reclaimed after graph comparison completed.
Source, dependency caches and unique historical failure evidence remain intact.

Storage inspection identified historical executable captures as the dominant
evidence cost. One inactive 140,360,792-byte historical array-read executable,
shared through five hard links, was losslessly archived to 36,433,249 bytes with
original paths/modes/owner/time/link metadata. Every restored member was hash
verified before unlinking raw copies, reclaiming 103,927,808 bytes. Those five old
raw paths now require restoring the retained archive; the private archive receipt
contains the exact command and original SHA. No unique bytes or failure logs
were discarded. Compact graph receipts retain every node and observation.
Final comparison headroom is 11.103 GiB.

## Coverage and next work

Tested source `dcf26ff64a9c093cd5a5b123b3b5c6df599e5bc5` adds four unchanged `flatten/dead_residue_elision`
originals (`_000`, `_001`, `_002`, `_004`) and **29 bounded applications**.
Applicability exposed nine incorrect standalone-build acceptances: the three
concrete originals compiled in port C/JS/native despite lacking the exact
top-level `IO` declaration required by pinned upstream. A shared, independently
reviewed preflight now rejects them before lowering, with exact status/stderr
and artifact preservation. Pure interpretation, strict emitters, evaluation,
sharing and safety limits are unchanged. [Details and receipts](cpu-dead-residue.md).

Full gate: **1,018 passed, zero failures, 49 ignored**. Focused controls:
**83 / 0 / 0**. All **72 complete graph** and **132 normalization/applicability**
observations complete. The runtime matrix records **120 exact executions,
66 successful generation/build steps and 12 unresolved upstream-JS printer
timeouts**. All port-required cases pass; the complete matrix exits 1. Full
values agree in native Rust, port C/JS, pinned interpretation and upstream C O3;
scalar projection success does not qualify the failed upstream-JS full printers.

Aggregate sharing/work/demand/cancellation regressions pass in the full gate;
no mid-normalization cancellation is claimed. All 1,302 checking statuses and
streams remain unchanged, including 439 rejecting negatives. Fourteen positive
controls pass; 18 observations retain three hole/TODO disagreements. Minimal
ordinary-IO scaffold C compiler failures and a foreign-source checking diagnostic
difference remain separately unqualified. External emitter consumers can now
receive Err for no-IO builds; repository callers handle status 1, but external
consumers have not been audited.

Detailed receipts link **358 originals**, leaving **944 unlinked**: **357 checker
negatives, one checking-only module, 49 interpreter cases and 537 compiled
candidates**. These are applicability categories, not missing runtime-test
counts. Links include historical, partial and blocked evidence, not completed
parity. Imported-JS printing, erased-hole/TODO, resource/host/foreign gaps,
F32.read, competing failure order, downstream API caveats and the unfinished
688-vector exploration remain unwaived.

Next proposed independent batch: `flatten/nested_computed_match_001`, `_dec` and
`_zro`; establish actual printability, then **12** bounded applications. This is
source/ledger selection only, without execution credit. Dependencies, toolchains,
references, budgets, guards and cancellation remain frozen. Local checkpoints
only; no installs, pushes, merges, desktop or GPU work. No user decision is
required by this correction.

## Receipt identities

The [execution ledger](cpu-dead-residue-evidence.csv) records 402 observations
without machine paths. Four interpretation links are appended without modifying
historical rows. Private receipts retain commands, status/streams, hashes,
timeouts, source identities and the failed exploratory collectors.

| Receipt | SHA-256 |
| --- | --- |
| `preflight.json` | `867d5732300382edb02f17cd57798c376a58b41faf748d603e9d500e56e552f9` |
| `applicability-baseline/baseline-complete.json` | `620cf60115b6aec773971ac3edc4bf8df198b9f03d8e7fb7256fdc634f46d3e2` |
| `application-preflight.json` | `d5fcd2a2af725b8db66e1dbe04659b262a1da194e015ecf61f2f131a3cc217e4` |
| `focused-renewed.json` | `f774350cdce203b826caeeed40679138f8e5e7d02d04bb85ae0218bd63ffa6ff` |
| `focused-final.json` | `f94d998f569d8dcdbc5c17757dbb4f80a05709d66181b253fd7a17dec36ca7af` |
| `focused-final.log` | `fd685e4f969c687985f86a77145cf146bb1dc64b49ce157c87a233fda2b74932` |
| `full-gate-final.json` | `02b3e843673f95d86129e560bb1637e986fcc0df86ad8d287c81f69ee97565fc` |
| `full-gate-final.log` | `eaf82f490d3d3a398892a7c82c550c3f40ca04e3b9f22b402cbd4ab23bd76acc` |
| `snapshot.json` | `9fcb297e2e2c040fce6d6f55a54c11944d46b07a95f370791b3a16f7352419ad` |
| `independent-review.md` | `701216047739e7e6f13e3861005b403ac429e6896bffa6f28939536bbb402272` |
| `final-evidence-review.md` | `02ca04447f0560799c49b039ba66344835ac87569f548364b5480816ad06f62e` |
| `caller-review.json` | `e54b426d524a30729e967032a7bf0a4756d1c8ed33e17c1f2548a9c34047b09c` |
| `open-matrix/receipt.json` | `ed2b3662df3c11a340eb63a97f9d42663261d102c5a5b8665952926a6f827d43` |
| `open-collector-note.json` | `229094085f4ca5f10417af5577f21a3c960aaede6a8d2ab85f34623760a77254` |
| `open-matrix-final/receipt.json` | `88d0d3a4dfb081eadb8b8b434e809046fe5336f1e9020b47d4eb9121933a6c54` |
| `graphs-matching/receipt.json` | `3e42aa2c625fe7edf861995be03cae26cbbb860361d343f1fb5a733ac30cbe54` |
| `matrix/receipt.json` | `04eb294fccbff71f507a821b637d6ba78d3db90ae1ecff9e4fb09f438d67196a` |
| `matrix-run.json` | `65b46140609c17622b8ecaa1d499798152ab2c02e1156ffd2900b9f6212dcd27` |
| `upstream-js-evidence.json` | `7626e9e80aa0a679a1a070f5fbf328941a466b025feeae71bf4b62256ff49c0e` |
| `gate-control-upstream.json` | `2dd25a477be85f3d1ac87281d6e4f3ceb49115062c27975a4b3168d54a7ebd5c` |
| `declaration-controls-initial.json` | `4ca933766fb8c7affa0cba98d36a2fec79141fc8ee4ef5100c379ff9e5bf37ca` |
| `declarations-run.json` | `93b95be0d2a06bd51e3e47407328356112327c1cd48300138147d6707b7e64c1` |
| `foreign-phase-controls.json` | `58b68e12927cee158c729190e37092fb07771753d387b564bef1c6cebd036d75` |
| `audit/receipt.json` | `0281844ae03d819036854592a8a32c9b4bb212f83f7621ecc250163812aaad53` |
| `audit-delta.json` | `71d2aca4119707c43595a872695dc6204e066b2e38e0ed71e14da554246d7a29` |
| `controls.json` | `400ee37df44bf627a7c61ff3576dff3d16794d578665557440ab3c128d82e00a` |
| `coverage-summary.json` | `4f7926f665798599657ac0222a5913d0f513eafc55276149fe0d54eae947c76c` |
| `tool-versions.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `comparison-preflight.json` | `e8c1f7bb054a934db6d7fcee1f194ed3409714b557ac1f9fb32bd97aaa332ccc` |
| `historical-capture-archive.json` | `60103528066f3c49f507002ce5906ae0ef7a056112995a7e30dd4595efccca41` |
| `comparison-library-reclaimed.json` | `026b97f32da05ea4232b3bc43ed97441416538b49b0b19156f38eaee131df306` |
| `next-gap.json` | `6f254f42d3d0fae7f335a632d850bff652b02acc5768452ce2bd1472f4b2e02a` |
