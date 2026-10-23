# Brace and numeric operator-chain qualification

Coverage source: `ef48b992be62ddfd34f20bdbae5649f331dcbe15`, local branch
`qualify/cpu-parse-operators-20261023`, based on `8f16bb5`.
Target: [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
All three originals agree across the five primary runtimes. No production defect
was demonstrated; changes only expand the comparison runner, attributed fixtures
and existing regression cases. Dependencies, toolchains, budgets, approved demand
semantics and error ordering remain unchanged. No installation or push.

## Applicable lanes and exact results

All three import Base, have IO mains and have no foreign companion. Pinned
[test selection](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L48)
selects C/JS candidates. Both upstream targets actually generate successfully;
[show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
takes its IO path. No function-printer exclusion was assumed.

| Exact original | Behavior exercised by main | stdout bytes |
| --- | --- | --- |
| [chain_brace_suffix](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_brace_suffix.bend) | A U32 operator suffix inside braces that annotate the result | `5\n` |
| [chain_default_nat](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_default_nat.bend) | Default Nat operators and multiplication precedence in `a + b * c` | `7\n` |
| [chain_suffix_u32](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_suffix_u32.bend) | U32 suffix propagation through nesting, call arguments, a let, shifts/bitwise operations and a reusable binding | `16\n` |

Native Rust, port C, port JS, upstream C and emitted upstream JS all exit **0**
with exactly these stdout bytes and empty stderr, twice: **30/30 exact primary
executions**, stable repetitions and zero timeouts. Original comments and bytes
are preserved under their Apache-2.0 attribution. Uncalled helpers in the brace
and default-Nat fixtures were loaded/checked; their runtime behavior is not
qualified by main. Companion fixtures remain separate inventory entries.

The direct upstream IO entry exits **1**, with empty stdout and exactly
`TypeError: require is not a function\n`, for all three, twice. Its
[io_run](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1271)
passes Bun's `import.meta.require` into generated code, unavailable on this Node
host. Emitted CommonJS JS works. These six direct observations remain host-blocked;
whole-fixture `matched` stays false despite primary-runtime agreement. No Bun was
installed and no reference source was adapted.

The existing regression tests now exercise all six parse-chain originals. They
preserve raw CLI program output under `--output-format json` and separately
verify that strict proof checking still refuses the five IO originals with
`main: undefined name IO`. The proof Base intentionally has a different surface
from executable loading; these refusals are not executable failures or parity
passes. Expanding the cases leaves the number of Rust test functions unchanged.

## Validation and reproducibility

The full standard gate passes **807 tests, zero failed, 49 ignored**. Strict
all-target/all-feature Clippy passes; focused parse-chain/parser/numeric tests
pass **31 tests**. All-feature CLI build, formatting and real Node TCP/UDP
provider checks pass. The 49 ignored checks remain 47 GPU tests and two optional
profilers. Builds and commits did not overlap the full gate.

The renewed positive matrix covers 38 fixtures, including three authored controls:
**368/380 exact primary observations**, with twelve existing Bun-FFI host failures.
Twelve extra direct-entry attempts give two exact and ten host-blocked results.
All recorded repeats are stable and no child times out; each child is bounded at
60 seconds. Runner exit **0** means every outcome met its recorded classification,
not that host-blocked cells passed. Upstream C uses the pinned Linux
[cli_build](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253)
profile `-std=c11 -O3 SOURCE -lpthread -lm -o EXECUTABLE` and executes with
`--threads 1 --gpu off`. Port C retains strict warning and existing link flags.

No production or shared comparison logic changed, so the negative and demand
matrices were not repeated. Their latest `ffebc4a` receipts remain explicit:
negative **44/56 exact**, eight resource-policy, two native-foreign and two
Bun-host gaps, runner exit 1; demand **92/100 exact**, six accepted C wildcard
exceptions and two unwaived Nat-first diagnostic-order differences, runner exit 1.
No historical failed or blocked cell is newly marked passed.

Exact commands, with the existing environment activated and
`CARGO_NET_OFFLINE=true BEND_GPU=off`:

```sh
rustup run nightly-2026-05-24 cargo fmt --all -- --check
cargo test --features extended_observability --test cpu_parse_chains --test parser --test parser_numeric
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/positive" -Suite all -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

`REFERENCE` names the clean pinned checkout and `EVIDENCE` a fresh private directory
outside it. The optional focused runner uses `-Suite parse-operators`. Exact
commands, source/binary hashes, statuses, raw streams, timings and timeout fields
are retained in its receipts. Exploratory preflight used the prior `ffebc4a`
binary while adding the cases on checkout `8f16bb5`; the final repeated receipt
uses the clean `ef48b99` binary. Those identities are not interchangeable.

Retained SHA-256 values:

- Repeated positive receipt: `4f71620eaa6817c1e5effd9293ecc93d6d1a6a0e9830c0bac69fcc69bfcbce3a`.
- Clean candidate binary: `39903ad9ad430719ce1af13ba98a403365a8feb0a4b732b60e48cc21dd38dfee`.
- Exploratory preflight receipt: `20b5b448fa5c512f2e9075c0b5bdcf1911cd2be08181d036bb6dbcbd20256e41`.
- Unchanged lockfile: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

The longest recorded comparison child took 1.085 seconds.

## Acceptance and next independent family

The runtime ledger links **57 originals**: 35 renewed here, six negatives at
`ffebc4a` and sixteen comptime originals at `208b343` with their recorded profile.
The eighteen interpretation originals retain `20170de`. The union is **75**;
**1,227** entries lack these receipt kinds: 439 checker negatives, twelve check-only
modules, 153 interpreter-only cases and 623 compiled candidates. This is an
applicability distinction, not a denominator of runnable CPU defects. Six parse
direct-entry rows were renewed; older collection direct-entry observations keep
their historical provenance.

Full CPU parity remains incomplete under the unchanged
[acceptance criterion](cpu-parity-acceptance.md). Host/resource/foreign gaps,
competing-error ordering, exact checker diagnostics and complete lane/API review
remain open. The earlier `KernelError` private-field and `run_main` `Ok(1)`
downstream-consumer caveat is still unqualified. No new exception was adopted.
GPU, fresh Windows/macOS, additional worker profiles, the full strict audit and
older normalization/comptime matrices were not rerun for this test-only change.

Next independent ledger-selected family: `parse/chain_suffix_f32`,
`parse/float_exponent`, `parse/float_pattern`. Source inspection identifies F32
operator suffixes, exponent parsing/printing and float-literal matching. Source
goldens are `7.5\n`, `0.0025 1000 150\n` and `1\n`; none is yet qualified by this
checkpoint. All are compiled candidates. The float-pattern comment claims a
compiled structural-word refusal, so establish actual generation behavior before
assigning its lanes; neither that comment nor a strict-proof refusal is enough.
