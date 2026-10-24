# Statement sequences and reflexivity qualification

Tested source: `6c8076a19225ae5cb7445b9752510d4a2db0bf9a`. Starting checkpoint:
`140f7cc95a0f984bdc4ed09d1428be789d7f045c`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Only tests, fixtures and documentation change. The final evidence commit is
documentation-only after this tested source.

## Applicability and behavior

Four [originals](../tests/fixtures/cpu-statement-sequence) are unchanged,
Apache-2.0 copies of pinned `tests/flatten/statement_sequence`, `_inc`, `_refl_f`
and `_refl_w`. Derived graphs/text/diagnostics retain attribution; authored
wrappers, adapters and tests use MPL-2.0. Every original standalone compilation
rejects with status 1, empty stdout and `Error: a build needs import Base\n`.
Typed Base-importing function adapters then reach actual printer refusals with
no artifact. The two proof adapters successfully generate executable code.

| Original | Observations | Exact result |
| --- | --- | --- |
| `statement_sequence` | W depths 0, 1, 2, 4 | W depths 4, 5, 6, 8 |
| `_inc` | W depths 0, 1, 2, 4 | W depths 1, 2, 3, 5 |
| `_refl_f` | Checked nullary identity-function proof | `{==}`, complete `Reflexivity` graph |
| `_refl_w` | Checked nullary constructor equality proof | `{==}`, complete `Reflexivity` graph |

Initial applicability has 56 observations using historical source `39634e2`.
The final 124 observations renew original/adapted applicability on the tested
source, including 60 exact refusals, 20 proof-artifact generations and four
executions of CLI-packaged native C proof adapters. This auxiliary packaged-C
lane is separate from native Rust Pure and from primary C O3 comparisons.
Eight direct applications have independently authored full constructor results
confirmed against the pinned interpreter. Two full-value lists and two scalar
projection mains reuse those inputs; neither representation substitutes for the
other. Nullary proof observations have no fabricated function arguments.

[Kernel tests](../tests/kernel_statement_sequence.rs) compare original full
graphs/text/readback, all direct results, full wrapper graphs and projections.
They check original and imported proof books before observing reflexivity.
Alpha-canonical graph comparison does not claim physical Rc identity; sharing
has separate regression controls. Closed-constructor graph encoding preserves
all children, names and order and rejects non-constructor nodes. Proof graphs
use a separate exact `Shape::Reflexivity` assertion.

[Compiler tests](../tests/compiler_statement_sequence.rs) execute six authored
mains through native Rust Pure, strict generated C and Node JS. Runtime proof
printing is output behavior after static checking, not runtime proof checking.
Independent read-only review found no issue or weakened comparison. No production
change or new contract decision was necessary.

## Results and remaining failures

| Check | Result |
| --- | --- |
| Full standard gate | **1,037 / 0 / 49**, 503.132 seconds |
| Focused checks | **36 / 0 / 0**, 25.172 seconds |
| Complete normalized graphs | 40 matching observations, two implementations twice |
| Normalization/applicability | 124 observations; 60 exact refusals; 20 proof generations |
| Runtime matrix | 68 exact executions; 36 generation/build successes; 4 unresolved upstream-JS timeouts |
| Checking audit | 1,302 unchanged statuses/streams; 439 expected negatives reject |
| Additional controls | 14 positive passes; 18 observations retain three hole/TODO disagreements |

Runtime lanes are upstream interpretation, port pure CLI, upstream C O3, strict
port C O3, upstream JS and port JS, repeated twice. Projection and proof outputs
match all six lanes. The two field-bearing full-value lists match native Rust,
port C/JS, upstream interpretation and upstream C, but their upstream-JS printers
hit the separate three-second diagnostic bound twice with empty streams. Retained
generated artifacts reproduce the local constructor tags versus qualified printer
descriptors described in the [imported-printer report](cpu-imported-printer.md).
Timeout termination is not an upstream language exit status. Matrix runner exit
is **1**; these required failures remain unqualified, with no new exception.

