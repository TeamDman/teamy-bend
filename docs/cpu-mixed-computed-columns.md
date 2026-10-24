# Mixed computed columns: bounded CPU checkpoint

Tested source: `b382a7683d2be3b06033cc4e6236595baed689f2`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The [eight originals](../tests/fixtures/cpu-mixed-computed-columns/README.md)
preserve upstream bytes and Apache attribution. New wrappers/tests are MPL-2.0.
**No production defect was demonstrated and no implementation change was needed.**
CPU parity remains incomplete.

## Bounded inputs and exact results

| Original suffix | Grid | Exact scalar observation |
| --- | --- | --- |
| `_000` | W depth 0..3 × Nat depth 0..2 | `[1, 2, 2, 3, 4, 4, 1, 2, 2, 3, 4, 4]` |
| `_000_par` | W depths 0..5 | `[0, 1, 0, 1, 0, 1]` |
| `_000_sel` | W depths 0..2 × Nat depths 0..2 | `[1, 2, 2, 3, 4, 4, 3, 4, 4]` |
| `_000_walk` | Nat depths 0..2 × W depths 0..2 | `[1, 2, 3, 2, 3, 4, 3, 4, 5]` |
| `_001` | (O,I) cubed | `[2, 3, 0, 1, 6, 7, 4, 5]` |
| `_001_flip` | O, I | `[1, 0]` |
| `_001_sel` | (O,I) cubed | `[0, 1, 2, 3, 4, 5, 6, 7]` |
| `_002` | Nt depths 0..2 × W depths 0..2 | `[1, 2, 3, 2, 3, 4, 3, 4, 5]` |

Products are lexicographic, rightmost argument fastest. R constructor suffixes
map to their numeric ID; W maps to its full returned depth; Bit maps O/I to 0/1.
The [63 constructor-valued results](../tests/fixtures/cpu-mixed-computed-columns/applications.tsv)
are independently checked against the pinned interpreter and port checked core.
This covers all Bit combinations for the selected functions and bounded recursive
constructor inputs, not arbitrary depths or competing-error order.

The pinned application adapter checks the unchanged book, applies constructors,
and uses `term_snf`, `term_lower`, `term_show`; two runs produce 126 exact values.
The Rust regression checks all 63 values and actual core structure, plus each
function normal form against its oracle golden. CLI pure `run` text and JSON
`eval` values also match. No separate access to the CLI writer's core is claimed.

Pure wrappers observe the same arguments as lists. IO adapters print those lists
and assert `ExecutableEntry::Io` in the regression. Pure `run_main` exercises
normalization; IO exercises the native executable driver. Native IO and generated
upstream/port C and JS agree on exact stdout, empty stderr and exit 0. Observing
constructor IDs and full depths leaves imported-name printing unqualified.

## Actual applicability

All eight originals accept checking and are interpreter-only under the pinned
gate. Their mains are functions and they lack Base. Compared emitters refuse
with exit 1, empty stdout, `Error: a build needs import Base\n`, and no artifact.
Separate Base-enabled function adapters accept upstream checking, then reach the
unprintable-main refusal. Upstream C/JS and port C/JS/native match its complete
streams/status and artifact absence. Type names follow each adapter's import
spelling; exact diagnostic bytes are in the private receipt. No original receives
an executable-runtime pass from these refusals or the observation wrappers.

Direct upstream IO under installed Node returns exit 1, empty stdout and
`TypeError: require is not a function\n` because `import.meta.require` is absent.
Sixteen observations remain **host-blocked**, with no Bun or substitute installed.
Eight IO adapters are excluded from strict proof checking; native IO runs and
emission establish executable acceptance separately. Neither category is counted
as a runtime success.

## Validation and reproduction

| Check at tested source | Result |
| --- | --- |
| Standard gate | 942 passed, 0 failed, 49 ignored; 106 result-bearing targets |
| Format / strict Clippy / all-feature build / TCP-UDP provider | Passed |
| Original normalization | 48 process observations plus checked-core regression |
| Direct oracle applications | 16 invocations, 126 values, 63 distinct fixture/input pairs |
| Original / Base-function refusal | 160 exact invocations; artifact absence checked |
| Accepted-source probes | 32, distinct from emission |
| Pure output / strict eval | 96 / 16 exact observations |
| Native IO and generated IO output | 80 exact observations |
| Direct IO oracle / strict IO proof | 16 host blocks / 8 exclusions |
| Generation / C compilation | 96 successful observations |
| Matrix total | 568; result lanes repeated twice, no timeouts or nondeterminism |
| Checking-status audit | 1,302 unchanged: 439 expected negatives rejected; others 366 accepted / 497 rejected |
| Positive / hole controls | 14 pass; 18 observations retain three unresolved cases |

The 49 ignores remain unrun: 47 hardware tests and two optional profilers. The
[kernel](../tests/kernel_mixed_computed_columns.rs) and
[compiler](../tests/compiler_mixed_computed_columns.rs) modules add three focused
tests, executed within the full gate. Existing proof, guard, cancellation and CLI
failure-boundary tests also ran. No redundant separate Cargo test run is claimed.
The gate took 306.53 seconds from clean source; all final external
matrices and controls use its captured binary.

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

