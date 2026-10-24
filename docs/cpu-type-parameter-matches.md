# Type parameters, list tails and constructor fields

Tested source: `60c4ba15df3a2cb81d4378243fcf3540f8ae5920`. Starting checkpoint:
`e666ad802f42da96437e3850ff3ce69bdcb2b2e6`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Changes are limited to tests, fixtures and documentation. The final evidence
commit is documentation-only after the tested source.

## Kind and printer applicability

Both [originals](../tests/fixtures/cpu-type-parameter-matches) are unchanged
Apache-2.0 copies of pinned `tests/flatten/type_param_match_000` and `_001`.
Derived graphs/text/diagnostics retain attribution; new tests and adapters use
MPL-2.0. Original standalone builds require Base; typed Base-importing function
adapters then refuse printing with status 1, empty stdout and no artifact.
Initial applicability has 26 observations using historical source `ad887a4`.
The final 60 observations renew original graph/text/report/applications and
40 exact repeated compilation refusals on the tested source.

The pinned [kind contract](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L189)
defines Type as Kind(&1), Data as Kind(&2), and permits Data where Type is expected.
Function types have kind Type. The
[inference rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3305)
also gives Kind(q) type Type. Actual typed callers confirm acceptance of `Type`,
`Data`, `H.Nat` and `(H.Nat -> H.Nat)` for each original. Passing `H.Z{}` instead
rejects at checking with exact expected-Type/observed-Nat diagnostics and Bend
source location, despite unused A. This prevents an untyped reduction of an
ignored argument from being mistaken for an accepted application.

The 180-observation kind matrix repeats normalization, port checking reports,
upstream phase probes and upstream C/JS plus port C/JS/native-target generation.
It includes **80 successful artifact generations** and **32 exact checking
rejections**, of which 20 are generation refusals without artifacts. Phase-only
observations remain separately labeled. Bare Type/Data probe artifacts are not
executed in this matrix. These generation successes do not establish runtime
reflection or make type checking a runtime operation.

The port's existing [executable elaborator](../src/kernel/elaborate.rs) represents
expressions of kind, quantity or equation type as erased after checking; no
elaboration or demand policy changed here. Runtime qualification below uses the
datatype and function-type arguments. Initial private kind probes have 70
observations at historical `ad887a4`; they are superseded for these public sources
by the current matrix, not relabeled as current results.

## Bounded behavior and structural checks

| Original | Inputs | Required complete result |
| --- | --- | --- |
| `_000` | Two type arguments × all 9 ordered list pairs = 18 | Tail of the first list, or Nil; second list does not change it |
| `_001` | Two type arguments × all 27 depth triples = 54 | F retains b,d when c=0; G retains b,c-1,d otherwise |

The type arguments are the local Nat datatype and Nat -> Nat. List inputs are
Nil, `[S{Z{}}]` and `[Z{}, S{Z{}}]`; each Nat triple ranges over depths 0,1,2.
Expected direct results are independently authored from these rules and compared
with the pinned interpreter before port testing. All **72** inputs are reused
by complete imported-value lists and separate scalar projections. F/G projection
tags are 0/1; full result graphs never collapse to those codes.

[Kernel tests](../tests/kernel_type_parameter_matches.rs) preserve complete
original graphs/text/readback, type binders, field positions, direct results and
wrapper graphs. Eight accepted kind probes also compare complete graphs; the
two invalid programs reject before normalization. The explicit-stack closed
constructor serializer retains every ordered child and its deep/escaping/
rejection controls. Alpha-canonical graphs do not assert physical Rc identity;
sharing has separate regressions.

[Compiler tests](../tests/compiler_type_parameter_matches.rs) execute four
bounded mains through native Rust Pure, strict generated C and Node JS. Original
printer refusals and invalid-kind failures preserve exact status, streams and
artifact absence. Negative diagnostics are losslessly JSON-encoded, preserving
blank-line trailing spaces without source-whitespace violations. Independent
review verified byte equality with raw fixtures at historical `42f49fa` and
found no issues or weakened comparisons. No production correction was required.

## Validation and open failures

