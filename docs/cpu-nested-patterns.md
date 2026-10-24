# Nested patterns and fallback reconstruction

Tested source **`062c7ec150168922ff642d07b1fc72ce5c14144c`**, following checkpoint
`f85d86655e017019c5cfe75e6cd96583023326a5`, qualifies fourteen unchanged
interpreter originals against **`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`**.
The four requested nested multi-scrutinee cases and ten adjacent cases all agree.
No production defect was demonstrated and no production source changed. The
commit adds regression fixtures, concrete application observations and a runner
selection; dependencies, toolchains, guards, budgets and policies are unchanged.
Full CPU parity remains incomplete.

## Scope and actual applicability

| Original family | Count | Qualified mechanism |
| --- | ---: | --- |
| flatten/nested_multi_scrutinee_000 through _003 | 4 | Nested pattern columns, binder reuse, priority and reconstructed fallbacks |
| flatten/nested_ctor_000 through _004 | 5 | Nested constructors, sibling field selection and multi-field patterns |
| flatten/default_rebuild_000 through _004 | 5 | Fallback reconstruction after partial Nat/List matching |

All fourteen were previously unlinked interpreter cases in the applicability ledger, not
missing generated-runtime cases. They preserve the pinned
[original flatten fixtures](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten)
byte-for-byte, including comments and golden output. The tests retain upstream
Apache-2.0 attribution. Actual pinned execution verifies each golden result;
port checking, typed strict evaluation, native pure output and actual normalized
core/text all match.

Their mains return functions. Original C/JS generation refuses the Base/IO
prerequisite upstream; the port refuses the unprintable function result. These
are different diagnostic causes, so no exact original build-diagnostic parity
is claimed. Separate authored probes add Base and prefix local names without
changing upstream originals. Both pinned C and JS emitters then refuse the
function printer, twice: **56 refusals**, with no artifacts. The pinned
[show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1203)
requires that printer before generating a standalone program. No generated
execution, new semantic exception or host-blocker claim is inferred.

## Concrete application evidence

The direct pinned core adapter checks each original and applies constructor
arguments to its actual main definition. **476 distinct vectors**, observed
twice, yield 952 exact result observations:

- 100 vectors for the four two-column Nat functions, using every pair of depths
  zero through four;
- 49 nested-constructor vectors, including every BitPair, three-Color Triple
  and three-Flag Cfg combination, plus the five Nat depths;
- 327 fallback vectors: five Nat depths for three functions and all 156 lists
  of lengths zero through three over those depths for each of two list functions.

The committed TSV drives direct Rust checked-definition applications; each
asserts both exact text and actual core structure. This includes reconstructed
values in fallback branches, rather than checking only function formatting.
The finite domains fully enumerate the specified Bit/Color/Flag combinations,
but they do not prove arbitrary Nat/List equivalence. No expected result is
computed by the port under test.

Oracle-only application/printer probes were captured at the clean starting
checkpoint against the unchanged pin. Their receipt and adapter/source hashes
remain separate from the final clean-source gate. The initial 140-observation
comparison used the previously tested binary (source 02b244e); it is retained
as initial evidence, not relabeled as the final build.

## Validation at the exact source

| Check | Result |
| --- | --- |
| Standard format/strict Clippy/build/provider/test gate | **883 passed, 0 failed, 49 ignored**; 98 result-bearing targets; exit 0 |
| Focused candidate controls | 47 passed; included again in the clean full gate |
| Exact normalization suite | 90 cases, 1,008 observations: 714 successful, 288 artifact/printer refusals, six strict F32 exclusions |
| No-main checking/compilation matrix | 306 observations, including 72 exact refusal diagnostics with no artifacts |
| Rewrite diagnostics and phase controls | 60 observations pass, including imported and nested origins |
| Earlier false-list/brace and positive controls | 144 observations pass |
| Serial upstream C/JS and port native/C/JS baseline | 50 exact executions, two repetitions, pinned upstream O3 |
| Corpus checking-status audit | 1,302 rows, zero transitions from 02b244e |

The reusable `compare-normalization.ps1 -Suite nested-patterns` selects the
fourteen originals. `-Suite all` now contains 90 cases: 83 originals and seven
authored controls. All required structure/text comparisons match; no tolerance,
structural relaxation or output repair was added. Every repeated final observation
is deterministic, with no timeout or abnormal exit. Expected refusals are
classified separately from successful execution.

All 439 expected checker negatives still reject. Other status results remain
366 accepted / 497 rejected; matching status is not full parity. The 49
hardware/profiler ignores were not run. Existing no-main stderr, false-fusion,
constructor-origin, cancellation and guard regressions remain in the full gate.
Only the named diagnostic matrices were renewed; other historical matrices
retain their actual commit identities. External CLI exit-code/public error
compatibility caveats remain as documented in the
[preceding report](cpu-compile-rewrite-diagnostics.md).

## Coverage and next work

| Category | Inventory | Detailed originals | Without detailed receipt |
| --- | ---: | ---: | ---: |
| Checker negative | 439 | 8 | 431 |
| Checking only | 12 | 11 | 1 |
| Interpreter only | 164 | 56 | 108 |
| Compiled candidate | 687 | 115 | 572 |
| Total | 1,302 | 190 | 1,112 |

Detailed links include historical and unresolved cells. They are not universal
passes or a count of missing CPU runtime tests. Runtime/interpretation union is
171. This source renews 83 interpretation rows, ten checking rows and three
runtime originals; nine checking and 85 runtime rows remain historical. Authored
controls and argument vectors add no inventory rows. The sole unlinked
checking-only module is still io/far_types, requiring foreign applicability review.

