# Nested word switches and canonical recursion

Tested source: `ad887a40b6c8acd092610b33fb29c9e1c89c5a2d`. Starting checkpoint:
`8ce23911f4b1ee859c6c838f09cbbc0cf0fe7d4c`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Only tests, fixtures and documentation change. The final evidence commit is
documentation-only after this tested source.

## Applicability and concrete observations

Four [originals](../tests/fixtures/cpu-word-switches) are unchanged Apache-2.0
copies from pinned `tests/flatten`. Derived normal forms, complete graphs and
refusal diagnostics retain attribution; new tests/adapters/wrappers use MPL-2.0.
Each original is a function. Actual standalone compilation returns status 1,
empty stdout and `Error: a build needs import Base\n`. Separate typed
Base-importing adapters then reach exact function-printer refusals without
artifacts. These refusals are not executable runtime passes.

Initial applicability has 52 observations using historical source `6c8076a`.
The final 120 observations renew original/adapted applicability on the tested
source: pure CLI text, strict eval report text, upstream type/checking phase,
direct applications and 80 exact repeated refusals are distinct observations.

| Original | Bounded inputs | Required result |
| --- | --- | --- |
| `nested_word_switch_000` | All 16 ordered depth pairs from 0,1,2,4 | K100 for both zero; KP adds one constructor to the entire second argument; KN/KM preserve whole arguments |
| `nested_word_switch_001` | Same 16 pairs | K1/K2/K3/K4 for zero/zero, zero/positive, positive/zero, positive/positive |
| `word_decrement_halt` | Depths 0,1,2,4,8 | Z in every case |
| `joint_fused_canonical_le` | Same 16 pairs | T iff the first depth is less than or equal to the second |

All **53** direct expected results are authored from those rules and checked
against the pinned interpreter before port testing. Four complete imported-value
lists and four projection mains reuse the inputs. Scalar codes are documented
in the fixture README; full constructor values never collapse to those codes.

[Kernel tests](../tests/kernel_word_switches.rs) compare original graphs/text
and readback, each complete application result, every wrapper/result graph and
individual projections. The canonical joint function must retain its point-free
match tree, recursive references, binder positions, ordered arms and fallbacks;
no presentation rewrite hides a different graph. Alpha-canonical comparison does
not assert physical Rc identity, which has separate sharing regressions. The
explicit-stack constructor serializer retains every ordered child and keeps its
deep/escaping/rejection controls.

[Compiler tests](../tests/compiler_word_switches.rs) execute all eight authored
mains through native Rust Pure, strict generated C and Node JS. Original and
adapted refusals preserve exact status/streams and artifact absence. Independent
read-only review of this source found no issue or weakened comparison. No
production correction or new contract decision was required.

## Results and unqualified lanes

| Check | Result |
| --- | --- |
| Full standard gate | **1,043 / 0 / 49**, 519.312 seconds |
| Focused tests | **36 / 0 / 0**, 21.824 seconds |
| Complete graphs | 48 matches, two implementations twice |
| Normalization/applicability | 120 observations, including 80 exact refusals |
| Runtime matrix | 94 exact executions; 48 generation/build successes; 2 unresolved upstream-JS timeouts |
| Checking audit | 1,302 unchanged statuses/streams; 439 expected negatives reject |
| Additional controls | 14 positive passes; 18 observations retain three hole/TODO disagreements |

The repeated external lanes are upstream interpretation, port pure CLI,
upstream C O3, strict port C O3, upstream JS and port JS. Projections match all
six lanes. Full-value nested switch 001, decrement and joint comparison also
match all six. The field-bearing nested switch 000 results match native Rust,
port C/JS, upstream interpretation and upstream C, but the upstream-JS printer
times out twice at the separate three-second diagnostic bound with empty
streams. Its generated local constructor tags disagree with qualified printer
descriptors, reproducing the [known imported-printer failure](cpu-imported-printer.md).
Generated failures are retained. Timeout termination is not a language exit
status; the matrix runner exits **1**, with no new oracle exception.

Formatting, strict Clippy, all-feature build and Node-provider checks pass.
The full gate renews aggregate sharing/native/C/JS, named-call reconstruction,
demand/error ordering, guards and cancellation. Work counts for 23/24/27-result
aggregates remain **634,793 / 861,655 / 1,338,629**, under unchanged 2,000,000-step
and 128-depth limits. Cancellation stays at existing before/after-normalization/
output boundaries; no mid-normalization interruption is claimed. The external
72-observation aggregate matrix remains historical at `936d299`. Ignored tests
are 47 GPU hardware checks and two optional profilers.

The [no-IO emitter API caveat](cpu-dead-residue.md) remains: external consumers
may receive Err where executable source was formerly returned. Repository callers
were audited at `dcf26ff`; external consumers were not. Existing minimal-IO
upstream C compiler failures, foreign checking diagnostics, erased-hole/TODO,
resource/host gaps, F32.read, competing failure order and the unfinished
688-vector exploration remain unwaived. Unchanged checking statuses alone do not
qualify the corpus or establish exact diagnostic parity.

## Reproduction and storage

Activate the installed tools and run from the repository root:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_word_switches --test compiler_word_switches --test kernel_statement_sequence --test compiler_statement_sequence --test compiler_dead_residue --test compiler_imported_printer --test executable_demand --test runtime_effects -- --nocapture
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Rust host debug overrides control build size, not Bend optimization. Primary
upstream C uses `clang-19 -std=c11 -O3`, `-lpthread -lm`, and runs with
`--threads 1 --gpu off`. Port C adds
`-Wall -Wextra -Werror -pedantic -pthread -lm -ldl`. Exact per-process commands,
status, base64 streams, repetitions and cleanup results are in private receipts;
public hashes and the execution ledger identify them without machine paths.
Ordinary children have 30 seconds, known upstream-JS printer probes three,
checking children 60 and outer runners 1,200 seconds. Task-owned timed-out
processes are killed and reaped.

