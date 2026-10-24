# Dotted bitwise operators and brace terms

Later checkpoint: [reflexivity diagnostics](cpu-reflexivity-diagnostics.md) at
`f317911` renews and closes the specific false-proof diagnostic gaps described
below. This document retains its original tested source and historical receipts.

Source `d5d0585bbed2fcd99babb9ce175a1c3796d66b27` qualifies the unchanged pinned
parse/bitwise_dot_operators and parse/brace_term_forms originals across available
CPU lanes. No production correction was needed. The original bitwise IO direct
host remains blocked, and an adjacent false-proof control retains the known
checker diagnostic mismatch. Neither is waived; full CPU parity is incomplete.

## Actual applicability and outputs

Both originals were submitted to the pinned C and JavaScript printers before
runtime qualification. All four generation attempts succeeded; the final matrix
repeats generation and executes the artifacts. C uses the upstream CLI O3 profile,
`-lpthread -lm`, and `--threads 1 --gpu off`; port C retains strict warnings.

| Program | Exact stdout including LF | Execution evidence | Strict proof mode |
| --- | --- | --- | --- |
| Original bitwise_dot_operators | `671\n` | Five lanes match twice; direct upstream IO blocked twice | Whole IO book excluded |
| Original brace_term_forms | `42\n` | Six lanes match twice | Pure main and named definitions pass |
| Bitwise pure-entry derivative | `671\n` | Six lanes match twice | main and main.out pass |

The five lanes are native, port/upstream C and port/upstream JavaScript. Pure
mains additionally use direct pinned interpretation. Successful executions exit
0 with empty stderr; every repetition is deterministic and no timeout occurs.
The bitwise original's Node direct-IO invocation exits 1, with empty stdout and
exact stderr `TypeError: require is not a function\n`. It remains
`blocked_import_meta_require`, distinct from unavailable Bun-FFI lanes.

Strict eval of the bitwise original, selecting main or main.out, rejects the
whole IO book: `main: undefined name IO`, exit 1 and empty stdout. Its four
repeated exclusions are deliberate proof-mode boundaries, not runtime passes.
The Apache-attributed derivative replaces only the IO main with a pure U32 main
calling the unchanged helper. The two originals remain byte-identical; the
derivative adds no inventory row.

The complete execution capture has **34 exact observations and two host blocks**.
The public runner exits 0 for its completed lanes, but keeps matched=false and
completed_runtimes_match=true for the bitwise original. The supplemental
qualification returns 1 for the host block and diagnostic mismatch described
below. No required lane is silently counted as passed.

## Normalized structure, text and false-proof behavior

| Definition | Pinned and native normal form |
| --- | --- |
| Brace main | `42` |
| Brace refl | `{==}` |
| Brace app | `f => x => f(x)` |
| Pure bitwise main | `671` |
| Pure bitwise main.out | `671` |

Each of these five definitions has two pinned normal-form observations, two
strict JSON entry/value reports and two actual-core comparisons: **30 matching
observations**. Core comparison is separate from pretty-text equality, with no
rewriting or weakening of the comparison vocabulary. The oracle adapter loads
and checks the selected original or labelled derivative, then normalizes a named
definition using the pinned Bend functions. It executes no IO main and changes no upstream source. A named
function helper is not a claim that a returned function can be printed by C/JS;
the original brace main is a printable U32 computation.

One adjacent control changes the unused equality from U32.add(1, 2) == 3 to
U32.add(1, 2) == 4. Parsing succeeds, then both checkers reject reflexivity at
`refl`; no execution starts. Upstream CLI, port run, strict check and strict eval
all exit 1 with empty stdout, twice: eight rejection observations. Two structured
oracle probes confirm checking phase, definition refl, normalized endpoints 3/4
and Bend line 9. Their adapter exit 0 reports an observed rejection, not an
accepted proof.

