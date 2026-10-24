# Dependent pairs, multi-column proofs and recursive rewriting

Tested source: `99ff1baa56ab31aa18fa71641255cfc769b275cf`. Starting checkpoint:
`21285874eaf53b557b60418097b7f3792f55994b`. Frozen target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The final evidence commit is documentation-only after the tested source.

## Applicability and bounded behavior

Three [unchanged originals](../tests/fixtures/cpu-dependent-proofs) come from
pinned `tests/proof`; they and derived oracle graphs/text/diagnostics retain
Apache-2.0 attribution. New tests/adapters use MPL-2.0. All originals check and
normalize: dependent_pair yields `Z{}`; the two equation proofs yield `{==}`.
Original builds need Base. Typed Base-importing main adapters generate; all port
lanes execute successfully. The dependent-pair adapter fails upstream C compilation
with an undefined constructor ID, while its other runtime lanes agree. Generation
reports and successful artifacts are separate from execution.

The complete packed value `MkP{Z{}, U{}}` checks and normalizes with both
fields preserved. All five emitters refuse its dependent-field printer with exact
status 1, empty stdout and no artifact. The
[pinned printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1200)
rejects fields whose type cannot be statically walked; its Eql descriptor at line
1227 permits closed equation printing. Thus `fst(packed())`, the entire pair,
and closed proof output have different applicability. Printing `{==}` does not
replace checking the equation endpoints.

| Family | Bounded direct applications | Separately checked observations |
| --- | --- | --- |
| dependent_pair | packed(), fst of the packed constructor, isZ at N depths 0/1/2/4: **6** | Complete DPair graph and printer refusal; invalid S/U witness rejects |
| multi_column_match | and2 and and_comm on all four Bit pairs: **8** | Four typed equation callers; false endpoint rejects |
| shared_scrutinee | add(n,Z) and add0(n), depths 0/1/2/4: **8** | Four typed equation callers; false endpoint rejects |

The 22 applications comprise eight proofs and fourteen helper/type-family
results. Eleven complete definition normal forms include isZ, packed, fst, and2,
and_comm, add, add0, q and three mains. The add0 graph retains every rewrite
field: recursive evidence, complete motive, body, equation endpoints, type and
binder positions. Exact text/readback checks remain alongside structural checks.
No graph nodes, motive fields or constructor children are dropped.

[Kernel tests](../tests/kernel_dependent_proofs.rs) compare the original/definition
graphs and all direct results, typed proof callers, complete values, scalar
projections and negative witnesses. The closed-constructor serializer retains its
deep/escaping/rejection controls. [Compiler tests](../tests/compiler_dependent_proofs.rs)
run 16 applicable mains through native Rust Pure, strict C and Node JS; the
complete DPair stays in exact-refusal tests. Three negatives require exact
status/stderr, empty stdout and no artifact across check/run/compile. Diagnostic
JSON preserves whitespace losslessly. Independent review found no issues in
source tests or the complete private graph probe. No production/shared helper,
dependency or evaluation policy changed.

## Validation and open oracle failures

| Check | Result |
| --- | --- |
| Full standard gate | **1,067 passed / 0 failed / 49 ignored**, 541.676 seconds |
| Focused tests | **39 / 0 / 0**, 29.031 seconds |
| Complete graphs | **112 structural observations**, both implementations twice |
| Normalization/applicability | **140 observations**: exact text/reports, 40 repeated refusals and 30 artifact generations kept distinct |
| Negative family | **48 exact rejections**, plus six checking-phase probes |
| Runtime matrix | **188 exact executions**, 95 successful generation/build checks, **two upstream-C compile failures and two upstream-JS timeouts** |
| Blocked runtime cells | **Two upstream-C executions unrun**, no executable or runtime status |
| Checking audit | **1,302 unchanged statuses/streams**; 439 expected negatives reject |
| Additional controls | **14 positive passes**; 18 observations preserve three hole/TODO disagreements |

