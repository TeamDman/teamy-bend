# Float syntax, literal patterns and numeric printing

Implementation: `bd6a228c62799fb7a2c602a898e77af13bef8b58`, local branch
`qualify/cpu-parse-floats-20261023`, based on `ef2a220`.
Target: [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Three demonstrated defects are corrected. The three original fixtures and an
authored bit/pattern control now agree in all five primary runtimes twice.
Full CPU parity remains incomplete. No dependency, toolchain, pin, safety limit,
approved demand rule or unresolved error-order contract changed; no push.

## Actual applicability and exact results

All three import Base and are compiled candidates under the pinned
[gate selection](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L48).
Both upstream targets actually generate successfully. The float-pattern comment
claiming compiled structural-word refusal is stale; the original comment and
fixture bytes remain unchanged. The first two mains use IO; the third returns
U32, supported by the upstream
[result printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204).

| Exact upstream original | stdout bytes | Status and stderr |
| --- | --- | --- |
| [parse/chain_suffix_f32.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_suffix_f32.bend) | `7.5\n` | 0, empty |
| [parse/float_exponent.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/float_exponent.bend) | `0.0025 1000 150\n` | 0, empty |
| [parse/float_pattern.bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/float_pattern.bend) | `1\n` | 0, empty |

Native Rust, port C, port JS, upstream C and emitted upstream JS match these
bytes and statuses twice: **30/30 exact observations**. Direct upstream entry
also succeeds for the pure fixture. For both IO entries it exits 1, stdout empty,
stderr exactly `TypeError: require is not a function\n`, twice. The upstream
[IO driver](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1271)
requires Bun's import.meta.require, absent under Node. These four direct failures
stay host-blocked; separately emitted CommonJS succeeds. No upstream adaptation
or installation was made. Whole-fixture agreement does not erase these cells.

## Corrections and regression boundaries

The preserved clean `ef48b99` binary refuses float_pattern before execution in
all three port lanes: `pattern compilation exceeds the resource limit of 64 levels`.
The first-row pattern algorithm recursively counted both each Word constructor
and its nullary Bool column. A complete 32-bit pattern exhausted that depth even
though upstream accepts it. `src/syntax/surface.rs` now continues nullary positive
columns iteratively, retaining bounded pending fallbacks. It compiles fallbacks
after their positive arms and unwinds in reverse, preserving generated Mat shape,
fresh-name order and first-error order. Ordinary constructor recursion and the
64-level guards remain. This follows the pinned
[match compiler](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2679)
without a float-specific or fixture-name special case.

The former regression expected a valid U32-zero pattern to fail. It now tests a
genuinely deep 65-Nat pattern instead; a separate short/65-column Bool regression
checks that iterative pending work remains bounded. U32-zero success, wildcard
binding, F32 bit-pattern distinctions and an unselected overflowing fallback are
covered. The initial 24-pass/one-failure parser run records the obsolete expectation;
the corrected parser suite passes all 26 tests. No failing receipt was discarded.

Once parsing worked, native pure output exposed a presentation defect: it printed
the entire U32/Word constructor tree instead of `1`. `src/kernel/term.rs` now
recognizes only a complete literal 32-bit Word inside U32 and prints its unsigned
decimal value, following pinned
[term_show](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1369)
and [u32_from_term](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1147).
Printing evaluates nothing. Malformed widths, nonliteral tails and other
constructors stay structural; outer annotations remain. This affects complete
U32 terms wherever Display is used, including diagnostics. It adds no Rust API
signature. F32 term display and runtime F32.show remain unchanged.

The authored rounding control found a separate real bit mismatch:
`F32.bits(1.0000000596046448)` produced `1065353217` in the preserved baseline,
where upstream produced `1065353216`. Pinned
[parse_term_num](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2217)
uses `Math.fround(Number(token))`: decimal to binary64, then binary32. The port
parsed decimal directly to binary32. `src/syntax/parser.rs` now follows the
pinned two-stage conversion, with a narrowly explained truncation-lint expectation
and unchanged finite-value rejection. Runtime F32.read keeps its separate
binary32 parsing contract. The source fixture's “rounded once” wording does not
override the actual frontend shared by all pinned backends.

The authored `float-bits-and-patterns.bend` control checks exponent bits, midpoint
ties, the above-midpoint decimal, positive/negative zero distinction, literal
matches, fallback-bound values, unused overflow, and U32 zero/MAX boundaries.
Its exact LF-separated stdout is:

```text
992204554
1065353216
1065353218
1065353216
10
2147483648
20
1071644672
7
99
1
4294967295
```

All five runtimes reproduce it twice, exit 0, empty stderr: ten additional exact
observations. The authored control adds no upstream inventory row. Native CLI
regressions also verify raw program output under `--output-format json`.

## Renewed validation

All final receipts below identify the clean `bd6a228` binary. Current runtime,
interpretation and demand ledger cells were renewed because production parsing
and display changed. Historical checking, O2, worker, platform and failure-order
investigation receipts retain their original identities.

| Check | Result | Remaining boundary |
| --- | --- | --- |
| Standard repository gate | **810 passed, 0 failed, 49 ignored**, exit 0 | Includes formatting, strict Clippy, all-feature build and real Node TCP/UDP provider. Ignores remain 47 GPU and two optional profiler checks. |
| Focused float/parser/numeric tests | **54 passed, 0 failed, 0 ignored** | Includes 26 parser tests and adjacent runtime numeric contracts. |
| All-target/all-feature strict Clippy | Exit 0 | No warning-policy relaxation. |
| Positive matrix, 42 fixtures | **408/420 exact** primary observations; runner 0 | Twelve existing Bun-FFI host failures. Runner success means classifications held, not that blocked cells passed. Eighteen additional direct entries: four exact, fourteen import.meta.require failures. |
| Six runtime-negative originals | **44/56 exact**, runner 1 | Eight resource-policy, two native-foreign and two Bun-host gaps remain. |
| Ten authored demand controls | **92/100 exact**, runner 1 | Six approved C wildcard exceptions; two unwaived Nat-first competing-error differences. |
| Normalization, eighteen originals plus four controls | 176 successful observations, 28 expected upstream function-printer generation refusals; runner 0 | All originals match raw text and core structure twice with RequireExactText. No structural check weakened. |
| Sixteen comptime originals | **156/160 exact** primary observations; runner 1 | Default upstream JS still fails hand/tmpl twice each; four supplemental stack-4096 runs match. All three port lanes and upstream C pass 32/32 each. |
| Full 1,302-fixture strict audit | 366 expected positives accepted, 487 refused; all 449 nonzero-expectation fixtures refused; runner 0 | Includes all 439 checker negatives, zero unexpected acceptances, abnormal exits or timeouts. Status-only checking, not diagnostic or runtime parity. |

The two newly accepted strict positives are `parse/float_pattern.bend` and
`check/let_opaque_match.bend`; the latter contains a U32-zero pattern in a helper.
Its new checking acceptance is not a runtime qualification. The historical
applicability CSV and audit metadata keep the old 364/489 counts.

Every compared execution is repeated twice with stable outcomes; no comparison
or audit child timed out. Deadlines remain 60 seconds. The longest comparison
child took 2.097 seconds; audit maximum was 1.206 seconds. C execution uses
`--threads 1 --gpu off`. Upstream compilation follows pinned Linux
[cli_build](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253):
`-std=c11 -O3 SOURCE -lpthread -lm -o EXECUTABLE`. This includes the renewed
comptime matrix; its older no-optimization-flag receipt remains historical.
Port strict warning and link flags are unchanged.

Default upstream JS hand/tmpl still exit 1 with empty stdout and
`bend: memory fault (machine stack overflow?)\n`. The ledger's `pass_stack4096`
qualifies only that explicit supplemental host profile, never the default one.
The upstream-C nat_first result remains unresolved at all investigated profiles.
Only the three previously approved wildcard cases have an exception; the O0
Word register-allocation failure remains separate oracle build evidence.

## Reproduction and evidence identity

With the existing activation sourced, `CARGO_NET_OFFLINE=true BEND_GPU=off`,
REFERENCE naming the clean pinned checkout and EVIDENCE a fresh private directory:

```sh
rustup run nightly-2026-05-24 cargo fmt --all -- --check
cargo test --features extended_observability --test cpu_parse_floats --test parser --test parser_numeric --test runtime_numeric
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/positive" -Suite all -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/negative" -Suite negatives -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/demand" -Suite demand -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/normalization" -Suite all -Repetitions 2 -RequireExactText
```

The focused differential option is `-Suite parse-floats`. Retained private
`probe-comptime.py` and `audit.py` record each child argv, status, raw streams,
source/binary hashes and deadline. They reuse installed tools and read the pinned
reference. The audit invokes the saved binary with `--output-format json check`
for each original; the comptime runner generates both targets and executes each
runtime twice, retaining default and supplemental JS lanes separately.

The exploratory preflight ran the preserved clean `ef48b99` binary on checkout
`ef2a220`. Intermediate pattern/printer and rounding probes identify their
candidate state separately; the rounding mismatch was additionally reproduced
with that preserved baseline. Final repeated receipts use clean `bd6a228`.
Builds and commits did not overlap the full gate. Before rebuilding, disk
headroom was recovered by deleting only recorded rebuildable root-package test
executables; source, binary evidence snapshots and all prior logs were retained.

| Retained artifact | SHA-256 |
| --- | --- |
| Positive receipt | `c7958606bce0e2ca4f2b698b064752c419fc3cef8fbb795bff65ef01d5c46dde` |
| Negative receipt | `59646605d04721396f82c4b1b68404db7beb20bcff152e2689f804386a3627ce` |
| Demand receipt | `76901e51510eef1a1d1bb8cfd402d3a91ba120816f9246aeef32cc48433ea779` |
| Normalization receipt | `019fd97eea0884fc502187af50717ef1bef8e1c2114e9d41003eb0611721ea73` |
| Comptime receipt | `966440912e971dbaffd1f4a924150fcc4af348a40e6e02304cf580e870ad5780` |
| Strict audit receipt | `50f2167627dab1f576817878f21ba9d026a4dfb5cfe1a786e5f213a85a97f8db` |
| Preflight receipt | `12389d58ac6252979618b668f96173e0dbaa2a53a0f720469245034297f574f3` |
| Rounding preflight receipt | `4fab6eb183012ffcac2f67d4644c887cbf152f9ab24495c05e4c51a4226c6925` |
| Clean CLI binary | `2d24ac3bcb52456e7fc6c8fc3875ad998cbc22ec0850f86d001575ee8101ab9f` |
| Normalization probe | `a95da3c746585a75fa039762538eb35f1ce2cb60439fd3cd53d14e38d4bfafc9` |
| Unchanged Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |

## Acceptance and next family

The runtime ledger now links **60 upstream originals**, all renewed here:
sixteen comptime, ten run, eighteen IO, five Base, two representation and nine
parse. Eighteen interpretation originals also have renewed receipts. The union
is **78**; **1,224** inventory entries lack these receipt kinds: 439 checker
negatives, twelve check-only modules, 153 interpreter-only cases and 620 compiled
candidates. These are different applicable modes, not 1,224 runtime failures.
Nine parse direct-entry rows were renewed; older collection direct-entry cells
retain their original observations. Authored controls add no inventory rows.

The [acceptance criterion](cpu-parity-acceptance.md) is unchanged. Host/resource/
foreign cells, unwaived competing-error order, exact checker diagnostics, complete
lane/API review and downstream consumers of the KernelError private field and
run_main Ok(1) changes remain open. No new Windows/macOS, GPU or expanded-worker
qualification is claimed. There is no environment blocker to another bounded
CPU family.

Next source-inspected, **runtime-unqualified** family:
`compile/float_builtin_agreement.bend` (`-2 -1 -1 1.5\n`),
`compile/float_accumulation.bend` (`2775\n`) and
`compile/float_unboxed_ctor.bend` (`375\n`). These Base/IO compiled candidates
exercise signed rounding intrinsics, dynamic accumulation and F32 fields/array
swaps. Establish actual applicability before comparing all five runtimes.
Existing numeric unit coverage is supporting evidence, not an original-fixture
five-runtime receipt. Broader F32 term printing remains unqualified.
