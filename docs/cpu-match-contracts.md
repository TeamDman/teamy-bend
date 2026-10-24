# Completed law parameters and residual match fallbacks

Tested source: `1fce12435a4b714f5be4ca8efcf0825fae7fc8b3`. Starting published checkpoint:
`d33f8cf08509314a5fd2baef54475867346c9d53`. The final evidence commit changes only
documentation. Frozen [upstream target](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599)
remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

## Demonstrated corrections

[Ordinary law completion in the pin](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2476)
sets arity to the actual definition's parameter count. The port instead retained
only the law's leading `for` parameters. Thus `law fallback: D -> Unit` followed
by `def fallback(d): Unit{}` incorrectly had arity zero. The checked argument API
refused one argument, and open normalization expanded `fallback` into a lambda.
The unchanged fixture requires `\{A: Unit{}; fallback}`.

The parser now records actual completion names/IDs separately; the checker
resolves quantities and dependent domains from the full law type using its
existing bounded reduction. Type equality, body checking, duplicate/special
contract checks and source-hole refusal remain. Elided IDs participate in fresh
ID discovery, and checked definitions clear pending completion metadata. Aliases
and computed function types do not acquire a new parse-time refusal. Raw strict
emitters use the actual arity; checked runtime and lowering use canonical binders.

Correct arity exposed a second defect: an anonymous match tried to consume its
bare named fallback, losing the residual constructor before saturation. The
[pinned MAT transition](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2978)
applies that constructor to the fallback. The port now does so, while retaining
the complete original stuck graph for literal/chained empty matches. This is a
reduction correction, not a printer substitution. Unselected fallback demand,
closed values, sharing and the approved runtime demand policy stay covered.

Adding public `DefDecl::completion_parameters` is a **Rust source-compatibility
break for external struct literals**. Use `None` for ordinary fully typed
construction; parsed law completions carry the source list. Raw source clients
must use `declared_arity()` rather than assuming `parameters.len()` is actual
completion arity. Checked engine metadata is canonical. Repository constructors
and strict emitters are updated; external consumers have not been audited.

## Applicability and bounded observations

Three [unchanged originals](../tests/fixtures/cpu-match-contracts) retain their
Apache-2.0 attribution; new adapters/tests use MPL-2.0. Original checking and pure
normalization succeed. Original executable generation refuses missing Base;
Base-importing main adapters generate separately. Artifact creation is not a
runtime pass.

| Original | Bounded calls | Observation |
| --- | --- | --- |
| computed_match_specialize | Six valid proofs, four `dbl` calls | Variable/computed hypothesis specialization, worker inputs 0/1/2/4; no false witnesses |
| cop_bare_trio_000 | Four `addself`, sixteen `Nat.add` pairs | Data-kind reusable binders and recursive fields over 0/1/2/4 |
| fits_residual_fallback | `fallback` and `matcher` on A/B | Named residual fallback, explicit argument versus nullary returned function |

Eleven original definition graphs retain every binder, constructor field, match
arm and named call. Thirty-four typed source callers compare complete results;
32 also use the declared-entry argument API. `matcher` actually has zero declared
parameters and returns a function, so its two calls use typed callers rather than
misstating API arity. Six closed proof programs, three complete constructor lists,
three scalar projections and three original-main adapters form **15 distinct
runtime mains**. They are not 34 separately compiled executions.

Scalar outputs are `[0, 2, 4, 8]`,
`[0, 1, 3, 7, 0, 1, 2, 4, 1, 2, 3, 5, 2, 3, 4, 6, 4, 5, 6, 8]`, and
`[0, 0, 0, 0]`. Four negative sources retain false variable/computed witnesses,
a false reusable-value result and an incomplete residual match. Their exact
checking diagnostics remain distinct from phase-only observations.

[Kernel family tests](../tests/kernel_match_contracts.rs),
[runtime family tests](../tests/compiler_match_contracts.rs) and
[completion/reducer controls](../src/kernel/law_completion_tests.rs) preserve
structural/text, application, strict-proof and runtime distinctions. The 11 new
unit controls exercise dependent renaming, aliases/computed types, erased/Many
quantities, partial/nullary declarations, false bodies/holes, forged and special
contract metadata, fresh-ID exhaustion, exact work boundaries, unchanged depth
limits, unselected demand and chained empty-match graph reconstruction. Six
positive unit source programs also pass the pinned upstream checker.

