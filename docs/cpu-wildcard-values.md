# Wildcard functions and empty-match values

Tested source: `49aeac60e8794b0f7f0644cbcc0450a5a355054a`. Target remains
[pinned Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The [three originals](../tests/fixtures/cpu-wildcard-values/README.md) retain
upstream bytes and Apache-2.0 attribution. New tests and wrappers are MPL-2.0.
Only tests, fixtures and evidence change; no production implementation, public
API, dependency, toolchain or safety-policy change is included.

## Applicability and exact normalization

| Original | Exact pinned and port text | Actual executable printer result |
| --- | --- | --- |
| `underscore_print` | `_ => 0n` | Refuses `@x:U32 -> Nat` |
| `wildcard_chain_print` | `_ => _ => 5n` | Refuses `@x:U32 -> @y:U32 -> Nat` |
| `efq_arm_value` | `(\{}, e => Empty.absurd(Bool, e))` | Refuses `Pair(@e:Empty -> Bool, @e:Empty -> Bool)` |

All originals pass strict checking and pure normalization. Actual upstream C/JS
and port C/JS/native emitters refuse all three: exit 1, empty stdout, exact
matching stderr, no output artifact. These are function-containing values,
not original runtime successes. The inventory's compiled-candidate hints do not
override measured printer applicability.

The pinned [binder allocator](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1612)
allocates a fresh identity for every binder and omits `_` from name lookup.
The persistent [kernel tests](../tests/kernel_wildcard_values.rs) check two
distinct IDs even though both lambdas display as `_`; the bodies remain constant
and do not capture the supplied U32 inputs. Exact printed text, Facet-decoded
graphs, and parsed-text structural round trips are separate assertions.
Graph comparison retains constructor names, application edges, references and
bound-variable distances; it does no eta conversion or evaluation repair.

For `efq_arm_value`, the first tuple field is the `Efq` value itself. The second
is a lambda containing `Empty.absurd(Bool, e)`, whose argument refers to that
lambda's ID. They must not collapse into the same representation. The pinned
[weak normalizer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2891)
refolds an empty match only when a pending named-call argument remains; the
[constructor-match path](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2980)
distinguishes a selected arm from exhausted matching. The
[checker rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3510)
requires an empty remaining constructor domain or an already unreachable context.
No source-hole or structural refusal is relaxed to obtain these results.

## Bounded applications and runtime evidence

The two separate [wrappers](../tests/fixtures/cpu-wildcard-values/wrappers) apply
U32 inputs 0, 7 and 4294967295. Three unary results are `0n`; all nine ordered
binary pairs return `5n`. The three first-argument applications of the chained
function each retain `_ => 5n`, without capturing the discarded outer argument.

Both `pick(True{})` and `pick(False{})` return `\{}` as an unapplied function.
These are two valid Boolean-to-function observations, not calls to a function
on `Empty`. No `Empty` value is constructed, no unreachable code is forced and
no runtime lane is claimed for that original.

The 17 applications therefore comprise **12 complete values, three partial
function values and two Boolean-to-function selections**. Pinned application
probes and persistent port tests agree exactly; complete results additionally
have graph assertions. These finite samples do not prove arbitrary-program parity.

The wildcard wrappers print `[0, 0, 0]` and `[5, 5, 5, 5, 5, 5, 5, 5, 5]`, each
with one newline, empty stderr and exit 0. They agree under upstream interpretation,
port pure CLI normalization and upstream/port generated C and JS, twice per
result lane. The [persistent compiler test](../tests/compiler_wildcard_values.rs)
separately executes the native driver and generated C/JS. That native observation
is not inferred from the CLI's pure normalization route. Partial and empty-domain
functions are not relabeled as executable closed results.

## Current-source validation

- Full standard gate: **977 passed, zero failed, 49 ignored** across
  **122 result-bearing targets**, in **325.14 seconds**.
  Format, strict Clippy, all-feature build and real socket-provider checks pass.
- Focused library, new tests, constructor-binder, normalization, demand and effect
  controls: **289 passed, zero failed, one ignored**. New tests pass on first run.
- Open matrix: **62 observations**: 18 original text results, 30 exact printer
  refusals, eight application/partial-application probe invocations and six
  type/phase observations. Result lanes repeat twice; type/phase probes once.
- Graph matrix: **16 observations**, four sources, two implementations, two
  repeats. All exact, including `partial_app_snf` named-call reconstruction.
- Closed matrix: **36 observations** for the two wrappers: 24 successful exact
  outputs and 12 generation/build successes. No blocked, timed-out or divergent
  result is counted as passing. No observed nondeterminism in final result lanes.
- All **1,302 checking statuses and streams** match the preceding port receipt:
  439 expected negatives reject; remaining cases are 366 accepted / 497 rejected.
  This audit is regression evidence, not universal upstream diagnostic parity.
- Fourteen positive controls pass; 18 observations retain three unresolved
  hole/TODO differences. Native wildcard/private-request refusal and the approved
  upstream C wildcard exception remain distinct. No new exception is introduced.

The full and focused gates retain sharing, named-call reconstruction, false-proof,
source-hole, first-error, work/depth/structural and cancellation regressions.
The 49 ignored cases remain unrun: 47 GPU hardware and two optional profilers.
The new matrices contain no IO or shift programs and use no GPU hardware.
Previously observed Node IO host limitations and external-consumer API caveats
remain open.

## Reproduction and storage

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test kernel_wildcard_values --test compiler_wildcard_values --test kernel_constructor_binders --test kernel_normalization --test executable_demand --test runtime_effects --quiet
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

Rust 1.96.0, formatter nightly 2026-05-24, Node 24.19, Clang 19.1.7 and PowerShell
7.5.4 are unchanged. Rust debug/incremental overrides control host build storage;
they do not alter Bend evaluation budgets or C optimization. External C uses the
pinned O3 profile; persistent C regressions retain strict warnings without an
explicit optimization flag. No dependency installation or network-policy change.

Timeouts remain 30 seconds per external comparison, 60 per audit child/private
probe build, 20 per persistent generated-child execution and 1,200 per gate.
Receipts retain exact commands, fixture/binary/artifact hashes, stdout/stderr,
statuses and deadlines. The [compact ledger](cpu-wildcard-values-evidence.csv)
contains no private paths or raw diagnostics.

Build preflight requires 11 GiB free above the unchanged 10 GiB reserve.
Completed focused test executables were reclaimed before the gate; no gate launch
was refused. Minimum gate headroom was **10.193 GiB**.
Only verified inactive, rebuildable Cargo executables were reclaimed afterward;
source, dependencies and unique historical evidence remain. The candidate binary
has a losslessly verified gzip snapshot. Final free space is
**11.013 GiB**; recheck before further builds.
Emergency workspace/temporary floors remain 2/1 GiB; neither was reached.

## Remaining acceptance

Source `49aeac60e8794b0f7f0644cbcc0450a5a355054a` qualifies the unchanged `eval/underscore_print`,
`eval/wildcard_chain_print` and `eval/efq_arm_value` originals. Exact normalized
graphs/text preserve distinct unused wildcard binders, a selected empty-match
function and the separately blocked named call. Twelve complete wildcard
applications, three partial applications and two Boolean-to-function selections
have separate evidence. No inhabitant of `Empty` is fabricated; the empty-match
function is never applied. No production or public API change was needed.
[Report](cpu-wildcard-values.md).

Clean-source gate: **977 passed, zero failed, 49 ignored**; focused controls:
**289 passed, zero failed, one ignored**. The open/applicability matrix records
62 observations; 16 paired graphs include renewed `partial_app_snf`; the closed
matrix records 36 observations for the two wildcard wrappers only. All original
executable printers refuse function-containing results exactly. Native executable
wrapper observations are separately covered by the persistent test; pure CLI
normalization is not relabeled as native executable evidence.

All 1,302 checking statuses and streams match the preceding port receipt; all
439 expected negatives reject. Fourteen positive controls pass; 18 observations
retain three unwaived hole/TODO cases. Private-effect wildcard refusal, source-hole
refusal, sharing, demand order, cancellation and resource guards remain under
regression. The new differential matrices contain no IO or shift programs;
earlier oracle receipts retain their actual source and lane identities.

Detailed receipts link **335 originals**; **967** lack detailed receipts:
**358 checker negatives, one checking-only module, 68 interpreter cases and 540
compiled candidates**. Three interpretation rows are added; earlier 146
interpretation, 94 runtime and 92 checking rows retain their identities. Linked
rows include partial/blocked cells and do not establish full CPU parity.
Imported-printer/JS-timeout, erased-hole/TODO, host/resource/foreign/F32.read/
error-order/API gaps and the unfinished 688-vector exploration remain unwaived.
Dependencies, references, toolchains, guards, budgets, cancellation and approved
demand policy remain frozen; no installs, desktop or GPU work. Publication is
recorded separately after explicit approval.

Next source-reviewed batch: `eval/open_closure`, `eval/open_closure_pipe` and
`eval/stuck_split_mint`. The first has an IO main; the others exercise open
higher-order bindings and blocked computed calls. Only source/inventory review
is complete; applicability and execution remain unqualified.

The [acceptance criterion](cpu-parity-acceptance.md) remains unchanged: a linked
receipt, matching rejection status or passing repository suite is not full parity.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `90b4aa3201abb48946d94044646b9eb0764357e4aba234eed360e994382649bc` |
| Frozen Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| snapshot.json | `08326cfcec46d4dc5087a277d88156495659d085e958993d6e643093d0ec1ceb` |
| snapshot/teamy-bend.gz | `db70203f3c4a40b52ca1e305ac77dc0a5f89af40518d142b0325053fcc1234eb` |
| focused.json | `0873c12c141984dfe00250fd92c971f4faab5a30f7ffd1e3106159028fca7131` |
| full-gate-final.json | `ec10674d50aeb1311d690ad970506e5ab8c71a361dfc03f57de76553fa4bd31d` |
| full-gate-final.log | `7e652b2bd71189eb6846dc24661a46c5a49d696cdc4bdb5f9a1f24de502a5ed7` |
| open-matrix/receipt.json | `8da80e1efc1e616db7c9ab068ad5519ca032cb8e0c3dee009f3d912dc2ca845c` |
| graphs-matching/receipt.json | `88302973e916399eb8798d9d85980bd052acdd444d92363e755d92c1d6e417a5` |
| closed-runtime/receipt.json | `f4db95a56a4e78f5a7c2980083f4c42ed5fa209811b4327eca236da30a0fd8f7` |
| audit/receipt.json | `78e12ad409796af1d8a33f18e3daa1ae6f084db4f7ce85d71830b1f169f7a5e3` |
| audit-delta.json | `31fcf4080bf4971d775d3c4ce9e9cc4586c485d4f650806355fc1d6d3e07fde2` |
| controls.json | `6517b7ae5bdcd9eb0132803aefa5eb844b207b83cd9bebec453fb7a36740f04a` |
| coverage-summary.json | `2d1e3e67fad41733066a4aa82c2a4485761baee1401b1b19a6becab2d795491a` |
| inactive-test-cache-reclaimed.json | `70db9a45ab2aa60786163b4161dd45f646746873bbeff6650b8e47805f79cfa5` |