Formatting, strict Clippy, all-feature build and Node provider checks pass.
The full gate renews aggregate sharing/native/C/JS, named-call reconstruction,
demand/error ordering, guards and cancellation. The 23/24/27-result work costs
remain **634,793 / 861,655 / 1,338,629** under unchanged 2,000,000-step and 128-depth
limits. Cancellation remains at existing before/after-normalization/output
boundaries. No mid-normalization interruption or general resource equivalence
is claimed. The external 72-observation aggregate matrix remains historical at
`936d299`; current regression tests do not renew that external receipt.
Ignored tests: 47 GPU hardware checks and two optional profilers.

The [IO emitter compatibility caveat](cpu-dead-residue.md) remains: external
callers may now receive Err for no-IO executable builds where source was formerly
returned. Repository callers were audited at `dcf26ff`; external consumers were
not. Minimal-IO upstream C compiler failures, foreign checking diagnostics,
erased-hole/TODO, resource/host gaps, F32.read, competing failure order and the
unfinished 688-vector exploration remain unwaived. Passing this batch does not
close the [CPU acceptance criterion](cpu-parity-acceptance.md).

## Reproduction and storage

Activate the installed tool environment, then from the repository root:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_statement_sequence --test compiler_statement_sequence --test kernel_annotated_scrutinee --test compiler_annotated_scrutinee --test compiler_dead_residue --test compiler_imported_printer --test executable_demand --test runtime_effects -- --nocapture
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug overrides affect Rust artifact size, not Bend runtime optimization.
Primary upstream C uses `clang-19 -std=c11 -O3`, `-lpthread -lm` and execution
`--threads 1 --gpu off`; port C additionally uses
`-Wall -Wextra -Werror -pedantic -pthread -lm -ldl`. CLI-target native packaging
retains its own O2 build profile and is labeled separately. Exact per-process
commands, status, base64 streams, repetitions and deadlines are in private raw
receipts; their hashes below and the public execution ledger preserve identity
without publishing machine paths. Ordinary children have 30 seconds, known
upstream-JS printer probes three seconds, checking children 60 seconds and outer
runners 1,200 seconds. Task-owned timed-out processes are killed and reaped.

Frozen versions match the prior receipt: Rust/Cargo 1.96.0, fmt
nightly-2026-05-24, Node 24.19.0, Clang 19.1.7, PowerShell 7.5.4. Docs pin:
`50e89d6897ce2a65167ef65b17c5fa83fe849158`. References remain clean.
Cargo.lock SHA-256: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured CLI SHA-256: `4353d3314a623a000a41e790157c3e0ce2e9b8e0126b5ee48439b3600d9db95a`.

Two preliminary full-gate attempts are retained as failures, not test passes:
the first omitted the explicit formatter argument and stopped at unavailable
generic nightly; the second hit the unchanged 10 GiB disk floor after building,
before completing tests. It was stopped and reaped. No tool was installed and
no floor was reduced. Growth came from rebuilt first-party test executables;
older identical binary captures also occupied duplicate storage. A byte/hash-
verified hard-link compaction recovered 0.911 GiB while preserving every capture
path and byte; its manifest retains prior inode/timestamp metadata. All unique
historical evidence remains present. Only inactive rebuildable test caches were
removed. The completed gate started at **11.861 GiB**, minimum
**10.910 GiB**, above the unchanged 10 GiB floor. Binary capture
and completed graph verification precede subsequent cache reclamation.
Private raw receipts, generated failures and captures remain outside Git;
publication of source and compact ledgers does not remotely preserve those files.

## Coverage and next batch

Detailed evidence links **372 originals**, leaving **930 unlinked**: **357 checker
negatives, one checking-only module, 35 interpreter cases and 537 compiled
candidates**. Linked receipts include historical, partial and blocked evidence;
they do not establish complete CPU parity or close the API/integration inventory.
Next proposed batch: `nested_word_switch_000`, `_001`, `word_decrement_halt`
and `joint_fused_canonical_le`, applicability first, then 53 bounded inputs.
This is source/ledger selection only, without execution credit. Frozen inputs,
budgets, guards and cancellation remain unchanged. Reviewed feature checkpoint
publication is authorized; no main merge, release, installation, desktop or GPU
work. Private evidence and binary captures remain outside Git.