## Current validation and unresolved lanes

| Check | Result |
| --- | --- |
| Standard gate | 1094 passed / 0 failed / 49 ignored; 560.123 seconds |
| Focused regression set | 43 / 0 / 0; 77.621 seconds |
| Complete normalized graphs | 216 observations, two runs per upstream/port lane |
| Normalization/applicability/text | 132 observations |
| Negative contracts | 64 exact check/run/build refusals; eight phase-only probes |
| Runtime matrix | 174 exact executions; 90 successful generation/builds; 6 JS timeouts |
| Current C build failures / unrun cells | 0 / 0 |
| Full checking audit | 1,302 unchanged statuses/streams, including 439 rejecting negatives |
| Additional controls | 14 positive passes; 18 observations retain three hole/TODO differences |

The [execution ledger](cpu-match-contracts-evidence.csv) groups **690 actual
process observations**. All applicable port lanes pass. Repeated exact results
include status and separate stdout/stderr bytes. External runtime lanes are
upstream interpreter, port pure CLI, upstream/port C at `-O3`, and upstream/port
Node JS. Each main runs twice. Native library execution is separately exercised
by repository tests. Upstream C uses the actual CLI profile; port C additionally
uses strict warning flags.

Known imported-constructor upstream-JS printer failures retain their separate
three-second diagnostic timeout: the complete constructor-list mains for
`computed_match_specialize` and `cop_bare_trio_000`, plus the original-main adapter
for `cop_bare_trio_000`, time out twice each. The other twelve mains agree across
all six external lanes. A timeout is host termination, not a Bend exit
status or successful execution. Current generated failing JS and exact streams
are preserved privately. Runtime runner exit
`1` and
`full_required_matrix_passed=false`
remain explicit. Earlier [dependent-pair](cpu-dependent-proofs.md) C failures for
undefined `CID____DEPENDENT_PAIR_U` and **two unrun upstream-C executions** are
historical, still unresolved, and not counted as rerun success here. Existing
minimal-IO C, foreign/native FFI, erased-hole/TODO, F32.read, failure-order,
host/resource/API gaps and incomplete 688-vector exploration remain unwaived.
External executable-emitter compatibility caveats also remain.

## Provenance, commands and storage

Initial clean `656b3eb058e244181c18011ce39857ee848c2168` focused evidence records
both arity/API refusal and expanded normal form. A later candidate failed the
closed residual aggregate and led to the second correction. Review caught the
empty-match edge case; its first candidate was not promoted as complete. Initial
unit-control mistakes (an invalid erased type parameter and expected diagnostic
wording) and two strict-Clippy length failures are retained as test/build evidence.
The final source above is independently reviewed; the gate and matrices are tied
to that source, not to the historical preparation binary.

