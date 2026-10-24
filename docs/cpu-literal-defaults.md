# Literal/default constructor qualification

Tested source: `eeb7b2814ae9fa21cf68b8c0c06cf236c419b75b`. Starting checkpoint:
`85e4b2c20fd5f724ac31d109910cb51f63e9b976`. Target:
[pinned Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
No production code, dependency, toolchain, guard or execution-policy changes.

## Applicability and bounded behavior

The five [originals](../tests/fixtures/cpu-literal-defaults) are byte-for-byte
Apache-2.0 copies of pinned `tests/flatten/literal_default_000..004.bend`.
New adapters/wrappers/tests use MPL-2.0; upstream-derived oracles retain attribution.
All original mains normalize to functions. Actual upstream C/JS and port C/JS/
native builds require Base: exit 1, empty stdout, exactly
`Error: a build needs import Base\n`, absent artifact. Separate typed Base-importing
adapters reach matching function-printer refusals. Initial applicability has
65 observations using historical binary source `0b041df`; these precede the grid.
Final applicability is independently renewed on the tested source.

| Original | Inputs | Required behavior |
| --- | ---: | --- |
| `_000` | 4 depths: 0, 1, 2, 4 | Z literal versus default |
| `_001` | 27 triples from depths 0, 1, 3 | First-column priority; retain both later columns or the whole first value |
| `_002` | 4 depths: 0, 1, 2, 4 | Zero or predecessor |
| `_003` | 6 depths: 0 through 5 | Literal ladder; reconstruct the whole default value after nested splits |
| `_004` | 4 depths: 0, 1, 2, 4 | Default binder retains the entire matched value |

[Kernel tests](../tests/kernel_literal_defaults.rs) compare all 45 direct original
applications with full constructor text and structure. Original normalization
uses complete alpha-canonical graphs, including ordered arms, fields and binding
relationships. That is not physical Rc identity; existing sharing controls remain
separate. Ten authored mains reuse the same grid: five complete imported-value
lists and five scalar projections. Their complete graphs/text are compared;
projection success cannot qualify a failed full-value printer. Native Pure,
strict generated C and Node JS execute these in [compiler tests](../tests/compiler_literal_defaults.rs).

## Demonstrated harness failure and correction

The initial clean focused run at `cf2e86a95b18abbe10ebc7591d6fe3b1bb17cbae`
exits 101. Five completed targets report 251 passes and one ignored test before
the new kernel target aborts with stack overflow. The preserved instrumented
probe reaches successful Bend evaluation, display and Shape construction for
`wrappers/literal_default_001`, then aborts in `facet_json::to_string` before
comparison. This is a recursive test-oracle serialization failure, not a
Bend evaluation mismatch. The initial native/C/JS and aggregate tests had passed.

The test-local replacement writes complete constructor JSON using an explicit
stack, delegates string escaping to Facet, preserves every name and ordered
child, and rejects all non-constructor nodes (including nested ones). The full
Shape visitor and its 128-level guard still run first. Original open graphs and
45 typed direct-application comparisons remain unchanged. Controls cover
shallow Facet equivalence, escaped names, child order, 96-deep output and
unsupported-node rejection. No generic JSON rewrite, dependency change, larger
host stack, evaluator switch or relaxed Bend budget was introduced.

Independent read-only review of `eeb7b2814ae9fa21cf68b8c0c06cf236c419b75b` found no blocking concern or reduced
structural coverage. The reviewer ran no tests. The dirty four-test candidate
probe and original failed run remain separate from clean-source validation.

## Current results

| Check | Result |
| --- | --- |
| Full standard gate | **1,009 / 0 / 49**, 488.518 seconds |
| Clean focused gate | **24 / 0 / 0**, 16.973 seconds |
| Original/applicability collection | 150 observations, including 100 exact refusals with no artifact |
| Graph collection | 60 exact structural observations, two implementations twice |
| Runtime collection | 112 exact executions, 60 successful builds/generation, eight unresolved upstream-JS timeouts |
| Checking audit | 1,302 unchanged statuses/streams; 439 expected negatives reject |
| Additional controls | 14 positive passes; 18 observations retain three hole/TODO disagreements |

All scalar projections and the full `_000` value agree across six available lanes.
The latter uses only nullary result constructors, whose generated JS tags are
qualified correctly. The four remaining full-value cases reach local field-bearing
tags absent from their qualified printer descriptors. Full constructor values
agree in native Rust, port C/JS, pinned interpretation and upstream C O3.
The four field-bearing upstream-JS full-value mains time out twice at the
separate three-second diagnostic bound, with empty streams; only those owned
processes are killed and reaped. This repeats the [qualified-name versus local-tag
printer defect](cpu-imported-printer.md). Generated failing programs and hashes
are retained. The generic progress label for `_000` inaccurately says timeout;
the structured rows record two exact successes and are used in these counts.
A timeout is not a language exit status. The overall runtime runner
exits **1**, and the full matrix remains incomplete; no oracle exception is added.

Full-gate aggregate sharing/work/native/C/JS/cancellation regressions pass:
23/24/27-result costs remain 634,793 / 861,655 / 1,338,629 steps. The existing
2,000,000-step and 128-depth guards remain. Cancellation remains at its existing
before/after-normalization/output boundaries; no mid-normalization interruption
or general resource equivalence is claimed. The complete external 72-observation
aggregate receipt remains historical at `936d299`, not renewed here.

Formatting, strict Clippy, all-feature build and Node socket-provider checks pass.
Unrun/ignored: 47 GPU hardware tests and two optional profilers. Expected
compile-fail doctest diagnostics are passing rejection evidence.

## Reproduction and storage

After activating the existing tools:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_literal_defaults --test compiler_literal_defaults --test compiler_imported_printer --test executable_demand --test runtime_effects --quiet
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug overrides do not change Bend runtime optimization. External commands
and all exact argument arrays/status/streams are saved in private receipts:

```sh
node "$BEND_REFERENCE/bend2/main.ts" "$fixture"
teamy-bend run "$fixture"
teamy-bend --output-format json eval "$fixture"
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

Two execution repetitions; ordinary matrix bound 30 seconds, known JS printer
probe three seconds, checking children 60 seconds, outer task bound 1,200 seconds.
Tools remain Rust 1.96.0, nightly-2026-05-24 fmt, Node 24.19.0, Clang 19.1.7 and
PowerShell 7.5.4. Cargo.lock SHA-256:
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured binary SHA-256: `fc0beea948255bfaa51e422bd143eeac83fd713ddd4e93da15f8125343d3c905`.

Build preflight requires 11 GiB total free, 1 GiB above the 10 GiB reserve.
Full-gate minimum: **10.020 GiB**. A preflight after the
failed focused run refused to start below that margin. Completed first-party
CLI/test caches were reclaimed after inactivity checks; exact failed/current
CLI bytes are retained in verified lossless archives. No source, dependency
cache or unique historical failure was deleted. Duplicate binary archives are
reused only after byte verification. Graph receipts are compact JSON retaining
all observations and nodes. The matching comparison library was reclaimed only
after all 60 graph observations completed; the probe and its source/hash remain.
Preflight refusal is not a test execution failure.

## Coverage and next work

Tested source `eeb7b2814ae9fa21cf68b8c0c06cf236c419b75b` adds five unchanged `flatten/literal_default_000..004`
originals and a bounded **45-input** grid. Original function graphs/text, direct
applications, full imported constructor values and scalar projections remain
separate. No production divergence was found. [Scope and receipts](cpu-literal-defaults.md).

Full gate: **1,009 passed, zero failures, 49 ignored**. Clean focused controls:
**24 / 0 / 0**. Sixty structural and 150 original/applicability observations
complete. Runtime observations comprise **112 exact executions, 60 successful
build/generation steps and eight unresolved upstream-JS printer timeouts**.
The complete runtime runner exits 1; full values match native Rust, port C/JS,
pinned interpretation and upstream C O3, without claiming upstream-JS parity.

A new test initially overflowed during recursive graph JSON encoding, after
Bend evaluation and Shape construction succeeded. A reviewed test-only explicit
stack now preserves every constructor name/ordered child and rejects other
nodes. Failed receipts remain tied to `cf2e86a`; no evaluator, stack-size,
guard, budget or cancellation change occurred. Current aggregate controls pass;
no mid-normalization cancellation is claimed. All 1,302 checking statuses and
streams remain unchanged, with 439 expected negatives rejecting. Fourteen
positive controls pass; 18 observations retain three hole/TODO disagreements.

Detailed receipts link **354 originals**, leaving **948 unlinked**: **357 checker
negatives, one checking-only module, 53 interpreter cases and 537 compiled
candidates**. Links include historical, partial and blocked observations, not
completed parity. Imported-JS printing, erased-hole/TODO, resource/host/foreign
gaps, F32.read, competing failure order, downstream API caveats and the unfinished
688-vector exploration remain unwaived.

Next proposed batch: `flatten/dead_residue_elision_000`, `_001`, `_002` and `_004`.
Establish original function/concrete-value applicability first, then **29**
applications of `main` or `f` as appropriate. No next-batch execution credit.
Dependencies, toolchains, upstream references, budgets, guards and cancellation
remain frozen; no installs, pushes, merges, desktop or GPU work. No new decision
is required by this checkpoint.

## Receipt identities

The [public execution ledger](cpu-literal-defaults-evidence.csv) records results
without machine paths. Private receipts retain commands, exact streams, hashes,
source identities, timeouts and timings.

| Receipt | SHA-256 |
| --- | --- |
| `preflight.json` | `7670dba082357a7410f00da21e4c2bf1300a83aa661f5ca17a980bb9a897ead8` |
| `applicability/receipt.json` | `385fd1f1c00147c091175aa4bdc6fa0e82ca4c53c0ff5c99477b157f19cb56ec` |
| `application-preflight.json` | `0ffa8c3395d788ba846aaaea958a9f9ac27c4ec164d6d71bd3a687b59d369aa9` |
| `focused-final.json` | `ad1c5c98988cc9839a8901aa1fc90c4fc69884b095720733ae5a1139a147e90a` |
| `focused-final.log` | `7ee0ad089075193b2d75e2924d73e86aaa2969d1e5b96be78c4ea350639bb771` |
| `stack-probe.json` | `523e6882953ff4c298a4ed8511875b0096c112600796580eb6b3bdbeec0aa798` |
| `stack-probe.log` | `e7d99c34aecf1bb05c75d0c49e84a766ad0815c0c4466a742cd9c71cf6ed6c02` |
| `graph-encoding-probe.json` | `9f5fd541899459b8c87b8b741edd39a949fb175fd6b28d75859e063beedf9f12` |
| `encoding-review.md` | `827442de38818b75de0bc46a82f897d56949b20d06394ee2bef0503f35cb5bbb` |
| `focused-renewed.json` | `996362266372ca9a239c9aa608a3651dc4c3dff9999cea6075735c72dec51448` |
| `full-gate-final.json` | `b4617924a68d73ae9925f0d71bc3da8d6ce0602f7b377eb089e7f0447960c796` |
| `full-gate-final.log` | `bcab1018e71a34f6b70e7f8772de9ebe6e811adfdab947ddec6a317a37be42eb` |
| `snapshot.json` | `9ead66540119b1b80fa903b42fd5c540ac2163563eb145e3de3708d4ea322358` |
| `open-matrix/receipt.json` | `1c3b6309104e3fdab76fcf6da7bb3acd3236d7c6cfe2e6e232881fad12e65593` |
| `graphs-matching/receipt.json` | `e52c5c70e869ec2885bb38835665a6d0f458cb02195a072bd8c24835e01aa9e8` |
| `matrix/receipt.json` | `37455e12d7ebd82a58794515035b43c8cbe2209488efc5b90c63f711313a9558` |
| `matrix-run.json` | `0a638f273c393de8070e5c69a8993ff6a7a5bd20d736bd14b60855a94121b030` |
| `audit/receipt.json` | `dc3eb62c23d07d7781cb822c41cf2e87a1ba8f47b2b40a39a1fa2288e25fe5c3` |
| `audit-delta.json` | `47a0ce0af68fb1b7945ff4aa210228e67f8461d79949877629bcbea120e11df4` |
| `controls.json` | `22eafaef5708fda196a4f40998a01fd43e0b0910b5140b3ed8016f1dd3c26308` |
| `coverage-summary.json` | `286a4f5e4d9023917a37c5aa095df4e1e9127e2343b482e2816d034376ec9698` |
| `tool-versions.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `comparison-preflight.json` | `8be6159eb3bb88dfd488a8dbb89bcfd8d61039afde4864dc6ad376b11f51cc9d` |
| `upstream-js-evidence.json` | `4dfe071257712a131c60217878e60ed1ae850e3c093b7e79e18ff9c013ef61d3` |
| `comparison-library-reclaimed.json` | `c27e70a261f05c03ade57b74bf72f7bc69f22628ed8404335a81099e0b111d1d` |
| `preflight-note.json` | `9e569157f9e0f30796e1883e48b1939687f72d56f21f0243ec22e4078217e5d8` |