The 112 graph observations cover 11 definitions, six complete/projected helper
mains, eight typed proofs and three main adapters. The 140-observation matrix
renews eight non-main definition texts twice, including the full recursive rewrite
motive. Type-only and phase-only probes are labeled separately. Positive artifact
generation checks record exact commands, streams and hashes without treating CLI
metadata as Bend program output. Artifact-hash variation across repeated builds:
**0 observations**; raw hashes remain in the receipt.

External runtime lanes are upstream interpreter, port pure CLI, upstream C O3,
strict port C O3, upstream JS and port JS, twice. The two equation main adapters,
all eight typed proofs, scalar projections and the complete Boolean list match
every lane. The dependent-pair adapter matches five lanes; upstream C cannot build.
The complete recursive-N list matches native Rust, port C/JS and upstream
interpreter/C; upstream JS times out twice at the separate three-second diagnostic
bound. Its local S constructor tag disagrees with the qualified printer descriptor,
reproducing the [known imported-printer failure](cpu-imported-printer.md). The
failing JS and exact empty streams remain preserved. Host SIGKILL is not a
language exit status. Runtime runner exit remains **1**; no exception is added.

The dependent-pair adapter exposes a separate upstream C code-generation defect.
Generated C refers to `CID____DEPENDENT_PAIR_U` without defining it. Two identical
O3 compilation attempts exit **1** with identical diagnostics. The generated
source SHA-256 is
`34d6fcd8ec760cab738c826d5ebe40478a610fa76081b85cd15909a745953205`.
Both intended runtime executions are **unrun**, with no executable or runtime
exit status. Installing a host dependency would not define this emitted symbol.
Port native/C/JS and upstream interpreter/JS succeed for this adapter; they do
not establish cross-upstream parity for the blocked C lane.

Pinned [`carb_book` and `type_adts`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1387)
collect reachable ADTs from types, annotations and references.
[`compile_book`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2827)
seeds constructor IDs from that collection, while
[`ctr_build`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1047)
emits their uses. Incomplete collection around the dependent witness is the
source-supported diagnosis, independently reviewed. The precise first missed
traversal edge and broader scope have not been established. Upstream source and
generated C were not patched; this remains an unresolved oracle failure.

The first runtime runner stopped after 236 process observations at that C build.
Its immutable receipt and failing generated source remain preserved. The resume
reuses those observations and the unchanged generated C, repeats the build,
records the blocked cells explicitly and completes the other lanes. The combined
receipt has 287 actual process observations; blocked executions add no process
rows or passing credit. `completed` means all intended cells were observed or
explicitly blocked; `full_required_matrix_passed` remains false.

The initial preparation runner incorrectly assumed successful port generation
would leave stdout empty. The CLI correctly returned its generation metadata;
that assertion failure and receipt remain preserved. The runner now checks
artifact generation separately from program output. Initial applicability and
application preparation use historical `6523d46`; all current validation uses the
tested source above. No repository test failed in this batch.

Formatting, strict Clippy, all-feature build and Node-provider checks pass.
Aggregate 23/24/27-result work remains **634,793 / 861,655 / 1,338,629** under the
unchanged 2,000,000-step and 128-depth limits. Full gates renew aggregate sharing,
named calls, native/C/JS, demand/error ordering and guard/cancellation controls.
The external 72-observation aggregate matrix stays historical at `936d299`.
Cancellation remains at existing before/after-normalization/output boundaries;
mid-normalization cancellation is not claimed. Ignored tests are 47 GPU hardware
checks and two optional profilers.

Known minimal-IO upstream C compile failures, foreign diagnostics, erased-hole/
TODO, resource/host gaps, F32.read, competing failure order and the unfinished
688-vector exploration remain unwaived. The earlier
[no-IO emitter Err compatibility caveat](cpu-dead-residue.md) remains: repository
callers were audited at `dcf26ff`, external consumers were not. Stable checking
statuses do not establish universal diagnostic or CPU parity.

## Reproduction and storage