The toolchain is Rust/Cargo 1.96.0, formatter nightly-2026-05-24, Node 24.19.0,
Clang 19.1.7 and PowerShell 7.5.4. Temporary host build settings are
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
CARGO_BUILD_JOBS=4`, with `CARGO_NET_OFFLINE=true BEND_GPU=off`.
These host settings do not change Bend CPU optimization profiles or budgets.

```sh
pwsh -NoLogo -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --features extended_observability --test kernel_match_contracts --test compiler_match_contracts --test kernel_telescope_capture --test compiler_telescope_capture --test compiler_imported_printer --test compiler_dead_residue --test executable_demand --test runtime_effects --test kernel_open_closures --test compiler_blocked_named_calls -- --nocapture
cargo test --locked --features extended_observability --lib law_completion -- --nocapture
node "$UPSTREAM/bend2/main.ts" "$FIXTURE"
node "$UPSTREAM/bend2/main.ts" "$FIXTURE" -o "$ARTIFACT.c"
clang-19 -std=c11 -O3 "$ARTIFACT.c" -lm -lpthread -o "$EXECUTABLE"
"$EXECUTABLE" --threads 1 --gpu off
node "$ARTIFACT.js"
```

`RUST_TEST_NOCAPTURE=1` is recorded for the full gate. Private runners retain
complete per-process commands, source/artifact hashes, status, separate streams,
repetitions, timeout bounds and source identity. Full/focused build monitoring
has a 1,200-second bound, prebuild reserve 11 GiB, stop floor 10 GiB and temporary
filesystem floor 1 GiB. Only inactive rebuildable caches are reclaimed after
capture; unique historical failures and source remain. The full gate started
with **11.75 GiB free**, bottomed at **10.69 GiB**, and did not hit the stop floor.
Reclaiming 139 inactive first-party executables and unused library/build
artifacts restored **11.81 GiB free**. Compiled first-party artifacts caused the
transient growth; source and unique evidence were not reclaimed. Kernel 2,000,000-step and
128-level bounds are unchanged. Cancellation remains at existing entry/effect
boundaries; no mid-normalization cancellation is claimed. The 49 standard ignores
remain unrun, including hardware/profiler-dependent checks.

The reviewed source, compact fixtures/expectations and reports are preserved in
Git. Ignored raw logs, full process receipts, compressed binaries and generated
oracle failures remain local; **Git publication does not back them up**.

| Private receipt or runner | SHA-256 |
| --- | --- |
| `tool-versions.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `snapshot.json` | `8984a684e0b11fb082ff4de6b0f0a04a46ef5a66c18215f8aaea50b5a874a226` |
| `independent-review.json` | `964c1f80ea3a6362b1cca2f2407fc9345275cc2602d9be66b7e6524784b26c14` |
| `full-gate-final.json` | `8395eac3576ae5422d62336a3e4e69396a17aad0a01098f57b375fcc534f55d5` |
| `focused-clean.json` | `664acd5d04c12a6d95ceec7235fc61d12eeb54967b7dcb432dd71d6cdb32249c` |
| `graphs-matching/receipt.json` | `86d71139396f0d5c4f89353efa70ca50616894655f2f33d6951812ba4b9975ea` |
| `open-matrix/receipt.json` | `d0d7c5b855f02139854d11b7d0f10f428e6d96cb650687edd1fbcd85a002a15e` |
| `negative-matrix/receipt.json` | `ee04c62e7878ac89efdaa3d3dfdd8c0d42b97408ce23f915a21c8321db5421b2` |
| `matrix/receipt.json` | `07f38b7566fb05e57cc66fcb6ea5425a0859319d6e47a667ef39b4b8cbdad25d` |
| `audit/receipt.json` | `fe0511f4552708194e203a71db3a5d1a4a2a34270df4a1eec85b3e7b84e164fd` |
| `audit-delta.json` | `5cec5604c29b61f0f20118813a2fb4db87f8924905557427a19599fe593a1e03` |
| `controls.json` | `aadcafa1e5533028cb5bccb4fc1b7ef6561fed2aa270a374c13d72dc1b899f3a` |
| `coverage-summary.json` | `75551a0fcb4bf423942324ca69322037dc519aefbf6fa4407ea32973c4803c44` |
| `law-controls-final.json` | `8546f5d70467a7ee135c914d3b9d5fcaf49afe99f6e5554f55c93f8739b8e3ec` |
| `run-graphs-matching.py` | `6f3aad1f45ce58869be79be5419658a6d4a1003bf24fee90d22a1c225683f314` |
| `open-matrix.py` | `5b3d96a31aa6d46d543f1928553c568151bc00320b61ff2f4344de0e1872f7fa` |
| `negative-matrix.py` | `27a9c9b981c7f27dd878e482bc20a7be0c22a44f68fbd96d8e45572d1b1630dd` |
| `runtime-matrix.py` | `0ad741612ace87337afbcd03a7afefaa186d82474648d24e3f2a902c891cf905` |

Detailed receipts link **390 originals**; **912 remain unlinked**: **357 checker
negatives, one checking-only module, 17 interpreter cases and 537 compiled
candidates**. These are applicability categories, not missing runtime-test counts.
Links include historical, partial and blocked evidence and do not establish full
CPU parity or close the API/integration inventory.

Next proposed substantive batch: `grade/accept_data`, with applicability-led
checks of reusable Nat/list/pair/equality values, phantom tags, affine function
lists and weakening. Empty-type elimination must remain a normalization/checking
observation without fabricating an inhabitant. This is a proposal, not executed
coverage. `reg/framed_cell_dep` is a later dependent-worker/IO control.
