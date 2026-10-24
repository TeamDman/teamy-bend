# Dependent telescopes and affine branch capture

Tested source: `5f83d2f27bbc026541f885126b112650090250fa`. Starting published checkpoint:
`91320f69523a86e9c2da3ee3227bce0c6e5cb0bb`. Final evidence commit is documentation-only.
Frozen [upstream target](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599)
remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

## Applicability and observations

Three [unchanged originals](../tests/fixtures/cpu-telescope-capture) come from
pinned `tests/check`; originals and derived oracle graphs/text/diagnostics retain
Apache-2.0 attribution. New tests/adapters are MPL-2.0. Original checking and pure
normalization succeed; executable generation requires Base. Base-importing main
adapters generate successfully in upstream C/JS and port C/JS/native targets.
Generation metadata is kept separate from program stdout and actual execution.

| Original | Bounded inputs | What is observed |
| --- | --- | --- |
| dependent_telescope | Four valid eight-field records | First field fixed at Sq(Zq); six unused fields vary through depths 0–4; final proof depends on first field |
| capture_price_twin_000 | Two valid zero-scrutinee calls | Identity and successor closures; branch-join capture preserves affine use |
| capture_price_twin_001 | Two valid constrained calls, four fe calls, eight use.go calls | Both closures; independent helper depths 0/1/2/4; computed and variable twins kept distinct |

The constrained `use` functions require a proof that their input (or computed
input) is zero. No false proof is supplied to reach their positive arm. Positive
scrutinees are tested only through independently callable `fe` and `use.go`.
Three negative sources supply impossible witnesses and must reject.

Eight original definition graphs retain all binders, match arms, constructor
fields and named calls. Twenty original-entry applications have independently
typed source callers; their complete results are compared. Three complete
constructor-list aggregates, three scalar projections and three original-main
adapters are separate observations. The scalar outputs are:

- telescope: `[1, 1, 1, 1]`
- variable twin: `[0, 1]`
- computed twin: `[0, 1, 0, 0, 1, 1, 1, 2, 2, 2, 3, 4, 4, 5]`

[Kernel tests](../tests/kernel_telescope_capture.rs) compare complete graphs and
text, all 20 direct results and typed callers, nine aggregate/adapter results,
and rejection of false witnesses. [Compiler tests](../tests/compiler_telescope_capture.rs)
run the nine aggregate/adapter mains in native Rust Pure, strict C and Node JS.
Those nine runtime programs are distinct from the 20 individually checked/evaluated
callers; they are not 20 separately compiled executions. No production or shared
helper code, semantics, dependencies or safety limits changed.

## Validation and unresolved oracle failures

| Check | Result |
| --- | --- |
| Standard gate | **1,075 passed / 0 failed / 49 ignored**, 546.653 seconds |
| Focused checks | **39 / 0 / 0**, 26.771 seconds |
| Complete graphs | **148 structural observations**, upstream and port twice |
| Normalization/applicability | **114 observations**, including 30 exact Base refusals and 30 artifact generations |
| Negative family | **48 exact check/run/compile rejections**, plus six phase probes |
| Runtime matrix | **100 exact executions**, 54 generation/build successes, **eight JS timeouts** |
| Current C failures / unrun cells | **0 / 0**; previous failures remain separately historical |
| Checking audit | **1,302 unchanged statuses/streams**, including 439 rejecting negatives |
| Additional controls | **14 positive passes**; 18 observations retain three hole/TODO differences |

The 148 graph observations cover eight definitions, twenty typed callers and nine
aggregate/adapter mains. The 114-observation matrix keeps exact text, strict-eval
reports, type observations and phase probes separate from execution. Repeated
artifact hashes are stable. The [execution ledger](cpu-telescope-capture-evidence.csv)
groups **478 actual process observations** without adding blocked or unrun credit.

External lanes are upstream interpreter, port pure CLI, upstream C O3, strict
port C O3, upstream JS and port JS. Each executable main runs twice. All port
lanes and upstream interpreter/C match. Upstream JS times out twice for each of
three complete imported-constructor lists and the field-bearing telescope main
adapter: **eight observations** at the separate three-second diagnostic bound.
The other five mains agree across all six external lanes. Saved generated JS
retains the known local constructor tag versus qualified printer-descriptor
mismatch. Host SIGKILL is not a Bend exit status; exact empty streams, commands
and generated failures are retained. Runtime runner exit **1** and
`full_required_matrix_passed=false` remain explicit.

