# Nested computed matching qualification

Tested source: `cc3818eaa394ab3be74d9edd14b00e7763c11da1`. Starting checkpoint:
`036c5f1b3216bb480d9e969cccad678a3d584657`. Target:
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Only tests, fixtures and evidence documentation change. The final evidence commit
is documentation-only after this tested source; its identity is in the handoff.

## Applicability and bounded results

The three [originals](../tests/fixtures/cpu-nested-computed) are byte-identical
Apache-2.0 copies of pinned `tests/flatten/nested_computed_match_001`, `_dec`
and `_zro`. Derived text/graph/diagnostic oracles retain attribution; new tests,
adapters and wrappers use MPL-2.0. Every original main is a function. The original
standalone C/JS/native builds return status 1, empty stdout and exactly
`Error: a build needs import Base\n`. Separate Base-importing typed adapters
reach the matching function-printer refusal, without an output artifact.
Neither refusal becomes a successful executable lane.

The 39-observation initial applicability receipt uses the historical `dcf26ff`
binary and is identified accordingly. The final 90-observation collection renews
applicability on the tested source, separately recording pure CLI text, strict
eval report text, upstream type/checking phase and concrete applications.

| Original | Inputs | Exact direct results |
| --- | --- | --- |
| `_001` | Nt depths 0, 1, 2, 3, 5 | K5, K6, K7, K7, K7 |
| `_001_dec` | T, F | K6, K7 |
| `_001_zro` | Nt depths 0, 1, 2, 3, 5 | T, F, F, F, F |

Names in this table denote complete nullary constructor values. The original
`_001` normal form retains the named nested call `dec(zro(p))` below the successor
arm; it is compared structurally and textually, separately from these closed
applications. [Kernel tests](../tests/kernel_nested_computed.rs) preserve every
graph node, ordered field/arm and binding relationship, with readback controls.
Alpha-canonical shape is not physical Rc identity; sharing controls remain separate.

Three full imported-value lists and three scalar projection mains reuse all
12 inputs. Complete graph equality is checked for both representations, retaining
the existing explicit-stack constructor serializer and its escaping/order/deep
output/rejection controls. No graph fields are dropped and no guard is relaxed.
[Compiler tests](../tests/compiler_nested_computed.rs) execute all six mains in
native Rust Pure, strict generated C and Node JS. Independent read-only review of
the tested source found no blocking concern or weakened comparisons.

## Validation and limits

| Check | Result |
| --- | --- |
| Full standard gate | **1,024 / 0 / 49**, 498.470 seconds |
| Focused gate | **33 / 0 / 0**, 18.518 seconds |
| Complete normalized graphs | 36 matching observations, both implementations twice |
| Normalization/applicability | 90 observations, including 60 exact refusals with no artifacts |
| Six executable mains | 72 exact executions and 36 successful generation/build checks |
| Checking audit | 1,302 unchanged statuses/streams; 439 expected negatives reject |
| Additional controls | 14 positive passes; 18 observations preserve three hole/TODO disagreements |

The executable matrix repeats upstream interpretation, port pure CLI, upstream
C O3, strict port C O3, upstream JS and port JS. Every required observation in
this bounded matrix completes with matching stdout/stderr/status; runner exit 0.
Full values use nullary result constructors with correctly qualified upstream-JS
tags. This does **not** close the known field-bearing imported-printer failures
from [earlier batches](cpu-imported-printer.md). Their failed programs and timeout
receipts remain unchanged and unwaived. No projection substitutes for full output.

Formatting, strict Clippy, all-feature build and Node socket-provider checks pass.
The full gate renews named-call reconstruction, aggregate sharing/work/native/C/JS,
demand/error order, guard and cancellation regressions. Aggregate 23/24/27-result
costs remain **634,793 / 861,655 / 1,338,629**, under the same 2,000,000-step and
128-depth guards. Existing before/after-normalization/output cancellation stays
unchanged; no mid-normalization interruption or general resource parity is claimed.
The separate external 72-observation aggregate receipt remains historical at
`936d299`. Forty-seven GPU hardware tests and two optional profilers remain ignored.

