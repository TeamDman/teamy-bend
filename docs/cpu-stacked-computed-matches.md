# Stacked computed matches: bounded CPU checkpoint

Tested source: `63f4e30c7764a2cb72a78323e8aecc545dad4aa4`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The [six original fixtures](../tests/fixtures/cpu-stacked-computed-matches/README.md)
retain exact upstream bytes and Apache attribution. New wrappers and regression
code use MPL-2.0. **No production defect was demonstrated and no implementation
change was needed.** CPU parity remains incomplete.

## Inputs and results

| Original suffix | Grid | Exact observed list |
| --- | --- | --- |
| `_000` | W depths 0..2 × 0..2 | `[0, 1, 1, 2, 3, 3, 2, 3, 3]` |
| `_000_nat` | W depths 0..2 | `[0, 1, 1]` |
| `_000_sel` | Nt depths 0..2 × 0..2 | `[0, 1, 1, 2, 3, 3, 2, 3, 3]` |
| `_001` | `(T,F)` cubed | `[4, 3, 4, 3, 2, 2, 1, 1]` |
| `_001_not` | `T`, `F` | `[0, 1]` |
| `_001_sel` | `(T,F)` cubed | `[1, 1, 2, 2, 3, 4, 3, 4]` |

Products are lexicographic, with the rightmost argument varying fastest. R values
map `Kn` to n; Nt maps to its full constructor depth; Bl maps T to 1 and F to 0.
The 39 exact constructor-valued results are retained in
[applications.tsv](../tests/fixtures/cpu-stacked-computed-matches/applications.tsv).
These cover all Boolean values for the selected functions and a bounded part of
the infinite constructor domains. They do not establish arbitrary-depth or
competing-failure-order behavior.

The pinned direct-application adapter loads/checks the unchanged book, applies
constructor arguments and uses `term_snf`, `term_lower`, `term_show`. Two runs
produce 78 oracle values. The Rust regression checks all 39 constructor results
and actual core structure, plus each complete normalized function against its
upstream golden form. CLI `run` text and JSON `eval` values agree; no separate
observation of the CLI writer's internal core is claimed.

Pure typed wrappers observe the same applications as lists. IO adapters print
those lists and assert `ExecutableEntry::Io` in the Rust regression. Pure
`run_main` exercises normalization; IO exercises the native executable driver.
Native IO and generated upstream/port C and JS agree on exact stdout, empty stderr
and exit 0. Constructor projection leaves the imported-name printer contract
unqualified; it does not repair or waive that separate discrepancy.

## Applicability and failure boundaries

The six originals are checking/interpretation fixtures under the pinned gate.
Their function-valued mains lack Base. All compared emitters refuse their builds
with exit 1, empty stdout and `Error: a build needs import Base\n`, without an
artifact. Separate Base-enabled adapters return the original function, accept
upstream checking, and reach the actual unprintable-main refusal. Both upstream
C/JS and port C/JS/native emitters agree on exact streams/status and artifact
absence. Imported type spelling in that diagnostic follows the adapter's import
path; private receipts retain the exact bytes. Refusal is not runtime execution.

Direct upstream IO under installed Node returns exit 1 with empty stdout and
`TypeError: require is not a function\n`, because `import.meta.require` is absent.
Twelve observations remain host-blocked. No replacement host or Bun was installed.
Six IO adapters are excluded by strict proof checking; executable acceptance is
separately established by the native IO run and emission. Neither exclusion nor
host failure is converted to a parity pass.

## Validation

| Check at the tested source | Result |
| --- | --- |
| Standard gate | 939 passed, 0 failed, 49 ignored; 104 result-bearing targets |
| Format / strict Clippy / all-feature build / TCP-UDP provider | Passed |
| Original normalization | 36 process observations; checked-core regression also passes |
| Direct upstream applications | 12 invocations; 78 values for 39 distinct fixture/input pairs |
| Original / Base-function refusal | 120 exact invocations; artifact absence checked |
| Accepted-source phase probes | 24, distinct from emission |
| Pure output / strict eval | 72 / 12 exact observations |
| Native IO and generated IO output | 60 exact observations |
| Direct IO oracle / strict IO proof | 12 host blocks / 6 exclusions |
| Generation / C compilation | 72 successful observations |
| Matrix total | 426; result lanes repeated twice, no timeout or nondeterminism |
| Checking-status audit | 1,302 unchanged; 439 expected negatives rejected, others 366 accepted / 497 rejected |
| Positive / hole controls | 14 passed; 18 observations retain three unwaived disagreements |

The 49 ignores remain unrun: 47 hardware tests and two optional profilers.
[Kernel regressions](../tests/kernel_stacked_computed_matches.rs) verify original
normal forms and direct applications; [compiler regressions](../tests/compiler_stacked_computed_matches.rs)
verify both pure and IO profiles. These three focused tests ran within the full
gate, alongside existing proof, guard, cancellation and CLI failure-boundary
controls. No redundant separate Cargo test run is claimed. The gate completed in
302.27 seconds from clean source. All final matrices and controls use
its captured binary. Earlier receipts retain their original source identities.