The [preceding dependent-pair report](cpu-dependent-proofs.md) retains its two O3
compilation failures for undefined `CID____DEPENDENT_PAIR_U` and **two unrun
upstream-C executions**. Those are historical `99ff1baa` observations, neither
rerun nor counted as successes here. Minimal-IO upstream C, foreign diagnostics,
erased-hole/TODO, F32.read, competing failure order, host/resource/API gaps and the
incomplete 688-vector exploration remain unwaived. The external executable-emitter
compatibility caveat from [dead-residue work](cpu-dead-residue.md) remains; repository
callers were audited at `dcf26ff`, external consumers were not.

## Test-harness corrections and safety controls

Initial preparation used an invalid lambda spelling in an adapter. Pinned source
accepts `x => x`; the malformed source and failed preparation receipt are retained.
Corrected preparation has 165 observations using historical binary `99ff1baa`;
current validation uses the clean tested source above.

The initial focused run at `d4e6c14` failed because `Path::with_extension` stripped
`.go` from a definition key and read `use`'s expectation for `use.go`. Expected
filenames now append their extensions explicitly. A separate abort occurred while
recursively decoding the expected graph JSON. A refined private probe recorded
`wrappers/dependent_telescope: decode expected Shape` **after evaluation returned**.
The test now uses the existing iterative constructor-graph serializer, comparing
every name and ordered child with the complete oracle JSON. Unsupported nodes
reject; deep, escaping-name and invalid-node controls remain. No larger host
stack, new evaluation policy or raised Bend guard was used. Independent review
found no remaining concerns. Initial failures remain unique evidence.

Formatting, strict Clippy, all-feature build and Node-provider checks pass.
Aggregate 23/24/27-result work remains **634,793 / 861,655 / 1,338,629**, under the
unchanged 2,000,000-step and 128-depth limits. Full gates renew sharing, named-call,
demand/error ordering, native/C/JS and guard/cancellation regressions. The separate
72-observation aggregate runtime receipt remains historical at `936d299`.
Cancellation remains at existing before/after-normalization/output boundaries;
no mid-normalization cancellation is claimed. Ignored tests: 47 GPU hardware
checks and two optional profilers.

## Reproduction, identity and storage

Activate installed tools and run from the repository root:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_telescope_capture --test compiler_telescope_capture --test kernel_dependent_proofs --test compiler_dependent_proofs --test compiler_imported_printer --test compiler_dead_residue --test executable_demand --test runtime_effects -- --nocapture
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug overrides control Rust build size, not Bend runtime optimization.
Upstream generation: `node <reference>/bend2/main.ts <fixture> -o <artifact>`.
Port generation: `teamy-bend compile <fixture> --executable --target <target> --output <artifact>`.
Upstream C: `clang-19 -std=c11 -O3 <source> -lpthread -lm -o <program>`.
Port C adds `-Wall -Wextra -Werror -pedantic`, with `-pthread -lm -ldl`.
C executes with `--threads 1 --gpu off`; generated JS uses Node.
Port definition observations: `teamy-bend --output-format json eval <fixture> --entry <name>`.
Exact commands, statuses, base64 streams and artifact hashes are in private receipts.
Ordinary children: 30 seconds; known JS printer probes: three; audit children: 60;
outer runners: 1,200. Only task-owned timed-out processes are killed and reaped.

Frozen tools: Rust/Cargo 1.96.0, fmt nightly-2026-05-24, Node 24.19.0,
Clang 19.1.7, PowerShell 7.5.4. Docs pin:
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Both references remain clean.
Cargo.lock SHA-256: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured CLI SHA-256: `f61c75fbbc954a0b5b43fefa14437ab154274a4c169ed4b8af6bf78a315c8a00`.

Gate start/minimum free space: **11.798 / 10.733 GiB**.
Prebuild reserve: 11 GiB; stop floor: 10 GiB; temporary-space floor: 1 GiB.
Only inactive rebuildable caches are reclaimed after verified capture. Historical
source, failures and unique evidence remain intact. Raw logs, complete command/
stream receipts, generated failures and binary captures are outside Git; pushing
the branch does not preserve those files remotely.

## Remaining coverage

Detailed receipts link **387 originals**, with **915 unlinked**: **357 checker
negatives, one checking-only module, 20 interpreter cases and 537 compiled
candidates**. These are applicability categories, not missing runtime-test counts.
Historical, partial and blocked links do not establish full CPU parity or close
the API/integration inventory.

