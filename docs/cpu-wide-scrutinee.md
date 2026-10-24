# Wide scrutinee columns: bounded CPU checkpoint

Tested source: `93d10bf440096736fd60a977032c3869d8ebc9c6`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The five [originals](../tests/fixtures/cpu-wide-scrutinee/README.md) retain exact
Apache-2.0 upstream bytes. New tests and observation wrappers use MPL-2.0.
No production defect was demonstrated or implementation change needed. CPU
parity remains incomplete.

## Applicability and concrete behavior

Four originals have type `@a:Nat -> @b:Nat -> @c:Nat -> @d:Nat -> R`;
`_002` adds `@e:Nat` before `R`. Pinned type introspection and Rust structural
assertions preserve every `Lone` binder, domain and result. These originals
return functions. Emission first fails because Base is absent; separate
Base-importing function aliases pass source checking but still cannot be printed.
Upstream C/JS and port C/JS/native emitters match complete stderr, exit 1, empty
stdout and artifact absence at both boundaries. The pinned
[printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
explains these outcomes. They are not executable passes for the original mains.

All input columns vary independently over constructor depths **0 and 2**, with
the rightmost column varying fastest. The four-argument grids contain 16 cases
each; `_002` has 32. This bounded sample is not arbitrary-depth or exhaustive
semantic coverage. It checks all three result constructors in each original:

| Original suffix | Applications | A / B / C counts | Behavior exercised |
| --- | ---: | --- | --- |
| `_000` | 16 | 4 / 8 / 4 | First/fourth discrimination; middle columns ignored; predecessor and full last field |
| `_001` | 16 | 4 / 8 / 4 | Second/fourth discrimination; first field forwarded; third ignored |
| `_002` | 32 | 4 / 8 / 20 | First/third/fifth discrimination; second/fourth ignored; reconstructed default |
| `_003` | 16 | 4 / 4 / 8 | Different ordered payloads in all three branches; first/last predecessors |
| `_004` | 16 | 1 / 8 / 7 | Four-column zero row; complete four-field default; first-field predecessor |

For example, `_002` at depths `(2,0,2,0,0)` returns `C` with depths `(2,2,0)`;
the failed last-column `S` test must reconstruct the complete first argument.
At `(2,0,2,0,2)` it returns `B` with depths `(1,2,1)`. `_003` at `(0,2,0,2)`
returns `B` with depths `(1,2,0)`, retaining field order. These observations derive
from the unchanged [five source fixtures](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten)
and pinned interpreter, not inferred branch labels alone.

The [96 exact constructor results](../tests/fixtures/cpu-wide-scrutinee/applications.tsv)
are checked twice against the pinned interpreter, yielding 192 oracle values.
The Rust checked evaluator compares complete values and structural shapes.
Original function normal forms match golden text and actual checked structure.
Pure CLI `run` text and JSON `eval` values are checked separately; no separate
observation of the CLI writer's internal core is claimed.

Typed pure wrappers return `List<List<U32>>`. Each observation retains the result
tag (A=0, B=1, C=2) followed by every field's constructor depth, in order. IO
adapters print those same rows. Tests assert `ExecutableEntry::Io` before invoking
the native executable driver; pure `run_main` uses normalization. Native IO and
upstream/port generated C and JS agree on exact stdout, empty stderr and exit 0.
These projections do not qualify the separately open imported-constructor spelling.

Direct upstream IO under installed Node returns exit 1, empty stdout and
`TypeError: require is not a function\n`, because `import.meta.require` is absent.
Ten calls remain host-blocked. Five strict IO proof checks remain proof-mode
exclusions; executable acceptance is checked separately. No host was installed
or substituted, and neither category is a runtime pass.

## Validation and provenance

| Check at tested source | Result |
| --- | --- |
| Full standard gate | 951 passed, 0 failed, 49 ignored; 112 result-bearing targets |
| Format / strict Clippy / all-feature build / TCP-UDP provider | Passed |
| New tests within the full gate | 3 passed: type/core/applications, pure C/JS, native IO C/JS |
| Original normalization / type introspection | 30 / 5 observations |
| Direct oracle applications | 10 invocations, 192 values, 96 distinct fixture/input pairs |
| Original missing-Base / function-printer refusals | 50 / 50 exact observations; no artifacts |
| Accepted-source phase probes | 20, distinct from emission and execution |
| Pure output / strict eval | 60 / 10 exact observations |
| Native IO and generated IO outputs | 50 exact observations |
| Direct IO oracle / strict IO proof | 10 host blocks / 5 exclusions |
| Generation / C build | 60 successes |
| Final matrix | 360 process observations; result lanes repeated twice, no timeout or nondeterminism |
| Checking-status audit | 1,302 unchanged: 439 expected negatives rejected; others 366 accepted / 497 rejected |
| Positive / hole controls | 14 pass; 18 observations preserve three unresolved cases |

The [kernel](../tests/kernel_wide_scrutinee.rs) and
[compiler](../tests/compiler_wide_scrutinee.rs) regressions ran alongside existing
proof, guard, cancellation and CLI boundary controls. The gate completed in
313.28 seconds from clean source. Its 49 ignores remain unrun:
47 GPU hardware tests and two optional profilers. The captured binary uses
`default,extended_observability`, verified against the actual Cargo artifact hash.

Initial applicability and value probes retain the earlier binary identity
`e46434c5417252287478158e45260b3cbe6603eb`; they are not relabeled as current-source
evidence. The final matrix repeats these contracts with the current captured binary.

## Reproduction and storage

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

Generated JS runs under Node; port emission uses `--target javascript`. The
matrix uses the pinned Linux O3 profile for C. The separate Rust test compiler
uses strict flags without an explicit optimization switch. Host Rust debug
overrides do not change Bend runtime optimization or budgets. Exact argv,
streams, statuses, hashes and durations remain in private receipts; the
[public CSV](cpu-wide-scrutinee-evidence.csv) records compact contract identities.
Bounds remain 30 seconds per matrix child, 60 per audit child, 20 per compiled
regression child and 1,200 for the gate.

Build preflight remains 11 GiB, one GiB above the 10 GiB reserve. The gate started
with 11.071 GiB free and reached 10.377 GiB minimum. After binary capture,
108 inactive rebuildable integration-test executables were retired
before external C builds. Source and unique historical evidence, dependencies
and toolchains were preserved. Current free space is 11.049 GiB; recheck
before another build. Emergency stop floors remain 2 GiB workspace / 1 GiB temp.

Detailed receipts link **318 originals**; **984** lack detailed receipts:
**360 checker negatives, one checking-only module, 73 interpreter cases and 550
compiled candidates**. Five interpretation rows are new; prior 130 interpretation,
93 runtime and 90 checking rows retain their historical identities. Linked
receipts include partial/blocked cells and do not establish CPU parity.
Imported-constructor printing/upstream JS timeouts, erased-hole/TODO,
host/resource/foreign/F32.read/runtime-error-order/API requirements and the
unfinished 688-vector exploration remain unwaived. Dependencies, references,
toolchains, guards, budgets, cancellation and approved demand policy remain
frozen. No installations, pushes, desktop or GPU work.

Next independent batch: five `list_ctor_000..004` originals, with a proposed
78-vector sample. The first four use all lists of length 0..2 over Bit F/T or
Nat depths 0/2; `_004` uses both Nat depths and five selected list shapes in each
list argument (50 combinations). This would exercise parameterized constructors,
nested head/tail defaults and complete ordered list payloads. Actual printability
and type/kind applicability must be established first. Only source/inventory
review is complete; no execution credit is assigned to this next batch.

The imported-printer [discrepancy and JS timeout](cpu-ragged-rows.md#open-imported-constructor-printer-discrepancy)
remain tied to earlier source `623c6bacfc82993e731c4c124306924c1d68d8eb`, not rerun
or relabeled here. The [acceptance criterion](cpu-parity-acceptance.md) still
requires every applicable lane and API/failure/resource contract.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `09f9cf7cac539af06dcec783173313225e6463b26ca9dd205eeb0430d3c97d6d` |
| snapshot.json | `6a15865e70126dd2491f12d7fc8d03d5f715e0da12138d9f572d4e2af472f3da` |
| snapshot/teamy-bend.gz | `d13bb21aa97c6d38e605f4189fe9061a1635d1c9709b48e1128458457079aeff` |
| full-gate.json | `b48eaac0ff91a67ee5a9f4bb907cfdbcd7ec463909567020a03360459366dc1f` |
| full-gate.log | `1c3c4cccf7d4a092f0260fa3bedb25eeb4e38a6b9edd4dd70da7135d7b2dd622` |
| qualified/receipt.json | `77774972d9529e31a956dc13da741ce3500f6ec15814d7d779bfa863d1134571` |
| preflight-applicability/receipt.json | `0d5e6cb68284f81f8e26a0b1d020d4e06ae437b468dbe7a55180a1ff7150af77` |
| preflight-values.json | `5227163ebbc0500afa328d5f35417d6973276682ab1dfda8da4d8eac8583de20` |
| audit/receipt.json | `4f575a085ab49bb725f84e72e3cb21c478dbc6dea570b07c691b04e9cc77de8f` |
| controls.json | `7eef27895104e5b0130113499837e57ccf2628d5af11322c33819b36cc4e7073` |
| coverage-summary.json | `4c686842a9fe8d42e88b03988df69ff489150eac4fc8bcf4f1cacd4a9a2be15b` |
| inactive-test-cache-reclaimed.json | `85bc1731041bcc31de0e67a2e0f1473a195dc41f1bf50f7ab51eaadb88b12ffd` |
| qualify.py | `9eaf7703e6ca390447b06dc12f440be5fdb64e8c4634e089066cc4657fa7ea4f` |
| types.mjs | `d2319d1a519234fbc7525a97f07d8ce456c3e3ef6aa14399e071265c54679709` |
