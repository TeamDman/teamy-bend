# Repeated prefix parsing parity

Tested source: `ead74b8db6e720be3047a434685cbaf55dc69dac`. The following checkpoint contains reports only.
Upstream: `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
**The generic `1n++++k` acceptance/runtime gap is fixed. CPU parity remains incomplete.**

## Contract and correction

The pinned parser consumes [one unary plus at a time](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1788).
Its [Nat literal parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2239)
first consumes one immediately adjacent plus before parsing the tail. Consequently
`1n++++k` is a successor with three reusable-prefix markers on `k`; it is neither
four additions nor unrestricted numeric unary plus. Repeated markers also apply to
reusable lambdas, lets and datatype quantities. Fresh IDs from different marker
counts need not coincide; equal-count compact/spaced forms are compared exactly.

The port lexer retains maximal-munch tokens. One contextual helper now consumes a
single prefix character for both unary and Nat-tail grammar positions. It restores
the complete token before returning either success or failure, keeping template
snapshots replayable. Global lexing, infix `++`, declaration quantities, checker and
runtime policy, term identity/sharing, cancellation boundaries and safety budgets
are unchanged. Independent review found no blocking issue; its suggested direct
nested-error token-restoration regression was added and passed.

## Qualification

Twelve authored positive fixtures cover four/eight markers, compact/spaced/newline
tails, zero, constructor binders, reusable lambdas and annotated lets, quantified
ADTs, template replay/cache and nested `>>` closers. The integration test compares
complete canonical value graphs against the pinned interpreter, twice. The process
matrix separately renews those oracle captures; it is not an additional port graph
assertion. Native Rust, generated port C/JS and upstream C/JS run twice per case.
`nat-four` yields `4n`; the template yields `[4n, 6n, 6n]`.

The IO companion prints `4\n` in all five executable lanes. Strict check/eval refuse
IO as designed; direct upstream Node fails `TypeError: require is not a function`.
Those failures do not establish runtime parity. String append retains identical
complete values and C/JS text `"abcd"`; the existing native constructor display
remains an unresolved presentation discrepancy.

Ten malformed/negative cases retain rejection at check/eval/run and all three
executable-emitter boundaries, with no output artifact. Four diagnostics match
upstream exactly: ordinary affine reuse, spaced-before-plus, grouped literal and
unannotated let. Six differ: append-prefix, constructor-name prefix, malformed tail,
number/string prefix and declaration quantity. Those are unqualified diagnostics,
not new exceptions. Both source-hole/affine/non-Data refusal and previous guards
remain enforced. Default test-thread checks cover 200 nested markers, 128/129 Nat
literal bounds, equal marker-count graphs, template cache reuse and exact checker
work; aggregate sharing and cancellation/work guards remain in the full gate.

| Validation | Result |
| --- | --- |
| Standard gate on clean source | **1,155 passed / 0 failed / 49 ignored** |
| Format, strict production Clippy, all-feature build, Node provider | Pass |
| Focused parser/template/graph/runtime/normalization/aggregate tests | 91 passed |
| Prefix/default-stack/error-path and adjacent runtime units | 8 passed |
| Focused integration Clippy | Pass |
| Matrix | 490 observations, deterministic repeats, zero timeouts |
| Checking phase controls | 92 observations, separately classified |
| Full checking census | 1,302 cases; all statuses, phases and raw streams unchanged |

Focused runs used the reviewed dirty candidate; the clean-source full gate renews
all those tests at the source SHA above. All 49 ignores remain unrun. Supplementary
all-target Clippy was not rerun; its historical failures remain open. No GPU/desktop
checks, dependency changes, stack overrides or increased limits occurred.

Observation categories: `{"build_only": 26, "exact_execution": 128, "exact_interpreter": 24, "exact_rejection": 72, "full_graph_oracle_capture_port_graph_asserted_in_tests": 24, "generation_only": 52, "strict_IO_proof_refusal": 4, "strict_check_or_value": 48, "unresolved_Node_host_failure": 2, "unresolved_diagnostic_difference": 108, "unresolved_native_String_presentation": 2}`.
Compilation/generation are not execution. Rejected cases have no executable lane;
failed or unavailable oracles remain failures. [Exact compact failure receipts](cpu-prefix-markers-failures.json)
and [per-case evidence](cpu-prefix-markers-evidence.csv) retain distinctions.

## Census and remaining work

Counts: `{"acceptance_disagreement": 494, "both_accept": 366, "different_rejection": 191, "exact_rejection": 245, "oracle_or_infrastructure_failure": 6}`.
The whole pinned inventory has no source containing the newly covered four-plus
form, and all phases, statuses and raw output streams match the predecessor exactly. This is
new adversarial/general grammar coverage, not a reduced census residual count.
There are still **300 differences outside the 385 historical IO exclusions**, plus
six upstream Node stack failures. The historical 302 classification remains
151 diagnostic candidates / 109 acceptance differences / 42 other; its current
residual categories remain 149 / 109 / 42. The new IO symptom exposed in the prior
Nat-tail task stays unwaived. [Current census](cpu-prefix-markers-census.json) and
[triage](cpu-checking-triage.csv) preserve baseline and current identities.

Family links remain 411 linked / 891 unlinked: 341 checker-negative, one checking-only,
13 interpreter-profile and 536 compiled candidates. No original fixture received a
fabricated runtime link. Links include historical/partial/blocked evidence and do
not satisfy the [CPU acceptance criterion](cpu-parity-acceptance.md) by themselves.
F32.read, competing-error order, erased-hole/TODO, unavailable Bun/foreign hosts,
printer/oracle failures and existing external API caveats remain unresolved.

Next substantive investigation: `base/list_fold.bend` accepts upstream but the port
rejects `List.for_each(~...)` during loading. Determine bundled template/IO intrinsic
applicability and minimize a pure companion before changing anything. A later
strict-proof IO refusal would remain separate from execution and would not
automatically become an exclusion. No correction or additional
qualification of that family is included here.

## Reproduction and provenance

Use the frozen Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, Clang 19.1.7
and PS 7.5.4 environment. These temporary host build settings control Rust debug
artifacts, not Bend runtime optimization or safety policy:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --offline --features extended_observability --test cpu_prefix_markers --test cpu_reusable_nat --test parser --test parser_templates --test kernel_open_closures --test kernel_normalization
cargo test --locked --offline --features extended_observability --lib prefix
cargo clippy --locked --offline --features extended_observability --test cpu_prefix_markers -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

With `REF` pointing at the unchanged pinned upstream checkout and `OUT` at a fresh
scratch directory, the primary O3 comparison for any positive fixture is:

```sh
CASE=tests/fixtures/cpu-prefix-markers/positive/nat-four.bend
node "$REF/bend2/main.ts" "$CASE"
target/debug/teamy-bend check "$CASE"
target/debug/teamy-bend eval "$CASE"
target/debug/teamy-bend run "$CASE"
node "$REF/bend2/main.ts" "$CASE" -o "$OUT/upstream.c"
node "$REF/bend2/main.ts" "$CASE" -o "$OUT/upstream.js"
target/debug/teamy-bend compile "$CASE" --executable --target c --output "$OUT/port.c"
target/debug/teamy-bend compile "$CASE" --executable --target javascript --output "$OUT/port.js"
"$TEAMY_BEND_CC" -std=c11 -O3 "$OUT/upstream.c" -lpthread -lm -o "$OUT/upstream"
"$TEAMY_BEND_CC" -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$OUT/port.c" -pthread -lm -ldl -o "$OUT/port"
"$OUT/upstream" --threads 1 --gpu off
"$OUT/port" --threads 1 --gpu off
node "$OUT/upstream.js"
node "$OUT/port.js"
```

Each process-matrix generation/build/execution has a 30-second bound; runtime,
checking and oracle graph observations repeat twice. Integration C compilation is
separately strict and unoptimized. The census uses the committed
[runner and phase probes](../scripts/checking-census/README.md), 30/60-second bounds,
and a recorded clean-source build receipt. No command executes a checking-only main.

Binary SHA-256: `c8230fde9ec840145c7c397663848a3ca84bc89f4eeb20f8a6e7ef82276742aa`.
Matrix receipt: `5ba79249d60d25cb1030ed81774a9bb006e07c5bbc6a6727b0c42b58bb3f234b`.
Census raw receipt: `d8416cc4feafd4546d8f9ded42e0aef444ddb1e7d0e2f27003314bdeef00c207`.
Cargo.lock remains `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`; docs remain
`50e89d6897ce2a65167ef65b17c5fa83fe849158`, both reference repositories clean.

The full gate began with 11.28 GiB free and used no less than
10.22 GiB; its enforced floor was 10 GiB (11 GiB preflight), with
at least 1 GiB free in temporary storage. Cache growth was rebuildable integration
and unit test executables. After the completed gate, inode/hash and active-process
checks reclaimed only inactive caches; headroom became 11.35 GiB.
Source, compact expected graphs/failures and historical evidence were preserved.

Reviewed isolated feature-branch publication preserves source, tests and these
compact receipts. Bulk command logs, generated C/JS artifacts, complete census
observations and captured binaries remain outside Git in the task evidence store;
Git preservation does not back up those unique raw artifacts. No main merge,
force push, package publication or release.