| Check | Result |
| --- | --- |
| Full standard gate | **1,051 / 0 / 49**, 526.051 seconds |
| Focused tests | **37 / 0 / 0**, 24.223 seconds |
| Complete graphs | 56 matches, two implementations twice |
| Original normalization/applicability | 60 observations, including 40 exact refusals |
| Kind matrix | 180 observations; 80 generations; 32 exact rejections; phase/report evidence separate |
| Runtime matrix | 44 exact executions; 24 generation/build successes; 4 unresolved upstream-JS timeouts |
| Checking audit | 1,302 unchanged statuses/streams; 439 expected negatives reject |
| Additional controls | 14 positive passes; 18 observations retain three hole/TODO disagreements |

External runtime lanes are upstream interpretation, port pure CLI, upstream C O3,
strict port C O3, upstream JS and port JS, twice. Both projections match all six
lanes. Complete field-bearing results match native Rust, port C/JS, upstream
interpretation and upstream C; both upstream-JS printers time out twice at the
separate three-second diagnostic bound with empty streams. Generated local
constructor tags disagree with qualified descriptors, reproducing the
[known imported-printer failure](cpu-imported-printer.md). Generated failures
remain intact. Timeout termination is not an upstream language exit status.
Matrix runner exit is **1**; no new exception is introduced.

Formatting, strict Clippy, all-feature build and Node-provider checks pass.
Aggregate 23/24/27-result work remains **634,793 / 861,655 / 1,338,629** under
unchanged 2,000,000-step and 128-depth limits. Full regression controls renew
sharing/native/C/JS, named-call reconstruction, demand/error ordering, guards
and cancellation. Cancellation remains at existing before/after-normalization/
output boundaries. The external 72-observation aggregate matrix stays historical
at `936d299`. Ignored tests are 47 GPU hardware checks and two optional profilers.

The earlier [no-IO emitter API caveat](cpu-dead-residue.md) remains: external
callers may receive Err where source was formerly returned. Repository callers
were audited at `dcf26ff`; external consumers were not. Minimal-IO upstream C
failures, foreign diagnostics, erased-hole/TODO, resource/host gaps, F32.read,
competing failure order and the incomplete 688-vector exploration remain
unwaived. Unchanged checking statuses are not universal diagnostic parity.

## Reproduction and storage

Activate installed tools, then run from the repository root:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_type_parameter_matches --test compiler_type_parameter_matches --test kernel_word_switches --test compiler_word_switches --test compiler_dead_residue --test compiler_imported_printer --test executable_demand --test runtime_effects -- --nocapture
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug settings control Rust artifact size, not Bend optimization. Primary
upstream C uses `clang-19 -std=c11 -O3`, `-lpthread -lm`, and
`--threads 1 --gpu off`. Port C adds
`-Wall -Wextra -Werror -pedantic -pthread -lm -ldl`. Native-target kind probes
use the CLI's own packaged-C profile and only claim artifact generation. Exact
commands/status/base64 streams and cleanup results are retained privately;
public hashes identify the receipts without machine paths. Ordinary children
have 30 seconds, known JS printer probes three, checking children 60 and outer
runners 1,200 seconds. Only task-owned timed-out processes are killed and reaped.

Frozen tools match baseline: Rust/Cargo 1.96.0, fmt nightly-2026-05-24,
Node 24.19.0, Clang 19.1.7, PowerShell 7.5.4. Docs pin:
`50e89d6897ce2a65167ef65b17c5fa83fe849158`; both reference checkouts remain clean.
Cargo.lock SHA-256: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured CLI SHA-256: `0ab1ec1adde47617df283de80690642117fb912ac3ebea6300dea31aa9638065`.

The gate began with **11.924 GiB** free and reached
**10.848 GiB**, above the unchanged 10 GiB stop floor.
The prebuild requirement remains 11 GiB. Inactive rebuildable caches are reclaimed after
verified binary capture; the comparison library only after complete graph
verification. All unique historical evidence remains intact. Private raw logs,
command/stream receipts, generated failures and captures stay outside Git;
publishing tests and compact ledgers does not preserve those files remotely.

## Coverage and next work

Detailed receipts link **378 originals**; **924 remain unlinked**: **357 checker
negatives, one checking-only module, 29 interpreter cases and 537 compiled
candidates**. These are applicability categories, not missing runtime-test
counts. No flatten interpreter candidate remains unlinked, but flatten still has
unqualified compiled/negative cases and blocked receipts. Links do not establish
full CPU parity or close the separate API/integration inventory.

