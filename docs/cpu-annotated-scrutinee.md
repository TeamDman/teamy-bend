# Annotated computed scrutinees and recursive list qualification

Tested source: `39634e208333393e6cf0b47502eb4669ba2fd590`. Starting checkpoint:
`60df78bd4dd5f381d20c518d3e4507ac6279e4c5`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
This batch changes tests, fixtures and documentation, with no production,
dependency, toolchain, evaluation-policy or safety-limit change. The final
evidence checkpoint is documentation-only after this tested source.

## Applicability and concrete coverage

Seven [originals](../tests/fixtures/cpu-annotated-scrutinee) are unchanged,
Apache-2.0 copies of pinned `tests/flatten/annotated_computed_scrutinee_000` and
its `_band`, `_beq`, `_neg`, `_sel`, `_str_eq`, `_str_len` companions. Derived
graph/text/diagnostic oracles retain attribution; authored wrappers/adapters/tests
use MPL-2.0. Every original main is a function. Actual standalone builds return
status 1, empty stdout and `Error: a build needs import Base\n`. Separate typed
Base-importing adapters reach exact function-printer refusals with no artifact.
These refusals do not become successful runtime lanes.

Initial applicability has 91 observations using historical binary `cc3818e`.
The 210 final observations renew applicability on the tested source, keeping
pure CLI text, strict eval report text, upstream type/checking phase and direct
application results distinct. There are 140 exact repeated original/adapter
refusals across upstream C/JS and port C/JS/native-target compilation.

| Original | Bounded inputs | Exact behavior |
| --- | ---: | --- |
| `_000` | 7 lists | Length, with zero selected to one |
| `_band` | 4 Boolean pairs | Boolean conjunction |
| `_beq` | 4 Boolean pairs | Boolean equality |
| `_neg` | 2 Booleans | Boolean negation |
| `_sel` | W depths 0, 1, 2, 4 | Depths 1, 1, 2, 4 |
| `_str_eq` | All 49 ordered list pairs | Equality iff all payloads and lengths agree |
| `_str_len` | 7 lists | Lengths 0, 1, 1, 2, 2, 2, 2 |

The lists are empty, T, F, TT, TF, FT and FF, encoded with the original `Ch`,
`Cs` and `Nl` constructors. T/F above denote constructors, not host Booleans
passed into Bend. Expected direct results and scalar projections are authored
from these rules and checked against the pinned interpreter before port testing.
The **77 inputs** are reused by seven complete imported-value lists and seven
scalar projection mains; one representation never substitutes for the other.

[Kernel tests](../tests/kernel_annotated_scrutinee.rs) compare full original
normal forms and readback, direct applications, and every wrapper/result graph.
Named calls, recursion, binder positions, ordered arms/fields and fallback nodes
remain explicit. The source annotation in `sel({str_len(s) : W})` is typechecked;
upstream normalizes it to `sel(str_len(s))`, as does the port. Full graph equality
does not preserve a source-only annotation node that neither normalizer emits.
Alpha-canonical graphs do not assert physical Rc identity; separate sharing
regressions remain. The explicit-stack closed-constructor serializer keeps all
nodes and existing depth/escaping/order/rejection controls.

[Compiler tests](../tests/compiler_annotated_scrutinee.rs) execute all fourteen
mains through native Rust Pure, strict generated C and Node JS. Independent
read-only review of the tested source found no blocking issue, incorrect expected
result or reduced structural coverage. No production fix was required.

## Results and unwaived limits

| Check | Result |
| --- | --- |
| Full standard gate | **1,030 / 0 / 49**, 506.416 seconds |
| Focused checks | **35 / 0 / 0**, 26.371 seconds |
| Complete normalized graphs | 84 matching observations, two implementations twice |
| Normalization/applicability | 210 observations, including 140 exact refusals without artifacts |
| Runtime matrix | 162 exact executions; 84 generation/build successes; 6 unresolved upstream-JS timeouts |
| Checking audit | 1,302 unchanged statuses/streams; 439 expected negatives reject |
| Additional controls | 14 positive passes; 18 observations retain three hole/TODO disagreements |

