# Proof case trees and closed equation output

Tested source: `6523d4654f81a427dc1bf30354deb7e0dd67bb91`. Starting checkpoint:
`e01e591c489f0314f8666a35502637c14a7147e7`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The final evidence commit changes documentation only after the tested source.

## Applicability before execution

The [three originals](../tests/fixtures/cpu-proof-case-trees) are unchanged
Apache-2.0 copies from pinned `tests/state`. Derived graphs/text/diagnostics retain
attribution; new adapters/tests use MPL-2.0. Their dependent proof-function types
check. Exact original normal forms are `d => turn_turn(d)`, `p => rot_x(p)` and
`n => wob_id(n)`. Full graph comparisons preserve lambda binders, applications
and named references; no printer rewriting hides differences.

Standalone compilation requires Base. With Base imported, the function-valued
adapters reject with exact status 1, empty stdout and no artifact across upstream
C/JS and port C/JS/native targets. The **60 repeated build refusals** are part of
the 102 current original observations. Initial 45-observation preparation used
historical source `60c4ba1`, not the final tested source.

Closed equation proofs have a different contract. The pinned
[printer descriptor](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1227)
explicitly maps Eql to printer node 5, which prints `{==}`. All sixteen typed
callers check and normalize before their executable outputs are compared.
Printing a reflexivity marker is not a proof certificate or a substitute for
checking the endpoints. No runtime reflection or new erased-value policy is
inferred.

An exploratory runner initially assumed closed equations would also refuse
printing; actual generation succeeded. That failed assertion and generated
source remain private evidence. Authored adapters also needed explicit reusable
binders and a constructor type annotation; their rejected versions remain intact.
Neither issue was a port defect or a change to an upstream fixture.

## Bounded inputs and complete observations

| Original | Proof inputs | Independent helper observations |
| --- | --- | --- |
| `eval_case_tree_001` | All four Dir constructors | turn/back on each: 8 results |
| `eval_case_tree_002` | All eight P3 triples over MN depths 0/1 | rot/getx/gety: 24 results |
| `eval_split_apply` | MN depths 0/1/2/4 | wob: 4 results |

The 16 closed proofs each have an explicit equation-typed source caller.
Thirty-six helper expectations preserve every constructor and ordered field.
Record observations retain the rotated record and both projections in a complete
result; separate scalar wrappers expose numeric codes without replacing it.
Originals, all six helper mains and sixteen proofs yield **100 full graph
comparisons**, two implementations repeated twice. Graphs are alpha-canonical
structural observations, not assertions of physical allocation identity.

[Kernel tests](../tests/kernel_proof_case_trees.rs) assert exact original
text/graphs/readback, complete helper results and wrapper graphs, sixteen typed
proofs and three false endpoint rejections. The closed-constructor JSON writer
keeps ordered children, deep graphs and escaping/rejection controls.
[Compiler tests](../tests/compiler_proof_case_trees.rs) execute six helper mains
and all sixteen proof mains through native Rust Pure, strict C and Node JS.
False claims require exact stderr/status, empty stdout and no artifacts at
check/run/compile boundaries. Negative diagnostic strings preserve whitespace
losslessly in JSON.

The first focused run at `a5996f5` failed: CheckedBook::evaluate uses declared
parameter count, while original main() has no parameters and returns a lambda.
The corrected test retains 36 helper API applications and checks all sixteen
H.main(arg) applications through the typed source callers. The independent
reviewer acknowledged missing this in the first review, inspected the corrected
API boundary and confirmed that no coverage was removed. No production or shared
helper changed, and no safety limit was adjusted.

## Validation and unresolved failures

| Check | Result |
| --- | --- |
| Full standard gate | **1,059 / 0 / 49**, 532.557 seconds |
| Focused checks | **39 / 0 / 0**, 28.175 seconds |
| Complete graphs | **100 exact structural observations** |
| Original applicability/normalization | **102 observations**, including 60 exact build refusals |
| Negative matrix | **48 exact rejections**, plus 6 checking-phase probes |
| Runtime matrix | **260 exact executions**, 132 generation/build successes, 4 unresolved upstream-JS timeouts |
| Checking audit | **1,302 unchanged statuses/streams**; 439 expected negatives reject |
| Additional controls | **14 positive passes**; 18 observations retain three hole/TODO disagreements |