Next proposed batch: `state/eval_case_tree_001`, `_002` and `eval_split_apply`.
Establish dependent proof-function and printer applicability first, then 16
bounded proof applications and 36 corresponding enum/record/recursive helper
observations. This is source/ledger selection only, without execution credit.
Frozen references, dependencies, toolchains, budgets, guards and cancellation
remain unchanged. Reviewed isolated checkpoints may be pushed; no main merge,
release, installation, desktop or GPU work. Private evidence remains outside Git.

## Receipt identities

The [execution ledger](cpu-type-parameter-matches-evidence.csv) records 368
observations. Two interpretation links are appended without changing historical
rows or source identities. The [CPU acceptance criterion](cpu-parity-acceptance.md)
remains unmet.

| Receipt | SHA-256 |
| --- | --- |
| `preflight.json` | `0d60ccc74c3e3a3aad77d9efe0809f19dd7d7474fa87b68d8515669f3e2690b2` |
| `applicability/receipt.json` | `4c405192489dfe5f15cda90a62aaf7183515e8ec8c741b4655934bc651ec35e7` |
| `kind-preflight/receipt.json` | `5436c09e0c6b4c7fc80576acaf56ba45ce52aa9607b14fb473583ba418979b30` |
| `application-preflight.json` | `4c26449ae1ad9856d14a0a38e757cbcba3e2d29305b11a3ebd98892ccad589b1` |
| `independent-review.md` | `303a54fc41fb2ccd74ea031cdcd82dc6d1fff989fe6f99c0d8ebba74da0a0817` |
| `focused-before-diagnostic-encoding.json` | `cca1c03527f457735895729fbc8203c4e45bfe93b5b45d1548697f1ab374def9` |
| `focused-final.json` | `0e5cda79a42ef8f65d16083bfdc141b884b0f1c2961e9ac253db36758a651528` |
| `focused-final.log` | `0ec0308f909d19a13a048a057326a975b55fb2711765f37a63bf27613b27ddc9` |
| `full-gate-final.json` | `28cbc9b8eae9c3da8d63fc9ee7cbc366e916899fa5e28b6aa067ddf3d9292c34` |
| `full-gate-final.log` | `9f7f7db3fd96f4aceacde3109274c46529f3823019f55782bc7e7cdd25bb2ccd` |
| `snapshot.json` | `1ba45e519d5d26e0eafcf6055ffa9cfe8ca15f704b3f6fb3b063dfeb91f5ea34` |
| `open-matrix/receipt.json` | `f00d59b5a1c89a2cfaac43f38664af8b890e73c3ae5118ebefd8e054878bd0d2` |
| `graphs-matching/receipt.json` | `d2a142fb5b18c92c49ee301cdf3ed9f38614fa4e101dbc82470d8085da647c25` |
| `kind-matrix/receipt.json` | `e2347e952a2f5874073c14ef07b0e01df01120239ab331ccb5553a34bd375296` |
| `matrix/receipt.json` | `d40fb794de880b9ba298cfbb901dd5cb3775a76630cbcd42209f8f53dca95ec8` |
| `matrix-run.json` | `cde84147a88331fb3a72c3b9ea5c3417712226c0efa41445474af8403a3c83dc` |
| `upstream-js-evidence.json` | `0baebf6ea33b72920aa3c780472f2f3a2025c8fbf54534685b638c1e217b6df8` |
| `audit/receipt.json` | `cc9a978ecc8970459e78f63d14faa95392227620abbb0c6f7e8cef48a734cf2f` |
| `audit-delta.json` | `cb8206c09b5f8f491c0f9614a39803d61e1430a81155ad17d20a20c57d8d4709` |
| `controls.json` | `5a691fd3f3df590e4ecaf422b06d5272dc6287703cf7c412a3f2d2bb9afdcd13` |
| `coverage-summary.json` | `b464c6cca9cbf9201eb5df24ce8bdf991ddb94fe8734df96d4b1320e28961f47` |
| `tool-versions.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `comparison-preflight.json` | `579219d5f7b372d1c4c2982876ac268dbbe63d44ee277160133c2ac7ea939e02` |
| `comparison-library-reclaimed.json` | `99c891d7c3462e77a0d8a6af7e269b054ac074c9abbcff64bd0f6d52938aa82a` |
| `next-gap.json` | `c4ffb4d52e7dc850b3c114f6a5a1df120ffd314cd834282ebebeeacb3c03685a` |