External runtime lanes are upstream interpretation, port pure CLI, upstream C O3,
strict port C O3, upstream JS and port JS, repeated twice. Scalar projections
and the four Boolean-result full-value lists match all six lanes. The three
field-bearing W-result lists (`_000`, `_sel`, `_str_len`) match native Rust,
port C/JS, upstream interpretation and upstream C. Their upstream-JS printers
hit the separate three-second diagnostic bound twice, with empty streams.
Generated field-bearing tags are local while descriptors use qualified names,
reproducing the [unresolved imported-printer defect](cpu-imported-printer.md).
Failing generated programs, hashes and prior receipts remain intact. Timeout
termination is not an upstream language exit status. The complete matrix runner
exits **1**; no new oracle exception or full CPU parity is claimed.

Formatting, strict Clippy, all-feature build and Node provider checks pass. The
full gate renews named-call reconstruction, aggregate sharing/work/native/C/JS,
demand/error ordering, guards and cancellation. Aggregate 23/24/27-result costs
remain **634,793 / 861,655 / 1,338,629**, with the same 2,000,000-step and 128-depth
limits. Cancellation stays at existing before/after-normalization/output
boundaries; no mid-normalization interruption or general resource equivalence
is claimed. The external 72-observation aggregate matrix remains historical at
`936d299`. Ignored checks: 47 GPU hardware tests and two optional profilers.