Raw stderr differs. Upstream displays normalized endpoints 3 and 4 plus the Bend
source line; the port displays `U32.add(1, 2) != 4` and host diagnostic context.
This is another instance of the [existing checker formatting gap](cpu-default-nat-functions.md#wrong-reverse-and-lost-duplicate-diagnostic-cause),
now explicitly linked in the [checking-control ledger](cpu-checking-control-evidence.csv).
The invalid proof stays rejected; phase agreement does not qualify raw diagnostic
parity. No formatter, semantic exception or diagnostic suppression was introduced.

## Validation and retained scope

| Check at d5d0585 | Result |
| --- | --- |
| Standard Linux gate | **838 passed, 0 failed, 49 ignored**; exit 0 |
| Focused integration checks | **29 passed**, including four new tests |
| Internal guard checks | **5 passed** |
| Pinned formatting, strict all-target Clippy, all-feature build | Passed |
| Three-program public comparison | Completed lanes match twice; exit 0 with one explicitly blocked direct-IO lane |
| Complete bounded qualification | 34 exact executions, two host blocks, four proof exclusions, 30 normal-form/core/report matches, eight false-proof rejections and two phase probes; exit 1 |
| Five-fixture baseline | **50/50 exact executions**; exit 0 |

The focused/full suites retain false-proof, work/depth refusal, cancellation,
first-error, carry/wrapping and partial-demand controls. No production file,
evaluator selection, dependency, toolchain, safety budget, guard or approved
demand contract changed from 0f83aea. The four new tests cover original and pure
output, actual normalized core/text, the IO proof boundary and unused false
reflexivity. No broader implementation rewrite was needed.

The final full gate and comparison receipts identify d5d0585. Initial printer/
CLI observations used the immutable binary from 0f83aea and are labelled
separately. Focused/format/strict-Clippy checks preceded the source commit using
identical contents. All earlier evidence remains intact.

The 42-fixture normalization matrix, full checking-status audit, negative/demand,
read/comptime and broader positive differential matrices were not renewed; their
receipts keep their historical commits. GPU and fresh Windows/macOS validation
were unrun. The 49 ignores remain 47 CUDA checks and two optional profilers.
Existing resource, foreign/Bun, F32.read, competing-error, host/API/platform and
checker-diagnostic requirements remain open.

Five runtime rows are current: two new originals and three baseline originals.
The remaining 76 runtime rows and all 38 interpretation/ten demand rows retain
historical receipts. The two prior checking controls remain at 0093d01; only the
new brace control is current. Runtime/interpretation union is **119 originals**;
1,183 remain unlinked: 439 checker negatives, twelve check-only modules, 153
interpreter cases and 579 compiled candidates. Applicability review and every
required lane/API cell remain prerequisites; these are not runtime defect counts
or a completion percentage.

## Reproduction and resource evidence

Use the existing activation and frozen CARGO_NET_OFFLINE=true / BEND_GPU=off
environment. All output directories must be fresh.

```sh
cargo +nightly-2026-05-24 fmt --all -- --check
cargo test --features extended_observability --test cpu_bitwise_braces --test cpu_parse_chains --test parser_numeric --test cli_proofs --test executable_check --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo test --features extended_observability --lib depth_tests --quiet
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <programs> -Suite parse-bitwise-braces -Repetitions 2
python3 <private-evidence>/qualify.py
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <baseline> -Suite baseline -Repetitions 2
```

Children have 60-second comparison deadlines. Private receipts retain complete
command arrays, original/derivative/source/binary hashes, raw stdout/stderr,
status, determinism, timing, timeout and child-exit information. Public notes
omit machine-specific captures.

Preflight first reclaimed inactive root test products, but stopped below the
12-GiB reserve. It then reclaimed only three obsolete example binaries and two
stale linker temporary files under target/debug, with hashes recorded. This
recovered another 476,213,248 bytes of allocated space. Named current binaries,
source and all historical evidence were preserved. No snapshot deduplication or
evidence deletion occurred in this checkpoint. A fresh preflight preceded the
full gate; the gate ran before new snapshots, with 2-GiB workspace and 1-GiB
temporary-space stop floors. It took 177.82 seconds;
minimum headroom was 3.06 GiB workspace and 4.76 GiB temporary space. No exhaustion occurred.

| Receipt/input | SHA256 |
| --- | --- |
| programs | `ec635ba91a8998b149f1a6cb2149b6b5b006cbf67e1d60cbd95a534823e8b587` |
| qualification | `f49760c97f7b4bfb64e9c556863c1d8d226523dbb35100c33ec8b93d295b2300` |
| baseline | `04183e7ba264ebb0539e29b5504a7607720e573bca5e1894bf8accfa1b187f83` |
| gate | `224cd743ad4365f1cc52531a105b6d3ed05b030e6e60e4c1c052896e189d7718` |
| initial | `ff3c95d5df7d580b65b2bed56fd1f91b52e933763e10beae765783755c2c1fcf` |
| cache-before-build | `456ad180960f9b44dcf2ee97e70d21fbe52a226fd695bed0fb7ac824c0e928f7` |
| cache-extra | `82dfe4ceffcfbeb7c662a9e00aee876a8d7639899cb557091527adc7caec18fd` |
| cache-before-gate | `c79f46b11864b1abd4c771b6c8c1d870762a7651fe73cca802e067f46f5626df` |
| normalize-adapter | `50cb83be28bc5dbd282cd1706e91fea8510c7accb87dbad2909205a8f04153bb` |
| phase-adapter | `a1df21c8e44bd38e25ed3e83fe3111306564f3107b8e4013625bc16f257b0455` |
| false-refl | `b7203318fec4cbb57cdca838921554b7b0a1cb36115b6f61cbab03ae1bfac0ae` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| Port binary | `27f7bf03f111b370a33edc05bbda1be7a43e8050ee6c3c7696cc80dc4ab9e566` |
| Core probe | `231b8942c076cb76b28e723f27b2076bb55d9d3d9e0f668f5b4266107928161e` |

Next independent family: parse/cons_operator and parse/cons_plus_pattern, selected
from the applicability ledger. Their sources were inspected only here; actual
printer applicability, runtime results and normalization remain unqualified.
Both references and the lockfile remain frozen and clean. No packages were
installed, no desktop/GPU work occurred and nothing was pushed.