External execution lanes are upstream interpreter, port pure CLI, upstream C O3,
strict port C O3, upstream JS and port JS, twice. All three scalar projections,
the enum complete results and all sixteen closed proofs match in all six lanes.
Complete record and recursive-MN results match port/native/C/JS and upstream
interpreter/C; upstream JS times out twice per case at the separate three-second
printer diagnostic bound. Its local constructor tags disagree with qualified
printer descriptors, reproducing the [known imported-printer failure](cpu-imported-printer.md).
Generated failing programs and exact streams remain preserved. SIGKILL after a
host timeout is not a language exit status. Runtime runner exit remains **1**.

Formatting, strict Clippy, all-feature build and the Node provider passed.
Aggregate 23/24/27-result work remains **634,793 / 861,655 / 1,338,629**, under the
unchanged 2,000,000-step and 128-depth guards. Full regressions renew sharing,
named-call reconstruction, native/C/JS, demand/error ordering, structural/work/
depth guards and existing cancellation boundaries. The separate 72-observation
aggregate matrix remains historical at `936d299`. Cancellation is only at
existing before/after-normalization/output boundaries; no mid-normalization
cancellation is claimed. Ignores remain 47 GPU hardware tests and two optional
profilers.

Minimal-IO upstream C compilation failures, foreign diagnostic differences,
erased-hole/TODO behavior, resource/host gaps, F32.read, competing failure order
and the unfinished 688-vector exploration remain unwaived. External callers of
the earlier [no-IO emitter Err change](cpu-dead-residue.md) remain unaudited;
repository callers were audited at `dcf26ff`. Checking-status stability does not
establish universal diagnostic equality.

## Reproduction, frozen inputs and disk safeguards

Activate installed tools, then from the repository root:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_proof_case_trees --test compiler_proof_case_trees --test kernel_type_parameter_matches --test compiler_type_parameter_matches --test compiler_dead_residue --test compiler_imported_printer --test executable_demand --test runtime_effects -- --nocapture
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Rust host debug settings control build size, not Bend runtime optimization.
Upstream generation uses `node <reference>/bend2/main.ts <fixture> -o <artifact>`;
port generation uses `teamy-bend compile <fixture> --executable --target <target>
--output <artifact>`, selecting `c` or `javascript`. Primary upstream C compiles with
`clang-19 -std=c11 -O3 <source> -lpthread -lm -o <program>` and executes with
`--threads 1 --gpu off`. Port C adds `-Wall -Wextra -Werror -pedantic` and uses
`-pthread -lm -ldl`. Generated JS runs under Node. Exact command arrays,
statuses, base64 streams, generated failure hashes and cleanup are private
receipts. Ordinary child bounds are 30 seconds, known upstream-JS printer probes
three, audit children 60, and outer runners 1,200. Only task-owned timed-out
processes are killed and reaped.

Tools remain Rust/Cargo 1.96.0, fmt nightly-2026-05-24, Node 24.19.0,
Clang 19.1.7 and PowerShell 7.5.4. Docs pin:
`50e89d6897ce2a65167ef65b17c5fa83fe849158`; reference checkouts remain clean.
Cargo.lock SHA-256: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured CLI SHA-256: `ed1b1656e18177b4142fc5f0a601dd14dea592d3e2903c8660616f92246808aa`.

Gate start/minimum free space: **11.834 / 10.813 GiB**.
Prebuild reserve stays 11 GiB, stop floor 10 GiB, temporary-space floor 1 GiB.
Only inactive rebuildable first-party caches are retired after verified capture;
the matching comparison library is kept until graph verification finishes.
Source and unique historical evidence remain intact. Raw logs, command/stream
receipts, generated failures and captured binaries remain outside Git; branch
publication does not preserve those private files remotely.

## Remaining acceptance coverage

Detailed receipts link **381 originals**, leaving **921 unlinked**: **357 checker
negatives, one checking-only module, 26 interpreter cases and 537 compiled
candidates**. These are applicability categories, not missing runtime-test
counts. Historical, partial and blocked links do not establish full CPU parity
or close the separate API/integration inventory.

Next proposed batch: `proof/dependent_pair`, `multi_column_match` and
`shared_scrutinee`. Establish kind/printer applicability before bounded dependent
pair validity, all four Bit pairs and Nat depths 0/1/2/4. Keep field refinement,
strict proofs, recursive rewriting and executable proof output distinct. This is
source/ledger selection only, with no execution credit. Frozen references,
dependencies, toolchains, budgets, guards and cancellation remain unchanged.
Reviewed isolated checkpoints may be pushed; no main merge or release.