## Receipt identities

The [execution ledger](cpu-statement-sequence-evidence.csv) records 272
observations. Four interpretation links are appended without rewriting old rows.

| Receipt | SHA-256 |
| --- | --- |
| `preflight.json` | `c6ea918a479337c60910edabeb227ecbb409cb28aff048602804f3546f8b257b` |
| `applicability/receipt.json` | `7bbd829d083c229736d51fdde9c2bbd2ae7b8f97b8348b891c8eb36a837cf0c9` |
| `application-preflight.json` | `909cdad737f021bca2b88f225963440314497896521304cc6fd80a265adc42d0` |
| `independent-review.md` | `4054dcec5b7b7257be3c26b1e245b6217d61ec6c28ff2794fb34719a7792119f` |
| `focused-final.json` | `a7bbd335abfc6e9dba3254fdf3ae8abd1ff0ae42b01f3bce515e2771a29076e1` |
| `focused-final.log` | `272347d1ce7f69f91e9aa74bd538f3d52d2568c57aa2a9fc344b858ce06eb00e` |
| `full-gate-formatter-argument-omitted.json` | `409000e3ef56d87f83e068c06abdb3f6fe1f6042fd907cda0b042d239202cd90` |
| `full-gate-reserve-stop.json` | `261a807f8df626eacc6bb5322685718bdcdc9510499042c98f07e6438ad8e917` |
| `identical-captures-inventory.json` | `7b367d86a0d98eb1e12410be57d11d6b92e5d0af6add26be96513a75d00b12d7` |
| `identical-captures-shared.json` | `ef908db70fb5b1df0f68057bdda109fa89eb844f1bca862199502ec8c98a32c1` |
| `full-gate-final.json` | `60a630c283a9091da791e48cb5d840fcc517da46a61e1ad0a38ea25ad1d0c537` |
| `full-gate-final.log` | `e600f06f18f0f19cf161d2541f79dc15597881961361e5328b3224257661cecc` |
| `snapshot.json` | `4796f3d68f3b02f51e3604246718897f1f0a8139909e3b67d80679c01c9abd62` |
| `open-matrix/receipt.json` | `c6b837728217f2459119afcaab289a94cd58978ab923da018a1e14e55285b13e` |
| `graphs-matching/receipt.json` | `da749e50275e54b54f46487a328ad66b729266a52622a6cc60892d35d8ee0662` |
| `matrix/receipt.json` | `34f7bd56108649b86e2affcd3405cee0d86001b07048ade2a31e53ab7cd0f862` |
| `matrix-run.json` | `5d6ab8d020a1b7eb24c6e283f52cd24ead33aefc200a8319dcc7ffeb68793335` |
| `upstream-js-evidence.json` | `ec074b4d2889daf4aaa3a192057adbe3cd4bba8f5d56b2ef816be8f85a820321` |
| `audit/receipt.json` | `a13604500aa1fe1c1e91c615c445fcc6b2dcb6c0427b36e4ac04947faee65b8f` |
| `audit-delta.json` | `234af184d7f18a9db91104de3806a17eb9d07e0ffad38b13be31a88225d97ef9` |
| `controls.json` | `a4873592cafa80dc54e32d2c1ef840c766114251b1da5273156fd1e61494b025` |
| `coverage-summary.json` | `36130358924cf64bbb6a4f1b6f8700d801eb741779943f7b3c9b38dd4c4da6f1` |
| `tool-versions.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `comparison-preflight.json` | `4b365369e50dfe2ace9b4f829455193cbf3db525c5cd73b1bfd70746f090aa10` |
| `comparison-library-reclaimed.json` | `c566374f439b4819e590c09d7749987e00968440d292ac67b65ee1c6e1f7aa7f` |
| `next-gap.json` | `f183aa27306ad7c3755c0be38522f49b049417764f623963e22c89d75dae8f00` |