The [acceptance criterion](cpu-parity-acceptance.md) is unchanged. F32.read,
competing-error order, whole-program resource limits, native foreign support,
unavailable Bun/host oracle lanes and complete API/lane review remain open.
No newly qualified cell resolves or waives those requirements.

Next independent family: `field_match_in_order` and its `_f`/`_g` companions,
then the adjacent destructuring-let cases. That family combines pure functions
with an IO main, so it needs actual printer/runtime applicability. The IO main's
existing strict-check refusal (`undefined name IO`) is proof-mode status evidence,
not proof of a runtime defect. Its required execution lanes remain unqualified.

## Reproduction, resources and provenance

After activation of the frozen environment, the recorded commands include:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_normalization --test cpu_no_main --test cli_checker_diagnostics --test cli_compile_executable --quiet
cargo build --locked --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-directory> -Suite baseline -Repetitions 2
python3 <private-evidence>/applications.py
python3 <private-evidence>/qualify.py --output <fresh-name> --snapshot final-snapshot
python3 <private-evidence>/diagnostics.py
python3 <private-evidence>/proof-regression.py
python3 <private-evidence>/audit.py
python3 <private-evidence>/verify.py
```

The private cpu-nested-patterns evidence directory holds exact argv, source and
binary hashes, raw stdout/stderr, statuses, timings, repeated observations and
cache receipts. Fixed-output scripts require fresh destinations on rerun.
Child limits are 30/60 seconds; the outer runner has a 20-minute deadline and
unchanged 2-GiB workspace / 1-GiB tmp stop floors. No source or raw private
capture was placed in the reference or public documentation.

Every build preflight met the 10-GiB reserve. Full-gate minimum headroom was
**9.33 GiB**. Repeated binary snapshots are retained as evidence;
inactive root artifacts from an older host build profile and rebuildable test
executables were reclaimed with protected-binary hash checks. Test-executable
cleanup alone left the next-build reserve short; post-validation cleanup of
root library/metadata caches and an alternate example restored it. Protected
CLI/probe/provider binaries and every evidence snapshot remain. Later builds
will rebuild the removed root cache under the same frozen settings. Final headroom is
**10.01 GiB**. Sources, lockfile, evidence and historical checkpoints survive.
Four build jobs, debug=0 and disabled incremental caching are recorded host
build controls, not Bend runtime optimization or safety-budget changes.

Rust 1.96.0, nightly-2026-05-24 formatting, Node 24.19.0, PowerShell 7.5.4 and
Clang 19.1.7 remain frozen. The runtime baseline uses upstream's O3 profile,
-lpthread -lm and `--threads 1 --gpu off`; port C retains strict C11 warnings
and -pthread -lm -ldl. No packages were installed, branches pushed, dependencies
adopted, or network/security settings changed. No desktop or GPU work occurred.

| Evidence | SHA256 |
| --- | --- |
| no-main-final/receipt.json | c9008e9f410372721b15931628cd56fcc45546398a43d9c558e0b8de1fc3f6e1 |
| normalization/receipt.json | 3bb50275bc2e796c08593aeb7fabeafda9891dd71794e30af574e780718614dc |
| applications/receipt.json | 46507e420b0a8bb783fc1e314452ae07761c952e09e092ae50a71289668c070a |
| audit/receipt.json | 37d4c7db5f52189efe89ac004d35d9fd2d985259cf2d59c04ee7920be1228c02 |
| baseline/receipt.json | a343fbb1415f3117f5ce97e7944e2bb62464b7ec21148399aa148f707a7c8561 |
| full-gate.log | 56ba088b4d598d83090707a97cfec0059c7fb433e35e21060bcb4abaf9b892c2 |
| full-gate.json | 74ada9db8a53c456da2734e6b9edecf13a2d5821e752bc561727732b8d5d8942 |
| diagnostics/receipt.json | e24063ea5816d7474dd0364ebaac2692104f4c77a12d2bf29ee5088e9efd87a5 |
| proof-regression/receipt.json | f314dbb15ad3287fdec3dd116887586859427c799cc564f1e022aee875aa08fe |
| oracle-provenance.json | ed8fec9c6deaaba4fb9772138b579c903a5bdc807652476ccb52ebb89e2e035e |
| pre-gate-cache.json | 6cef22293fe01373451616560a4adfb047a205b0f63e5551ca88b613f7a7e0da |
| stale-profile-cache.json | ef09477cffe3fb0ce7544c3e08e52eb30990d68cb6f91fc611c2f6a8f80e6398 |
| final-cache-preflight.json | d24522b5fe5c3ad703c17f92830a900455241e96785cb735e7498a9096f191ca |
| root-cache-cleanup.json | dd39b380aff1ff4cdf6a430030cbfbfd596d079eefec9e3e2e0858e20d158971 |
| final-cache-complete.json | 45330eb53758a5763041e2189f9ab8fc8b8bb86ec18a9706c988145b21bb2cff |

Tested binary SHA256: `f8200bb22001badc8a3b12be230277764770f5c53c3690c24ec0331cae59e931`.
Normalization probe SHA256: `4b27a16243f30f6c954af3987633d4a0b40ef8e6b04700049793400c2b6642a1`.
Cargo.lock: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Upstream and teamy-docs `50e89d6897ce2a65167ef65b17c5fa83fe849158` remain clean.
The later documentation checkpoint reports this tested source and does not claim
a second gate execution. No task-owned test remains running.