The [IO correction's compatibility caveat](cpu-dead-residue.md) is unchanged:
external executable-emitter callers may receive Err for formerly accepted no-IO
inputs. Repository callers were audited at `dcf26ff`; external consumers were
not audited. Current passing refusal tests do not perform that external audit.
Minimal-IO upstream C compiler failures, foreign checking diagnostic differences
and all earlier host/resource/semantic disagreements remain separately open.

## Reproduction and storage

Activate the existing tools and run from the repository root:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_annotated_scrutinee --test compiler_annotated_scrutinee --test kernel_nested_computed --test compiler_nested_computed --test compiler_dead_residue --test compiler_imported_printer --test executable_demand --test runtime_effects --quiet
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug overrides do not change Bend optimization. Exact private argument
arrays, status/stream bytes, source/artifact hashes and timings accompany:

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
teamy-bend --output-format json check "$fixture"
```

Refusal probes also use `--target native`, which packages C and is distinct from
the native Rust driver. Execution repeats twice. Ordinary children have 30-second
bounds; imported-JS full-value printer probes have three seconds; checking
children have 60 seconds; outer runners have 1,200 seconds. Only task-owned
timed-out processes are killed and reaped.

Frozen versions verified from the repository: Rust/Cargo 1.96.0, fmt
nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4. Docs pin:
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. Both references remain clean.
Cargo.lock SHA-256: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured binary SHA-256: `547054f314c82eb29f1a0e222251eed36cf0a5250e4106a71d82ff4fe9c015c1`.

Build preflight requires 11 GiB free, one GiB above the 10 GiB stop floor.
The gate began with 11.044 GiB and reached a minimum of
**10.028 GiB**. Completed inactive first-party test/build
caches are reclaimed after verified binary capture; the comparison library is
reclaimed only after its graph comparisons finish. Existing archives are reused
only after exact byte comparison; unique differing captures remain losslessly
archived. Source, dependency caches, historical failures and the earlier
array-read archive/restore receipt are preserved. Compact graph receipts retain
all nodes. Final comparison headroom is 11.083 GiB.

## Coverage and next work

Tested source `39634e208333393e6cf0b47502eb4669ba2fd590` adds seven unchanged
`flatten/annotated_computed_scrutinee_000` originals/companions and **77 bounded
applications** covering annotated list length/selection, all Boolean cases and
all 49 ordered pairs of seven lists. Original graphs/text, complete constructors,
scalar projections and executable output remain separate. No production change
was needed. [Scope and receipts](cpu-annotated-scrutinee.md).

Full gate: **1,030 passed, zero failures, 49 ignored**. Focused controls:
**35 / 0 / 0**. All **84 complete graph** and **210 normalization/applicability**
observations complete, including 140 exact build refusals without artifacts.
The runtime matrix records **162 exact executions, 84 successful generation/
build checks and 6 unresolved upstream-JS printer timeouts**; runner exit
**1**. All required port observations pass. Projection or nullary-result
success does not qualify the field-bearing full-value upstream-JS printers.

Independent review found no blocking concerns or weakened comparisons. The source
annotation is checked; its elimination from normal form agrees with upstream.
Aggregate sharing/work/native/C/JS/demand/cancellation controls pass, with no
mid-normalization cancellation claim. All 1,302 checking statuses/streams remain
unchanged, including 439 rejecting negatives. Fourteen positive controls pass;
18 observations preserve three hole/TODO disagreements. External consumers of
the IO emitter correction remain unaudited: no-IO compilation may return Err
where it previously produced source. Earlier minimal-IO upstream C failures,
foreign diagnostics, resource/host gaps, F32.read, competing failure order and
the incomplete 688-vector exploration remain unwaived.

Detailed receipts link **368 originals**; **934 remain unlinked**: **357 checker
negatives, one checking-only module, 39 interpreter cases and 537 compiled
candidates**. Links include historical, partial and blocked results; they do not
establish complete CPU parity or close the separate API/integration inventory.
Next proposed batch: four `flatten/statement_sequence` originals/companions,
applicability first, then eight closed function inputs and two nullary proof
observations. No execution credit is assigned to that proposal. Dependencies,
toolchains, pins, budgets, guards and cancellation remain frozen. Reviewed
feature-branch preservation is now authorized; no main merge or release. Private
raw receipts, binaries and generated failures remain outside Git and must be
retained separately. No installs, desktop or GPU work.

## Receipt identities

The [execution ledger](cpu-annotated-scrutinee-evidence.csv) records 546 observations
without machine paths. Seven interpretation links are appended without rewriting
historical rows or their source identities.

| Receipt | SHA-256 |
| --- | --- |
| `preflight.json` | `800af9620cbf7033d4b3a6c0203fde34cd9c6266bd2a5a7ca71a06c2dea71b7a` |
| `applicability/receipt.json` | `75b57d219dd98cd5c8e139e7be757a0e970a2dd9f10d0b42d867a2dc86f79b43` |
| `application-preflight.json` | `7884b8ccda0af00dce6731c7d3dc324514cf00c07e9a90bd39a15d7783da3fe1` |
| `independent-review.md` | `da5fbc68c0bae81e7066004400ee09e720e65cc3f889676081b586769053e461` |
| `focused-final.json` | `869db961f4397c4e93b8bdc7f06026ada45c7444d4631c4f0bef05bd4f212f3c` |
| `focused-final.log` | `f0847d55df82b7091790c2b900de975d769521fa50deb15526630a8e03bf7a49` |
| `full-gate-final.json` | `fd99ea021edf3371675fd21a9ed6967a85e8f634ea6db346b143b7256bfc134e` |
| `full-gate-final.log` | `d2d2d14a9cb5f599048d868cea2b1b1a05e6ff1e69734fbf11c8582b87ad9e76` |
| `snapshot.json` | `5c52f2a925ac0d4a49e72b64e01ef5405c314b8f7f1531909a717455ee70f769` |
| `open-matrix/receipt.json` | `d6ffbd924aff91d8d6e3f532d62255c00f823edc3118edd8bfa57c99fc4c8c3a` |
| `graphs-matching/receipt.json` | `cc11f1a8b034acacb02c2d572244b347e03711c17cee9648ac6068e97675c246` |
| `matrix/receipt.json` | `9a6143f91277c81924c5a9c0bc6157d22d6d863ae025ac82f5cd306ac254c955` |
| `matrix-run.json` | `e65b2480a5f753fb37d98660d7b2dfed8181c0498261ad17f8a67ff41549b22e` |
| `upstream-js-evidence.json` | `76a567b2ceed48908d95336e8d7eb2802786b5d801ae12eedd4d13c5d9eebd10` |
| `audit/receipt.json` | `4d726c903b2daf0442903bcdbee5599d3c49f636ee61d2cef26e6e938cc08850` |
| `audit-delta.json` | `7391d0aa89776da6d3a8b1103350be5fb22c62b00a90f1588d47cd8e3e1b29a1` |
| `controls.json` | `81710a7fdd786cacb2bec233f32bb7fafeddc0b6eff42661660fe6085cb3e01e` |
| `coverage-summary.json` | `1dc229e65c28f62ac931348ac64d5dcd3f49fa2b6eb6d3c8a8588dc1f01369b2` |
| `tool-versions.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `comparison-preflight.json` | `74bbadeed4b683d28de9139ef158af90d78d67c553cbc500fc01493353fcdb68` |
| `comparison-library-reclaimed.json` | `2720e060c89b453f14623cf2aabe0bf0013a745657280383fa1d256db0c6881c` |
| `next-gap.json` | `5eedbf1f09f00d1525b0e15a50bc9ca19759be224fb0a3bd4a08a1f399c98d95` |