The [IO preflight correction](cpu-dead-residue.md) is unchanged. Executable emitter
APIs can now return Err where no-IO inputs previously produced source. Repository
callers were audited at `dcf26ff`; external consumers were not audited. This batch
renews its regression tests, not an external compatibility audit. Upstream C's
minimal-IO scaffold compiler failure and the foreign-source checker diagnostic
disagreement retain their prior evidence identities and remain unqualified.

## Commands, frozen inputs and storage

After activating the existing tools, from the repository root:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --test kernel_nested_computed --test compiler_nested_computed --test kernel_dead_residue --test compiler_dead_residue --test compiler_imported_printer --test executable_demand --test runtime_effects --quiet
RUST_TEST_NOCAPTURE=1 pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug settings do not alter Bend runtime optimization. External comparisons:

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

Original and adapter refusal probes additionally use `--target native`, which is
C packaging, not the native Rust driver. All exact commands, raw status/streams,
source/artifact hashes, deadlines and timings are retained in private receipts.
Two execution repetitions; ordinary children 30 seconds, imported-JS full-value
printer probes three seconds, checking children 60 seconds, outer bound 1,200
seconds. No timeout occurred in this matrix.

Repository tool versions remain Rust/Cargo 1.96.0, nightly-2026-05-24 fmt,
Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4. A probe outside the repository
observed its pre-existing global Rust 1.99.1; all build/check processes use the
repository override. Repeating version probes from that context exactly matched
the frozen versions. No installation or toolchain change occurred. Docs reference
remains `50e89d6897ce2a65167ef65b17c5fa83fe849158`; both reference checkouts are clean.
Cargo.lock SHA-256: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.
Captured binary SHA-256: `1d93a1ac927d2aa60ac77cee3156a6c7decfedc805cd8229e274853c0a8fb307`.

Build preflight requires 11 GiB free, one GiB above the 10 GiB stop floor.
The gate started with 11.054 GiB and reached a minimum
of **10.057 GiB**. Completed inactive first-party build caches
are reclaimed after binary capture; the comparison library is reclaimed only
after its graph checks complete. Existing lossless binary archives are reused
only after byte comparison, avoiding redundant retained captures. Source,
dependency caches and unique historical evidence remain intact. The earlier
array-read archive and restore receipt remain at their original evidence identity.
Compact graphs retain all nodes. Final comparison headroom is
11.094 GiB.

## Coverage and next batch

Tested source `cc3818eaa394ab3be74d9edd14b00e7763c11da1` adds three unchanged
`flatten/nested_computed_match_001`, `_dec` and `_zro` originals and **12 bounded
applications**. Original function graphs/text preserve `dec(zro(p))`; closed
constructor results, scalar projections and execution are distinct comparisons.
No production defect or code change was found. [Scope and receipts](cpu-nested-computed.md).

Full gate: **1,024 passed, zero failures, 49 ignored**. Focused controls:
**33 / 0 / 0**. All **36 complete graph** and **90 normalization/applicability**
observations pass, including 60 exact standalone-build refusals without artifacts.
Six authored executable mains produce **72 exact repeated executions and 36
successful generation/build checks**, with zero matrix timeouts. Their result
constructors are nullary; this success does not resolve the historical
field-bearing imported-constructor upstream-JS printer failures.

Aggregate sharing/work/native/C/JS/demand/cancellation regressions pass in the
full gate; no mid-normalization cancellation is claimed. All 1,302 checking
statuses and streams remain unchanged, including 439 rejecting negatives.
Fourteen positive controls pass; 18 observations retain three hole/TODO
disagreements. The IO emitter correction's API caveat remains: no-IO executable
compilation may now return Err for external consumers, which have not been
audited. Minimal-IO upstream C compiler failures and foreign checking diagnostic
differences remain separately unqualified; prior receipts are not renewed by
matching cases in this batch.

Detailed evidence links **361 originals**; **941 remain unlinked**: **357 checker
negatives, one checking-only module, 46 interpreter cases and 537 compiled
candidates**. Links include partial, blocked and historical evidence, not
completed parity. F32.read, competing failure order, resource/host/foreign gaps,
erased-hole/TODO, downstream API caveats and the incomplete 688-vector exploration
remain unwaived. Missing API/integration requirements still require review.