Next proposed batch: `check/computed_match_specialize`, `cop_bare_trio_000` and
`fits_residual_fallback`. Establish proof-function and printer applicability;
then bound valid hypothesis specialization, Data-kind binder reuse and residual
fallback behavior. The first fixture has limited checking/main-output controls,
not complete definition/application qualification. This proposal adds no execution
credit beyond those separately recorded controls. No fabricated witnesses, budget increases or new exceptions.

The [acceptance criterion](cpu-parity-acceptance.md) remains unmet. This batch
adds three interpretation links without rewriting historical evidence rows.

## Receipt identities

| Receipt | SHA-256 |
| --- | --- |
| `preflight.json` | `fb84add9f084b644987b3f137c73250f17c962cba74e5e75b2678b9a234465f8` |
| `prepare-run.json` | `d997733ac1e1177079de944864392f103ca0039db64601aadfd394b0659f8638` |
| `prepare-run.log` | `e5e5b82b73d7e46a728b88f371214f667bed161ed8559a515a92f4fd73c6ff97` |
| `prepare.py.initial` | `c252423e24236baaa9942a81fb8feb5735653b659015f9f835d89e4f416b4fb5` |
| `preparation.json.initial` | `0f4e7b758a1b88dbc886766a451c02a19c59a98ef2eeb5b82285ce48ddbe8e02` |
| `invalid-lambda-adapter-initial.bend` | `16162cfa8b929e991f34ff63d3c481be4eeeb0a17c9c41e88f64d67a0fc4525d` |
| `prepare-corrected.json` | `ff6f5d839a5514405de7f137f5e779c45e41274551648e229f913b4071543a39` |
| `preparation.json` | `88b021c853d9c192bacc9a4a328b12e09e628cc366254d6659c97b304067041d` |
| `focused-initial.json` | `87210aac1acfb27f10329049995012f30ff88674c675b30812558d49995ac60b` |
| `focused-initial.log` | `1c8dfa54f32ce6811a4ebddb98a3ded9873a03dc48db52fa28dc71c40e198292` |
| `stack-probe-result.json` | `ede5719c7a0286e24f7164bb364a534a44e4d25ebc9eaba725ecfa9daea6a39b` |
| `independent-review.md` | `79a2ec7d3d807dc56431bca244d59db11eceb785cc80469c260bcb88f0d6dd5f` |
| `focused-final.json` | `3ced9a15040a6f1ab6424ad31e394b29f7289463a34084c39a64986fe56ffa47` |
| `focused-final.log` | `7614bd4b7d473aa76ba8784de1f3c541bd80994c570094c3525e7dcbda4c85e5` |
| `full-gate-final.json` | `00c4f5152fd7f3420dec1fbb2e796bb4c4f785bde6f0405afe7758ed667e9d79` |
| `full-gate-final.log` | `e9a9fe457de818e177119d3ab91cd8a75d825c9d264c218f45340d690b8663b4` |
| `snapshot.json` | `21104e87d5d8425c6897ca47653e530d99cf550087c2b0299d18dde97edabf71` |
| `open-matrix/receipt.json` | `31f4ab84ee4ff5f2c5c7a63dfd38587235821e46e79eadd78095452b18c60427` |
| `graphs-matching/receipt.json` | `35f9958c148ae70d1465f14d5829225308443e7ae27b97b0860361100de4af7c` |
| `negative-matrix/receipt.json` | `45ff0001b6b5ca0cc8943a5ef9a9a0aa82f9e00e8f130b8cdc4fe43f8beb5355` |
| `matrix/receipt.json` | `6b794118e0a975b0df00c007bcc5468056ea3243e06ed2ede87d2c7bbd7b9bcf` |
| `matrix-run.json` | `60d930e15313ea3e8215967e305ab287263287dfba46bb7ba103fbdcd80e3e40` |
| `upstream-js-evidence.json` | `59abaf0b0c36b194d51e80c98e1aed7de90d7486665b06a71b19ad32df29e08f` |
| `audit/receipt.json` | `d3a6528e6422a6f16c490f03dd57cfba708d213b38d88060785784b8e46d6a8c` |
| `audit-delta.json` | `85223020b625f7f2b663ac6be81bdfccb4a20bced90dc820002a4ca33280555d` |
| `controls.json` | `ecc9fec06f0d737c44b3bb7ecd3d5b8132bd0c814eaa4156c51cd4ff8640f813` |
| `coverage-summary.json` | `eced89de11d1c0b5b2b1c5b88161a41630cc948275d6a7007b724fe73cba76fb` |
| `tool-versions.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `next-gap.json` | `6fa5cefeb5a25c1b5784ea657e58611920a5b9ce7943ec8871161db76b43141c` |