The primary C matrix uses the pinned Linux [O3 profile](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253).
The Rust integration helper separately uses strict C flags with no explicit
optimization switch. Host Rust debug overrides do not change Bend optimization
or budgets. Generated JS runs under Node; port emission selects `--target javascript`.
Private receipts retain exact commands, streams, statuses and source/artifact
hashes; the [public CSV](cpu-mixed-computed-columns-evidence.csv) keeps compact
identities. Bounds remain 30 seconds per matrix child, 60 per audit child, 20 per
compiled test child and 1,200 for the gate.

## Storage and remaining work

Before the build, two identical inactive comparison executables were coalesced,
recovering 116 MiB while retaining both paths, hashes, mode, ownership and mtime.
The preflight required 11 GiB, one GiB above the 10 GiB reserve. The gate began
with 11.127 GiB and never fell below
10.477 GiB. After capturing a verified binary and lossless
archive, 102 inactive rebuildable test executables were retired,
recovering 0.650 GiB before the external
C matrix. Source and unique historical evidence remain. Current headroom is
11.102 GiB. Recheck before further builds; stop floors
remain 2 GiB workspace / 1 GiB temporary storage.

Source `b382a7683d2be3b06033cc4e6236595baed689f2` qualifies eight unchanged `flatten/mixed_computed_column_000..002`
originals and companions with **63 bounded applications**. The grid covers parity
recursion, value-column threading, full returned constructor depth, and all Bit
combinations for computed middle arguments. Normalized text, actual checked core,
concrete values, strict-proof applicability and native IO execution remain distinct.
No production defect was demonstrated or implementation change needed.
[Report](cpu-mixed-computed-columns.md).

The clean-source gate passed **942 tests, zero failed, 49 ignored**. The repeated
matrix records **568 process observations**, including 16 direct Node IO host
blocks and eight strict-proof exclusions. All 1,302 checking statuses are unchanged;
14 positive controls pass and 18 observations retain three unwaived hole/TODO
cases. Original missing-Base and separate function-printer refusals match exactly,
with no artifact; wrappers do not supply executable passes for original functions.

Detailed receipts now link **307 originals**; **995** lack detailed receipts:
**360 checker negatives, one checking-only module, 81 interpreter cases and 553
compiled candidates**. Eight interpretation rows are new. The prior 116
interpretation, 93 runtime and 90 checking rows keep their actual tested identities.
Linked receipts include partial/blocked cells and do not establish CPU parity.
Imported-constructor spelling and upstream JS timeout evidence remain separately
open at the earlier source. Erased-hole/TODO, host/resource/foreign/F32.read,
runtime-error-order, external API and unfinished 688-vector requirements remain
unwaived. No new semantic exception or resource policy was adopted.

Next independent batch: `mixed_computed_column_003`, `_id` and `_sel`, proposed
39 vectors. These import Base and are compiled candidates in the inventory;
actual function quantity/printability must be established, preserving the `+b`
reuse annotation. Only source/applicability review is complete, with no execution
credit. Dependencies, references, toolchains, guards, budgets, cancellation and
approved demand semantics remain frozen. No installs, pushes, desktop or GPU work.

The [imported-printer evidence](cpu-ragged-rows.md#open-imported-constructor-printer-discrepancy)
remains at source `623c6bacfc82993e731c4c124306924c1d68d8eb`: interpreter/upstream C/
pure native output uses `helper.Box{7}`, port C/JS uses `Box{7}`, and pinned JS
reached the three-second bound. The larger 30-second timeout is also preserved.
This batch neither reran nor relabeled those results. The
[acceptance criterion](cpu-parity-acceptance.md) still requires all applicable
lanes, APIs and failure/resource contracts, not simply passing counts.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `c8a205fb5f97adee080e1f118a24a3c9c712b1193cb82028b29f81a27496e684` |
| snapshot.json | `dff9781563719ac6adb7bb5907c73a810ceeece8b6f4e3d8419e14483907e8a2` |
| snapshot/teamy-bend.gz | `7dad672652accabe1eb9c0bcb7ba528d2aa102ebbe0ff64d296bd46eb454a6db` |
| full-gate.json | `235daf5d50e416f00809942ee75592307dbc5bce165942ee79ae57c02b294acb` |
| full-gate.log | `fd8c9fd8cad62b5cad6f238c446a1bf2309e33bf5305cd9fc7f8ca15a2e0caec` |
| qualified/receipt.json | `33dbc12fd561068cb1f70f26f88332b0ca125dd95d93d484ab9220acd60fa80b` |
| audit/receipt.json | `0b2647c6970d9c7f4e18a0f4a673235feded1d6cffa88f46041fd8a4313f670d` |
| controls.json | `7f9e6a9143c2db7d6b60b1e0ba1146b1cb0d4e1d2f2ee9947cc2701e89aecbfc` |
| coverage-summary.json | `4edd4dacf2debe4db1808340d23ea9f09205638e4bd3a4f8beb0e4935db477e0` |
| duplicate-cache-coalesced.json | `b8ccf0d4666cc277fcc4b22f49db9f9d020858eadcd068291b0d18f5876a3b88` |
| inactive-test-cache-reclaimed.json | `7ca146c907193fc863132caed4d5a61b606c7b31fc6971307ccc09c809e80021` |
| qualify.py | `07e3ec5a23c76f0e0a784c6e0afb5b326c682d2f8719589d0390d15d6d547b25` |