Frozen tools match the previous receipt: Rust/Cargo 1.96.0, fmt
nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4. Docs pin:
`50e89d6897ce2a65167ef65b17c5fa83fe849158`; both reference checkouts stay clean.
Cargo.lock SHA-256: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured CLI SHA-256: `865be9ac70bed15aa96e7cad6b2e58fc45c724d2d5b92eee5b944d652198d3ef`.

Build preflight requires 11 GiB free above an unchanged 10 GiB stop floor.
The full gate began at **11.935 GiB**, minimum
**10.881 GiB**. Completed inactive first-party test/build
caches are reclaimed only after verified binary capture; the comparison library
is reclaimed after complete graph verification. Previous deduplication manifests,
all unique captures and old failure receipts remain intact. No additional
historical evidence compaction or removal was needed in this batch. Private raw
logs, command/stream receipts, generated failures and binary captures remain
outside Git; publishing source and compact ledgers does not preserve those
private files remotely.

## Coverage and next batch

Detailed evidence links **376 originals**, leaving **926 unlinked**: **357 checker
negatives, one checking-only module, 31 interpreter cases and 537 compiled
candidates**. These are applicability categories, not missing runtime-test
counts. Links include historical, partial and blocked evidence; they do not
establish full CPU parity or close the API/integration inventory.

Next proposed family: `flatten/type_param_match_000` and `_001`, the two remaining
unlinked flatten interpreter candidates. Establish type/kind and actual printer
applicability first, then validate the leading Type argument before choosing
bounded list/Nat inputs. This is source/ledger selection only, without execution
credit. Dependencies, toolchains, references, budgets, guards and cancellation
remain frozen. Reviewed feature checkpoints may be pushed; no main merge,
release, installation, desktop or GPU work. Private evidence stays outside Git.

## Receipt identities

The [execution ledger](cpu-word-switches-evidence.csv) records 312
observations. Four interpretation links are appended without changing historical
rows or source identities. The [CPU acceptance criterion](cpu-parity-acceptance.md)
remains unmet.

| Receipt | SHA-256 |
| --- | --- |
| `preflight.json` | `b3cede4d89f8acc681b7d3f2d9e9e70b89eef7ac62bfc12b17c72fa42df757e8` |
| `applicability/receipt.json` | `dbeb70c92e502ba8a982e62efcfed7be65b7c0dec65723183f71eef904b02f23` |
| `application-preflight.json` | `913c7214cfa6579d1c0e6e01ff21dd4b0d0adc29a4b2fee12cad2681cc95ce21` |
| `independent-review.md` | `81eec1f3cfb09ab71340c04a03ed7a97ff01728d3a5b5829141cb4415924fc02` |
| `focused-final.json` | `f47fd8a5c4c10cbc9f3ce5b238232e905ab692ec4e956cd1115f55c36884a835` |
| `focused-final.log` | `2d1fdbb6530d808d4374444a5e59cf067f04e872295c6a1f424a866871006abd` |
| `full-gate-final.json` | `aa102c4c3d3b8a791ca1f7cea6c5f24680cc14bdfe506928b0ac231a1df088f9` |
| `full-gate-final.log` | `45ed7282b7d545414e23615fa7b00c8a3ef5fe26847140f4aa826f4c2abb5bec` |
| `snapshot.json` | `3870f2d4383af43b56d2e265ee265e12dc3576af50fc358b41a28f08b9b7aa81` |
| `open-matrix/receipt.json` | `e18e8ab3aafbc9fa396346345c8eb6017f8778fb53cf919661c97e4f3623cb47` |
| `graphs-matching/receipt.json` | `d96362f92494c3a675c522e0006f933ed56a5c765444e73d8e8851f004282654` |
| `matrix/receipt.json` | `19efa67a0dfaf1e7248e85696b56446c377de5ae565a4ab96b6c50f4acac9b1d` |
| `matrix-run.json` | `b35cc3d5c65408a9c2c624df3ec7b64f744e976f1526c1bb7286d99c0180c1f9` |
| `upstream-js-evidence.json` | `9c18ac6c0b5be693211fdc3d80dac2d80e193bce07a5b11280af521af3de9f5f` |
| `audit/receipt.json` | `5345ff110ef51cf9e7c41500ca314fe32be8da1e731214239afdef0f1c6d1523` |
| `audit-delta.json` | `e2120dd8c17baf5a2e854c9211c3e3c4a91baffdfc6cf3b5fb02216aa1abf1a9` |
| `controls.json` | `458cf81ee0ad9e46cc941311a28a1ece4c09cbc7e3bc70eeecc5fe96eeaed7e5` |
| `coverage-summary.json` | `969ef36715425d6d1a3fd964b6c5a1cb0a17f29e1465230287bab5b54fcbfa5c` |
| `tool-versions.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `comparison-preflight.json` | `23a5c10a1e508bb81781e52cde9aeeba6a4c234d8989404303867de0fffeb750` |
| `comparison-library-reclaimed.json` | `0a7227cdd50647a7e064b63d8ec7140e4058dde49873ebc9d5c5b63d81b6051c` |
| `next-gap.json` | `c79d329742f6f967a8737e3d4d7f9cb9a22576c1dc642a333e3caab9506fe857` |