Activate installed tools and run from the repository root:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_dependent_proofs --test compiler_dependent_proofs --test kernel_proof_case_trees --test compiler_proof_case_trees --test compiler_dead_residue --test compiler_imported_printer --test executable_demand --test runtime_effects -- --nocapture
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug overrides control Rust build size, not Bend optimization. Upstream
uses `node <reference>/bend2/main.ts <fixture> -o <artifact>`. Port generation is
`teamy-bend compile <fixture> --executable --target <target> --output <artifact>`,
selecting c, javascript or native. C oracle builds use
`clang-19 -std=c11 -O3 <source> -lpthread -lm -o <program>`; port C adds
`-Wall -Wextra -Werror -pedantic` and uses `-pthread -lm -ldl`. C executes with
`--threads 1 --gpu off`, generated JS with Node. Port definition observations use
`teamy-bend --output-format json eval <fixture> --entry <name>`.
Exact commands, status, base64 streams, artifact hashes and cleanup are private.
Ordinary children have 30 seconds, known JS printer probes three, audit children
60, outer runners 1,200. Only task-owned timed-out processes are killed/reaped.

Frozen tools: Rust/Cargo 1.96.0, fmt nightly-2026-05-24, Node 24.19.0,
Clang 19.1.7 and PowerShell 7.5.4. Docs pin:
`50e89d6897ce2a65167ef65b17c5fa83fe849158`; reference checkouts remain clean.
Cargo.lock SHA-256: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured CLI SHA-256: `6d8124a89ee7216efd12a5e92e028d5905b56dac669e705daac59e0f6462ae78`.

Gate start/minimum free space: **11.823 / 10.780 GiB**.
The prebuild reserve is 11 GiB, stop floor 10 GiB, temporary-space floor 1 GiB.
Only inactive rebuildable caches are retired after verified capture; the matching
library is retained until complete graphs finish. Source and unique historical
evidence remain intact. Raw logs, command/stream receipts, generated failures and
binary captures stay outside Git and are not preserved by branch publication.

## Remaining coverage

Detailed receipts link **384 originals**, with **918 unlinked**: **357 checker
negatives, one checking-only module, 23 interpreter cases and 537 compiled
candidates**. These categories are not missing runtime-test counts. Links include
historical, partial and blocked results; CPU parity and the API/integration
inventory remain incomplete.

Next proposed batch: `check/dependent_telescope`, `capture_price_twin_000` and
`_001`. Establish wide dependent-field and branch-join applicability first, then
bounded valid records/closure applications and invalid witnesses. An equality
hypothesis forcing a zero scrutinee must not be faked to reach a positive arm;
test the independently callable helper where appropriate. Source/ledger selection
only, with no execution credit. Inputs, guards, budgets and cancellation remain
frozen. Reviewed feature checkpoints may be pushed; no main merge or release.


The [ledger](cpu-dependent-proofs-evidence.csv) groups **593 process observations**.
Two [blocked execution cells](cpu-dependent-proofs-blocked.csv) are separate and
do not contribute to that process count.
Three interpretation links append without rewriting historical rows. The
[acceptance criterion](cpu-parity-acceptance.md) remains unmet.

## Receipt identities

