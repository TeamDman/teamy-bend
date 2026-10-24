# Qualified names in executable constructor output

Tested source: `936d299d3f6db320b7ae0fcc82c33bfa934bcbbf`. Starting checkpoint:
`4721bf142fdd92d9e0171c1bc370894d6fa7c229`. Target:
[pinned Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).

## Renewed discrepancy and scoped correction

The [historical minimal reproducer](cpu-ragged-rows.md#open-imported-constructor-printer-discrepancy)
is retained byte-for-byte in [fixtures](../tests/fixtures/cpu-imported-printer).
An imported helper declares `Box{value: U32}`; main returns `H.make()`.
The renewed baseline uses tested binary source `7209617`; the starting checkpoint
changes only documentation. Both repetitions reproduce:

| Lane | Before | After |
| --- | --- | --- |
| Pinned interpreter | `helper.Box{7}\n`, exit 0 | Same |
| Upstream C O3 | `helper.Box{7}\n`, exit 0 | Same |
| Port pure CLI/native | `helper.Box{7}\n`, exit 0 | Same |
| Port generated C/JS | `Box{7}\n`, exit 0 | `helper.Box{7}\n`, exit 0 |
| Upstream generated JS | Empty streams; killed at 3 seconds | Same unresolved timeout |

Successful rows have empty stderr. The local non-imported control prints
`Box{7}\n` in all six lanes. A killed process is not a language error exit or
a successful comparison. The earlier larger 30-second timeout remains historical;
no new 30-second completion is claimed for it.

Pinned [show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1195)
specifies the term printer's spelling and puts canonical constructor `c.k` names
in descriptors. Its [C printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L5600)
selects by constructor ID, then prints that canonical name. Only exact names
`Con`, `Nil` and `Tuple` receive list/tuple sugar.

The port incorrectly used declaration-local runtime tags for both matching and
printing. [C generation](../src/compiler/executable_c_values.rs) now uses the
canonical constructor name for output; CID dispatch and field traversal are
unchanged. The [JS descriptor](../src/compiler/executable_javascript.rs) keeps
its local `tag` and adds a separately escaped `name`. The
[JS printer](../src/compiler/executable_core.js) still matches by local tag but
prints and selects bracket sugar by canonical name. Imported `helper.Con`,
`helper.Nil` and `helper.Tuple` therefore print as ordinary qualified constructors.
Base lists, tuples and booleans retain their existing spelling.

Runtime/foreign tags, CIDs, foreign aliases, layouts, fields, evaluation order,
sharing and public APIs are unchanged. Existing encoders escape canonical names.
Existing byte counters charge the longer output; no budget is enlarged or
discounted. No CLI/logging/dependency migration or upstream source edit occurs.

## Separate upstream JavaScript defect

Pinned [JS expression generation](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2980)
emits the local `name_own` tag. The minimal generated program returns
`{$: "Box", ["value"]: 7}`, but its printer descriptor names `helper.Box`.
The [show_val search](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L5995)
has no unmatched-tag termination. It cannot find `Box` in that descriptor.
This is a printer algorithm mismatch, not an import-meta/require host failure.
The six imported variants each reproduce the empty-stream timeout twice at the
recorded three-second diagnostic bound. The local and Base-sugar JS controls pass.
The actual generated failing JS is retained privately with hashes.

The port follows the pinned interpreter/C and documented printer contract; it
does not imitate the loop. No oracle source is changed, new dependency installed,
or parity exception adopted. The full printer matrix deliberately exits 1
because these required oracle observations remain unresolved.

## Independent review and validation

An independent read-only review of the exact tested source found **no blocking
defect**. It checked runtime-tag/ABI separation, type-directed dispatch, canonical
sugar, escaping, resource accounting, refusals and cancellation. It performed no
test execution. Existing cancellation boundaries remain before/after pure
normalization and before output; there is no mid-normalization interruption.

[New regression coverage](../tests/compiler_imported_printer.rs) executes eight
cases natively and as generated C/JS: local, unchanged imported reproducer,
renamed alias, dotted declaration, nested module path, two modules with the same
local tag, imported bracket-like constructors and Base sugar. Outputs and status
are exact; stderr is empty. Original/imported source hashes and generated-artifact
hashes are separate in [the ledger](cpu-imported-printer-evidence.csv).

| Check | Result on tested source |
| --- | --- |
| Standard gate | **998 passed / 0 failed / 49 ignored**, 127 result-bearing targets, 472.802 seconds |
| Focused regressions | **268 / 0 / 1**, 153.866 seconds |
| Printer matrix | 144 observations: 84 exact executions, 48 generation/build successes, 12 unresolved upstream-JS timeouts |
| Aggregate runtime matrix | 72 observations: 46 exact outputs, 24 generation/build successes, two upstream direct-Node host blocks |
| Checking audit | 1,302 unchanged statuses and streams; 439 expected negatives reject; other rows 366 accept / 497 reject |
| Additional controls | 14 positive passes; 18 observations retain three hole/TODO disagreements |

Formatting, strict Clippy, all-feature build and native Node socket-provider
checks pass. The 49 ignores are 47 GPU hardware tests and two optional profiler
tests, still unrun. Compile-fail doctest error text is expected passing evidence.
The aggregate's 23/24/27 work counts remain **634,793 / 861,655 / 1,338,629**.
Its exact original sources remain unchanged, including cancellation/writer tests.
The two direct-upstream-Node IO observations retain exit 1, empty stdout and
`TypeError: require is not a function\n`; this host limitation is distinct from
the imported-JS printer loop. No unavailable Bun lane becomes a pass.

Two newly authored control drafts initially used invalid tuple/List type syntax
and a Base-shadowing type name. Upstream rejected them during fixture preparation;
those drafts and failure records are preserved privately. Corrected controls
passed before candidate qualification. They are not port semantic discrepancies.

## Commands and storage

Actual commands after activating frozen tools:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export RUST_TEST_NOCAPTURE=1
cargo test --locked --features extended_observability --lib --test compiler_imported_printer --test compiler_constructor_continuations --test compiler_open_closures --test compiler_executable_foreign --test executable_demand --test runtime_effects --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

The debug/profile overrides affect host Rust builds, not Bend runtime optimization.
External matrices use the captured final binary and two execution repetitions:

```sh
node "$BEND_REFERENCE/bend2/main.ts" "$fixture"
teamy-bend run "$fixture"
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

O3 matches the pinned upstream CLI. Ordinary matrix children retain 30-second
bounds; the known imported-JS printer loop retains its separate three-second
bound. Checking children retain 60-second bounds and task wrappers 1,200 seconds.
Only timed-out task-owned process groups are killed and reaped.

Rust 1.96.0, nightly-2026-05-24 formatting, Node 24.19.0, Clang 19.1.7 and
PowerShell 7.5.4 remain frozen. Cargo.lock SHA-256:
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Binary SHA-256: `25c158d0e4248100ba24cd30dcb0c2e8317bde833c3de87c248592aaa8e77e66`.

The 11 GiB build preflight maintains a 1 GiB margin above the 10 GiB reserve.
Full-gate minimum free space was **10.094 GiB**. Only inactive,
verified first-party build caches were reclaimed. Source, dependencies and unique
historical failures are preserved. Successful matrix artifacts are reproducible
from retained hashes/commands; failing JS and exact binary snapshots are retained.
The first post-gate cache preflight refused just below 11 GiB before any
matrix ran. Reclaiming additional inactive first-party CLI/library/example
artifacts restored 11.057 GiB; the verified working binary/archive and all unique
evidence remain. No disk exhaustion occurred.

## Remaining scope

Tested source `936d299d3f6db320b7ae0fcc82c33bfa934bcbbf` corrects the demonstrated port
imported-constructor display gap. C and JS pure output now use canonical
qualified names while retaining local runtime/foreign tags and constructor IDs.
The independent source review found no blocking defect. Imported `Con`, `Nil`
and `Tuple` also retain their qualified spelling instead of receiving Base sugar.
[Diagnosis, commands and receipts](cpu-imported-printer.md).

The full gate passes **998 tests, zero failures, 49 ignored**; focused controls
pass **268 / 0 / 1**. Eight authored printer cases produce **84 exact execution
matches and 48 successful generation/build steps**. The **12 upstream-JS timeout
observations remain unresolved**; the complete printer matrix exits 1. This is
an upstream descriptor/tag search defect reproduced under Node, not a missing
host dependency or an approved exception. The minimal historical source and
old failed receipts are preserved. All port observations match pinned interpreter
and C O3 output; generated-JS oracle agreement is not claimed for imported cases.

The renewed aggregate matrix has **46 exact outputs, 24 successful builds/
generation steps and two unchanged direct-upstream-Node IO host blocks**. The
unchanged 27-result aggregate still costs 1,338,629 steps, below the same
2,000,000-step/128-depth guards. All 1,302 checking statuses and streams remain
unchanged; 439 expected negatives reject. Fourteen positive controls pass;
18 observations preserve three unwaived hole/TODO disagreements. No
mid-normalization cancellation or general resource-equivalence claim is added.

Coverage remains **344 originals linked / 958 without detailed receipts**:
**357 checker negatives, one checking-only module, 63 interpreter cases and 537
compiled candidates**. Authored printer reproducers do not increase original
inventory coverage. Links include historical, partial and blocked evidence.
Upstream imported-JS printing, erased-hole/TODO, resource/host/foreign gaps,
F32.read, competing failure order, downstream API caveats and the unfinished
688-vector exploration remain unwaived. CPU parity is incomplete.

Next independent batch: `flatten/simple_ctor_000`, `_001`,
`single_ctor_type_001`, `_mk` and `nested_literal`, first establishing actual
printer/type applicability, then normalized graphs/text and a bounded 33-value
application grid. These are ledger-selected interpreter cases; no new execution
credit is claimed. Dependencies, pins, toolchains, guards, budgets and security
policies remain frozen. Local checkpoints only; no publication, merge,
installation, desktop or GPU work. No new user decision was needed for this fix.

## Receipt identities

Raw evidence is private; it retains exact commands, exit/status/stream bytes,
source identities, bounds and timings. No historical receipt is relabelled.

| Receipt | SHA-256 |
| --- | --- |
| `baseline/receipt.json` | `448f289100ca30e17d2d1fdd2cdc656ef592ce8ee0d7fbdfdac51a97354af576` |
| `baseline-summary.json` | `15cd36ff53841d1e289b7e826ca8da5e45dee55124ddfc8f181528a21514bec8` |
| `source-basis.json` | `9ea4758a5b4619708efd6befa25f7615381a757560bb21fc7d5ab23f4d1bba4a` |
| `independent-review.md` | `305ea2938ce2305cdc329b251438e653459181252313a0a444dd65fa299797be` |
| `focused.json` | `54c3cf679d5426882982a08747375b81cecdee7fe4a41762090cacd981a4f906` |
| `full-gate-final.json` | `847d034e0ab6e22276f3266c0457f5b15207f388dfc61430d234846d8cf19f99` |
| `full-gate-final.log` | `934ada438fa7d7ee8b7dbe5cfb92cb1386f94e45c8d48526a9b08300d8b2835b` |
| `snapshot.json` | `b99cfae0f056761011dae1bd72979ab0ea9350e72b755ddd8b82af78e4a6dde7` |
| `matrix/receipt.json` | `5be4521c4bb844a938bba28bcf4438e45f2ed4395d46419686907eee1fe1472b` |
| `matrix-run.json` | `763aa63d8c1f6693c1d2b67b029e693af9e0a2f90f023df114c85f9dda0d131d` |
| `closed-runtime/receipt.json` | `ca997a4c0c3494b498253d0ff1142e991094b20ef25c5d82429952f2067e01aa` |
| `aggregate-run.json` | `33fa10f05160ea0fae101e82c109ff8a54fbb155817c80999a1a4b1ed31e9215` |
| `audit/receipt.json` | `2d1bccd7d03b3458b017754d86f7b16c810b9e0ff129c0e856a8b23140c032dc` |
| `audit-delta.json` | `b10f456879183b8cbdce0e94a2fc7a15ad3de04f11b1462961b9bb7a219fd6a2` |
| `controls.json` | `29c4cb3990cac5c5d9d3560e0748c1ce0ef1da8bc034a20091565c70839db1bf` |
| `coverage-summary.json` | `50e2eb69091c440e0ddcc9f764f1713e566e0e1e7ede1f6d52230985cae8fa9d` |
| `comparison-preflight.json` | `a35261d8b5563b869a7ea037b03e378fb9702263a998498bbd86e89d1d3e8e6b` |
| `captured-build-reclaimed.json` | `5ff0a607687351ae2763eca80b5d85639f2bfa269ab92f96900028c71ccb3373` |