Next proposed coherent family: seven `flatten/annotated_computed_scrutinee_000`
originals/companions, actual applicability first, then **77** bounded applications
covering list equality/length, annotation, selection and Boolean helpers.
This is source/ledger selection only; no next-batch execution credit.
Dependencies, pins, toolchains, guards, budgets and cancellation stay frozen.
Local checkpoints only; no installs, publication, merge, desktop or GPU work.

## Receipt identities

The [public ledger](cpu-nested-computed-evidence.csv) records 234 observations
without machine paths. Three interpretation links are appended; old rows and
historical acceptance receipts are preserved.

| Receipt | SHA-256 |
| --- | --- |
| `preflight.json` | `6760e5ada27b0fa108cb22087429f66d81b7009f18c1b2623063906e5ddc980f` |
| `applicability/receipt.json` | `2d0925d34aa2f96472d1a800262a1e333703f1d8ac94e0cb6891fcfbcdd50429` |
| `application-preflight.json` | `538f6ff377f502d6edd84c913e9c3837afe8f2d75ecabefa5b56b2c5d7a86a55` |
| `independent-review.md` | `9cb44e5b8a90d8895ac8114c6c6d993ca517d864252ec23ac4cceaab9d9582d0` |
| `focused-final.json` | `756585ac7b820da05918a05ab157377560e84a7d35cc690c9237e41a7b627ad3` |
| `focused-final.log` | `f485faccae70daae376f0d3d4eb68dbe5862f83dd0e531d3f61d2c08ac56a067` |
| `full-gate-final.json` | `53c1bb8c97e6fba0c0af516392a89c81a22da3fbcfe5660f6b65d9df6f0b962d` |
| `full-gate-final.log` | `2ef2d950b8514ee7e6d264947f5ff66b2b7a069c4844dda7808dc1d5e6c44b34` |
| `snapshot.json` | `bdba0378d133436ba906f950f3667f099edc41f829468793b1c05a37058d7ff3` |
| `open-matrix/receipt.json` | `0c19e15e67d58b93ce15bca8a6cc6e14f730b3fa35c6177041b4655bdd61ba41` |
| `graphs-matching/receipt.json` | `74fa33335d3e38f2bec8ed5ea5eed3a0d818a3067dbae6f2a25c2fc22cfe388a` |
| `matrix/receipt.json` | `6b195c0ef2ac7baf3907d07609188bbe0498208d69c9f6866be9ad314999d6cc` |
| `matrix-run.json` | `3555f9b8d8c60935297c4f23f882f2da96aabc58f60f82efff403f2d4746a7b0` |
| `audit/receipt.json` | `4912b85742dce1862b1623df58573b64f0828b48fd05c31ac5aa0548e2a27a90` |
| `audit-delta.json` | `7a7a11eb1a2092d71a2edb606cec9e51e90eab41d4991f1221ad5fb869a703e8` |
| `controls.json` | `e578107c36020c7c9d1649fe6370d55121a010cfceaa742cf7f24ac356ae3589` |
| `coverage-summary.json` | `406d125d923c35a1518fa4c636d76c37ecef85c9b920137f08af1b3ccb95a5a8` |
| `tool-versions-repository.json` | `4eb2afbecdd21c2905f4a66d972b268edbc13c180d3633e99799e641fd2aa46e` |
| `tool-version-context.json` | `6f2bfe2a0b344314463209c1bd7366de54e7013c029e924305d1af4b6b225f94` |
| `comparison-preflight.json` | `987c00267328138bf91108a7902fe7bd084e876b0b0e4824f1f258e05c3dbf03` |
| `comparison-library-reclaimed.json` | `cf3bb05c74496e5ae5ebc0b6cfb43f7fa16de14cc9d98eb76ab88368efc02b34` |
| `next-gap.json` | `731a7fe607865d16a3dce142f894abf87059564f853a83e1395a866315a5a297` |
| `upstream-js-evidence.json` | `7be9674e1dc4203782d74df9488595b7f3d08fd820b06da031215757fce6cc9f` |