## Reproduction and evidence

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$BEND_REFERENCE/bend2/main.ts" "$ORIGINAL"
"$BEND_BINARY" run "$ORIGINAL"
"$BEND_BINARY" --output-format json eval "$ORIGINAL"
node "$BEND_REFERENCE/bend2/main.ts" "$WRAPPER" -o "$WORK/up.c"
clang-19 -std=c11 -O3 "$WORK/up.c" -lpthread -lm -o "$WORK/up"
"$WORK/up" --threads 1 --gpu off
"$BEND_BINARY" compile "$WRAPPER" --executable --target c --output "$WORK/port.c"
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$WORK/port.c" -pthread -lm -ldl -o "$WORK/port"
"$WORK/port" --threads 1 --gpu off
```

The primary C profile matches the pinned Linux [CLI O3 profile](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253).
The Rust integration helper separately uses strict C flags without an explicit
optimization switch. Host Rust debug settings do not change Bend optimization or
budgets. Generated JS uses Node; port generation selects `--target javascript`.
Private receipts retain every exact command, stream, exit, source/artifact hash
and adapter source. The public [evidence CSV](cpu-stacked-computed-matches-evidence.csv)
keeps compact identities. Matrix children have 30-second bounds, audit children
60, compiled Rust-test children 20 and the full gate 1,200; limits were unchanged.

Build preflight required 11 GiB, a full GiB above the 10 GiB reserve. The gate
started with 11.034 GiB and its minimum was
10.399 GiB. After capturing the verified binary and lossless
archive, 100 inactive rebuildable test executables were retired,
recovering 0.635 GiB before the external
C matrix. Source, unique historical evidence, dependencies and toolchains remain.
Current headroom is 11.014 GiB. Recheck before another
build; stop floors remain 2 GiB workspace / 1 GiB temporary storage.

Source `63f4e30c7764a2cb72a78323e8aecc545dad4aa4` qualifies six unchanged `flatten/stacked_computed_match_000/001`
originals and `_nat`/`_not`/`_sel` companions through exact normal forms and **39
bounded applications**. The grid covers constructor depths 0..2 and every Boolean
combination for the three-column functions. Actual checked-core structure, CLI
text, concrete values, pure observation and native IO evidence remain separate.
No production behavior change was needed. [Report](cpu-stacked-computed-matches.md).

The clean-source full gate passed **939 tests, zero failed, 49 ignored**. The
repeated matrix contains **426 process observations**, including 12 direct Node
IO-oracle host blocks and six strict-proof exclusions. All 1,302 checking statuses
are unchanged. Fourteen positive controls passed; 18 observations preserve three
unwaived hole/TODO disagreements. Original missing-Base refusals and Base-enabled
function-printer refusals are exact, with no emitted artifact; neither supplies
an executable pass for the original function-valued main.

Detailed receipts now link **299 originals**, with **1,003** lacking detailed
receipts: **360 checker negatives, one checking-only module, 89 interpreter cases
and 553 compiled candidates**. Six interpretation rows are new; the previous
110 interpretation, 93 runtime and 90 checking rows retain their tested identities.
These counts include partial/blocked cells and do not establish CPU parity.
Imported-constructor printing and upstream JS timeouts remain separately open at
their earlier source. Erased-hole/TODO, host/resource/foreign/F32.read,
runtime-error-order, external API and unfinished 688-vector requirements remain
unwaived. No new exception or resource policy was adopted.

Next independent batch: eight `mixed_computed_column_000..002` originals and
companions, with a proposed 63-vector grid for parity recursion, value-column
threading and computed middle arguments. Only source/applicability review is
complete; no execution credit. Dependencies, references, toolchains, guards,
budgets, cancellation and approved demand semantics remain frozen. No installs,
pushes, desktop or GPU work.

The [imported-printer discrepancy](cpu-ragged-rows.md#open-imported-constructor-printer-discrepancy)
remains at source `623c6bacfc82993e731c4c124306924c1d68d8eb`: interpreter/upstream C/
pure native output uses `helper.Box{7}`, port generated C/JS uses `Box{7}`, and
pinned JS reached its three-second timeout. The larger 30-second timeout is also
preserved. Neither was rerun or relabeled by this batch. The
[acceptance criterion](cpu-parity-acceptance.md) still requires all applicable
lanes, APIs and failure/resource contracts, not simply more linked rows or tests.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `7de16b8ecfdf9eab8b156eedcae9136be6cf3a6f7d00c6d2017ce8a2e9d9db3c` |
| snapshot.json | `ff55ad159e75276315900686019f105976ed3843607ae73bd81a1ebdc0061e98` |
| snapshot/teamy-bend.gz | `ea3b67b6697df8b908a7f3a72ae4776d1d61a61aad079925169032aa44e0f0be` |
| full-gate.json | `4d7123d4183a1e61cdfde318c13dc61da5e03f0eca31c4ee9820a5816d96e0f5` |
| full-gate.log | `1fc9aed7272d541dfea0c49e9826087b9703642d8fe8bffb9d4591f07fa0206f` |
| qualified/receipt.json | `8dc3e70195e1d80637f61d0ea905cc71ed7b5fafbc7b0cfe44f56e768ee6526a` |
| audit/receipt.json | `20b07c7685111b01767b33fc62e83b96a3f65a85a0104d8f38d35cc9239144a5` |
| controls.json | `d21c0dfd3e8038fa663d35173f1c8b0672f7a1c566efd3acda418a1aa83b5177` |
| coverage-summary.json | `1e62e878dd32b5bd982d9be93277e21d81e8192acf36f26cf34a96439302482c` |
| inactive-test-cache-reclaimed.json | `59bf2cf9c67468893e3b0ec35b24ecf52998a284771dc8ef468212ee60beaf48` |
| qualify.py | `7ac0ac6394df4c71a69fe4e65842d2dd08e4d41fa2a039a01caa444e6f552083` |
