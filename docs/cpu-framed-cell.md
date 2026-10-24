# Dependent worker and original IO qualification

Tested source: `d1adf2ae5291faa59a30f605982e4b0d069d05f9`. Starting published checkpoint:
`484d109451af00ce89fe508eca75868d1d95a12a`. The evidence checkpoint adds only
documentation. Target remains
[`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
No production defect was demonstrated and no runtime/checker implementation was
changed. Tests add coverage; full CPU parity remains incomplete.

## Contracts and applicability

The [unchanged original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/reg/framed_cell_dep.bend)
checks an erased equality parameter whose type mentions an earlier Nat parameter.
The worker normalizes to `n => e => n`, with complete graph
`Lambda(Lambda(Bound(1)))`; its caller normalizes to `m => m`. The graph therefore
checks which binder is referenced, not just binder spelling.

Four inputs (0, 1, 3, 8) exercise both the worker and its caller: eight closed
applications. Typed adapters import the unchanged original. Unit tests compare
every graph child for both definitions and all eight closed Nat results. Two
negative adapters reject a witness for the wrong Nat and false reflexivity,
with exact upstream expected/observed types and locations. Raw executable-core
normalization is paired with checked adapters; it is not a strict proof API.

The original IO main prints `3\n` with status 0 and empty stderr in native Rust,
port C/JS and upstream C/JS. Strict checking/evaluation refuses IO assumptions;
those refusals do not qualify executable behavior. The two function adapters
normalize to named references, but actual C/JS/native executable generation
refuses their unprintable function types exactly. Their upstream alias graphs
are retained as upstream-only observations, not additional port graph matches.

Upstream's direct IO CLI fails twice under Node with
`TypeError: require is not a function`. The pinned
[execution helper](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1274)
passes `import.meta.require` into the generated IO program. This host limitation
remains unresolved. Generated JS runs successfully without modifying upstream
source or installing Bun. Its success does not erase the direct-host failure.

## Evidence

| Check | Result |
| --- | --- |
| Standard gate | **1109 passed, 0 failed, 49 ignored**; formatting, production strict Clippy, all-feature build and real Node provider pass |
| Focused unit plus integration/aggregate/demand/effect checks | **51 / 0 / 0** |
| New integration target strict Clippy | Pass |
| Graph observations | 24 pinned-oracle captures; executable-core unit tests assert both original graphs and eight full closed results |
| Worker/alias text | 12 exact observations |
| Runtime/interpreter observations | 106 exact status/stdout/stderr matches; **2 unresolved upstream direct-host failures** |
| Generation/build controls | 54 successful controls, with pinned upstream C `-O3` profile |
| Negative/printer contracts | 48 exact refusal observations; refused generation leaves no artifact |
| Strict-proof applicability | 52 IO exclusions, not upstream execution matches |
| Matrix | **298 observations**, zero timeouts; runner completes known-outcome checks, not full parity |
| Ten historical diagnostic/acceptance cells | 40 renewed observations, unchanged exactly in each implementation |
| Positive/hole controls | 14 positive observations pass; 18 retain three erased-hole/TODO disagreements |

Two repetitions agree byte-for-byte for every repeated process group. The eight
upstream function execution cells remain unrun after verified generation refusal;
they are inapplicable printer lanes, not successful executions. No new oracle
build failure, timeout or port mismatch was observed in this family.

The ten renewed cells still comprise six constructor-label differences, three
pattern-binder provenance differences and the `unsafe_many` acceptance difference.
No exception was added. The previous supplementary all-target Clippy run at
`96f4d4759a91c8ba81c04550a238aa476269fcd4` failed on seven observed pre-existing
test lints (and was fail-fast, not exhaustive). It was **not rerun or declared
fixed** here. Focused lint success and the standard gate do not close that result.
The historical 1,302-status audit is retained, not reported as a fresh audit.

Independent review found no blocking concerns. The initial typed-let negative
was a harness syntax error and was replaced by properly declared witnesses;
its exact failure remains private evidence. An initial comparison also confirmed
that strict checking rejects imported IO before executable proof diagnostics.
These exploratory failures did not establish production defects.

## Reproduction and identities

[Fixtures and attribution](../tests/fixtures/cpu-framed-cell/README.md) retain the
original byte-for-byte. SHA-256:
`1be042ae06857025a7be7688cb1d6159caecb5aeacca6221d9214df6d169211d`.
CLI SHA-256: `379a27503d184dc3ed744d46c3db83fd8e7c9ebd73f96b8e08b83024a4b47d2d`.
Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

Use the existing activation script, Rust/Cargo1.96.0, formatter
nightly-2026-05-24, Node24.19.0, Clang19.1.7 and PowerShell7.5.4. Host build
settings are `CARGO_NET_OFFLINE=true`, `BEND_GPU=off`, `CARGO_BUILD_JOBS=4`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`;
the full gate additionally uses `RUST_TEST_NOCAPTURE=1`. Host debug settings do
not change Bend optimization profiles or its work/depth limits.

```sh
pwsh -NoLogo -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --offline --features extended_observability --lib framed_ -- --nocapture
cargo test --locked --offline --features extended_observability \
  --test compiler_framed_cell --test kernel_data_grades --test compiler_data_grades \
  --test kernel_match_contracts --test compiler_match_contracts \
  --test kernel_telescope_capture --test compiler_telescope_capture \
  --test compiler_imported_printer --test compiler_dead_residue \
  --test executable_demand --test runtime_effects -- --nocapture
cargo clippy --test compiler_framed_cell --features extended_observability --locked --offline -- -D warnings
```

Upstream generation uses `node bend2/main.ts SOURCE -o OUT.c|OUT.js`; port
generation uses `teamy-bend compile SOURCE --executable --target c|javascript
--output OUT`. The matrix builds upstream C with configured Clang,
`-std=c11 -O3 SOURCE -lpthread -lm -o EXE`; port C adds
`-Wall -Wextra -Werror -pedantic` and links `-pthread -lm -ldl`.
C execution uses `--threads 1 --gpu off`; JS execution uses Node without an
adapter. Every matrix process is bounded to 30 seconds; full/focused runners to
1,200 seconds. Integration-test children retain their existing 20-second bound.

Disk preflight requires 11 GiB, stops below 10 GiB, and keeps 1 GiB free on the
temporary filesystem. Only inactive rebuildable test caches are reclaimed.
[Compact evidence](cpu-framed-cell-evidence.csv) is in Git. Raw receipts, captures,
CLI snapshots and unique historical failures remain outside Git and require
separate preservation. No upstream/docs, dependency, toolchain, guard, budget or
cancellation policy changed; no mid-normalization cancellation is claimed.

## Remaining coverage

Detailed links now cover **393 originals**; **909 remain
unlinked**: 356 checker negatives, one checking-only module, 15 interpreter-profile
cases and 537 compiled candidates. These are source-profile applicability hints,
not counts of missing runtime tests. This fixture was statically classified as
interpretation-only but has now demonstrated executable C/JS applicability.
Historical/blocked links do not establish parity or close the separate API and
integration inventory in the [acceptance criterion](cpu-parity-acceptance.md).

Earlier imported-constructor JS printer timeouts, dependent-pair C build/two
unrun executions, minimal-IO C failures, F32.read, competing argument failures,
foreign/host/resource gaps and the incomplete 688-vector exploration remain
unwaived. The prior public `DefDecl::completion_parameters` and executable-emitter
Err compatibility caveats remain; external consumers have not been audited.

Next proposed coherent batch: `io/main_forged_io` and `import/base_prelude`,
establishing actual IO identity/printing applicability before bounded constructor
applications. This is source/ledger selection only, with no execution credit.

## Receipt hashes

| Receipt | SHA-256 |
| --- | --- |
| `full-gate.json` | `d6bbace7bbe1fd84ddc8812ddb18503c2f6866af43979fb99504093e1827732b` |
| `focused-unit.json` | `299f6f83b2bcddfcc9118dc44af378f9470bfd87f1f7caad54e072a7f07cc179` |
| `focused-integration.json` | `b0e49519b6b78502486546670b6ba8325634bd5d6e19b4cffafbe839842ea453` |
| `focused-clippy.json` | `f343a584cde4fecdb07638c33ee898bd5508047e9cbbbdddd9e5d917522b4b08` |
| `matrix/receipt.json` | `f664d6088a94214e3f6c5c14eadc29da1cca410eacbfe3e64c80fdb38dc60b2e` |
| `retained-controls.json` | `fc8160e9753278867f9964ebf4abefc01c15b1491dc9d890af551ef6544bbbf1` |
| `controls.json` | `769b276a0bd8b483285f32721fa39a2a3234d3fadfc5cb336605526ff57609bf` |
| `independent-review.json` | `494c84d1e5ff44741237b320d1aec0c8b13caa58c18eaab94c5838f60097954e` |
| `snapshot.json` | `7d2bb9187fcbebc9a70027701c8dc032b52af98c4ec1022372d0dffbf51905d9` |
| `frozen-inputs.json` | `0ba1f97480f2cefcc32f1d6cb44b7d2577376260832771848e8c1ff4c00a12f0` |
| `coverage-summary.json` | `03734e1e7462a11a4de9ef2ac58575f3c62c93fd45739bd6f39c176551589dc2` |
| `matrix.py` | `100a4d493d7eabcf0708dd61ac8348b1c414f915c04c14e9731a5ec94278e5e6` |