| Receipt | SHA-256 |
| --- | --- |
| `preflight.json` | `3ee40c54085897e9fbcb4afc503d14f4a51e388bf30724e5b9bdad061d1c44d3` |
| `applicability-initial.json` | `5929342f552498dc0867385775199e9ca8a570dc693b121516d36c03955e5df8` |
| `setup-initial.log` | `976adc355071643e40e62a9e30136931595df5f2994a010e4c3bed2907f0c9ba` |
| `applicability/receipt.json` | `82d1eed64cb243d2f61762852f4b19f2ddfb47b96eff8f72ad70ec9ed9260229` |
| `application-preflight.json` | `b31f8aac53c00140447138c8d2fc67e5ec81f23583dc5ed80beef4f6e2c472d9` |
| `independent-review.md` | `6319de00e81b90410f1fea6d2f541eb92ce6df57676eb91a34bd8646f477eb87` |
| `focused-final.json` | `ecc6af59063b575e91e4c44b21491af1f3bf146326217f2b4b46ae96e5289bc3` |
| `focused-final.log` | `e3724a977bfc858934f391a9784fa45985c1b0ae705a05a78b38969704cef7df` |
| `full-gate-final.json` | `d0a10dfe89ac9ad5dad56b35f0d56eeeca716f7cb33cad78cf8dfcd70fee1c4c` |
| `full-gate-final.log` | `808c0d539a289c1e0bde3a8227e85a0c5ab64ec2fffdcdf2b584fa90e35999d3` |
| `snapshot.json` | `376014e5369c5f6f4c1efa536b006eee9335c160289efaf95753e5c356912872` |
| `open-matrix/receipt.json` | `df4def6bbe3c5e615d44bf774343fea3e17b53fcfecc720d0df868a5e4cf3b52` |
| `graphs-matching/receipt.json` | `6ffd8c4ea45054b04677cb0fcf200f0e31ce4256b7c1472bd766b1ac5eff0e2e` |
| `negative-matrix/receipt.json` | `8cec2b8c855b97b4f12d86aad6ed8aef0f9229c2411d3ced15d4eab6096d345d` |
| `matrix/receipt.json` | `00d98af257fce26b35a1bad705a8de3c48bd6b3b90e3017d21d101450b5d0a88` |
| `matrix-run.json` | `ed955f8bde1c80f3d83d2fb26e0d71c950dfff296267338a0fef07ead740955e` |
| `matrix-initial/receipt.json` | `1327d157bb5ab1835120e10c257d23914e55bcb9ffa9d32bf45643c3c7ab2b86` |
| `matrix-initial/dependent-pair-upstream.c` | `34d6fcd8ec760cab738c826d5ebe40478a610fa76081b85cd15909a745953205` |
| `matrix-initial-run.json` | `ed955f8bde1c80f3d83d2fb26e0d71c950dfff296267338a0fef07ead740955e` |
| `matrix-initial-run.log` | `9e9afff6af4e4cb2f6c6265c3870e8a0f5eab3841bb5290989ff973763fb6c39` |
| `matrix-resume.json` | `011dfe88363389c803e5ef89a8a4549310d806636a2303016ca6b18048341085` |
| `matrix-resume.log` | `0e3edcfa9268fff223f9ec2313c8322eb37bcd899ebe0d1f42bdaf2604699444` |
| `runtime-matrix-initial.py` | `f13b3f49955f9997340684ec8bea3969723d7ca52c06df48b6a206a4ee2ae93f` |
| `runtime-matrix-resume.py` | `24d9999d5d5ecd07ae4266efc52d11a0ea89ade2468f682f6f2a77f8289fe54a` |
| `upstream-c-independent-review.md` | `0840b641b9e7b92fae2d09c5f393fe71785be23dcf5504afc163e03206eaedce` |
| `upstream-js-evidence.json` | `5c564571b682d9d7207487dc5e00211def6d7cce81cba80df4f30c6ac72979f8` |
| `audit/receipt.json` | `038d48928c03f3a27e4438a1ffbd77e5f97812209386a50b81cb787dae8a6db4` |
| `audit-delta.json` | `c6e77cbf7663e7f6ab499c0ca2babe4065da35c70325ad9100cf525d3e177931` |
| `controls.json` | `049f429f3b73b565988bfcf3fb1a7fdbe08207948f7dd0f777fb9eb3f26fa198` |
| `coverage-summary.json` | `78c1558b2bd424e8cfbf141bfd7e612efc818706b3be9a0ce4acb653d25e63b3` |
| `tool-versions.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `comparison-library-reclaimed.json` | `fc88ebe1905e486ffc77e432e3799b1c57e76291652c34488787d23efdc43c00` |
| `next-gap.json` | `9ef505d368703d3c40bf1466ae9ab15ad5e5d25c4d6735a971ee2a24a8f252a1` |