The [execution ledger](cpu-proof-case-trees-evidence.csv) groups **652 process
observations**. Three interpretation links append without changing historical
rows. The [CPU parity acceptance criterion](cpu-parity-acceptance.md) remains
unmet.

## Receipt identities

| Receipt | SHA-256 |
| --- | --- |
| `preflight.json` | `75d3dcbfe3d608300ce8d035b25b50245a548c5fd07abeb425d14d53466f1aa5` |
| `applicability/receipt.json` | `949c9aa27b81452d50d45e87f824c7aeebe77faa0c73d9feedc0e83058a99251` |
| `application-preflight-initial.json` | `36d5f4bb42c02bf2af70512ca5c0e8da4e81fd457cf6acaba45817b609c1666c` |
| `application-preflight-affine-adapter.json` | `1e93b67eb03060f2af0dd2eb04b22785649e9d8abc4f1ca5f58d44ec037889bd` |
| `application-preflight-unannotated-adapter.json` | `284174a6b3ffad8cc9f190dd75bb3aa64b0d5564638421378872de16e6ac6686` |
| `application-preflight.json` | `6f6c5def46f3a4f350551d5b523ba4a7fb1b25e9b5dc00919e0f3df45d0943ba` |
| `independent-review.md` | `ef21f78ff12635ed98aa32a221cc16cead030bc0e8838c265d231ae609fecf86` |
| `focused-initial.json` | `f4f1f20ec2d5fbdce1366484a388924e6f86f5b309ce69f04874122ed047a955` |
| `focused-initial.log` | `119ce083db766de0d748fbf16a5114b9f10281a6ec994481db932e1ce9a35a8d` |
| `focused-final.json` | `28df8e40caeb45e166b97198924a70f280c81ab9c7a4ba9d9dc7c4f2f84e7dbe` |
| `focused-final.log` | `04c9370052e918ae5840c955fea853c943c33cbd1f7af5eb634dda11de0b35df` |
| `full-gate-final.json` | `75aa7df271ad0628eaed869c48ef3ce7e88639ec08a8eb44e65c3fa1a3947fea` |
| `full-gate-final.log` | `b7090852cdffb7a40584aefcae755a99bdd12660caad5cc13c95d161327d2b4a` |
| `snapshot.json` | `991d266bc6cdd2efb1f5f410299b00fc26e50fa7e37ef54e9622c0025b98eaab` |
| `open-matrix/receipt.json` | `7c021f279f9ca33d44a58d9d58e89c56b1f053ed111b3d0e07f4e437ed43afca` |
| `graphs-matching/receipt.json` | `70f279cd9f2daad68f6e336245e74cf047cb3bd3d6432b4655549d83006ea492` |
| `negative-matrix/receipt.json` | `fee1b9068a4616f61e5cef9abdc3a28832034de7858cc9890b04eae92221d2f0` |
| `matrix/receipt.json` | `b6ebdef05c7dcd053dd6d33815f21c7de8ce0e078d97c82eb14a51324f62fc59` |
| `matrix-run.json` | `fee542beae23da27e9943350275fe2f5227e550bb26d94c493fd4cbefe126fc5` |
| `upstream-js-evidence.json` | `9fe0ce2edbe711a1f69cbe3120f9cabb3a029ac88ebbfa584c379e83078f2ade` |
| `audit/receipt.json` | `2334a52435ec5969f1b8ccdbf2946643771f6e432c3ded0c570837515e0066c0` |
| `audit-delta.json` | `b9200974e03b857f5e238646da08d396c08e3b6a71cec6213ac3f53fb9bed3cc` |
| `controls.json` | `f8790c417f57b16a92f84d48e8b979706b1fb488924935278489bb34756f7ae5` |
| `coverage-summary.json` | `6ad01f1e5cc71cd1858987fb3bdd95f9479291ea3fffddb6a953cbc587d2a09e` |
| `tool-versions.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `comparison-library-reclaimed.json` | `4db32f4c733faee1d75e620bdadcdcc1df9ebc896572bc01427c2fa3495a2f95` |
| `next-gap.json` | `31b34226eca31622d34841b2d07f21fc80b7cd57104eb93a256246e0ced8db7d` |
