# Parser application, grouping and spacing qualification

Coverage source: `ffebc4a3aec2fce5f40ecf8b45d31a82a6c795e9`, local branch
`qualify/cpu-parse-chains-20261023`, based on `467de3c`.
Target remains [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
No production defect was demonstrated. This checkpoint adds exact original
fixtures, native output/proof-boundary regression coverage and repeatable runtime
qualification. Evaluation, approved demand semantics, error ordering and safety
limits are unchanged. Dependencies, lockfile and tools remain frozen; no push.

## Applicability and exact outcomes

All three originals import Base, have a main and have no foreign companion.
The pinned [test selection](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L48)
therefore selects C/JS candidates in addition to checking/interpretation.
Actual generation succeeds for both upstream targets. The pure Boolean main
also passes [show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204);
the two IO mains use its IO path. No function-printer exclusion applies.

| Original fixture | Exercised main | stdout bytes | Native / port C / port JS / upstream C / upstream JS |
| --- | --- | --- | --- |
| [adjacent_application](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/adjacent_application.bend) | Whitespace before ordinary application parentheses | `True{}\n` | All exit 0, empty stderr, twice |
| [chain_paren_group](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_paren_group.bend) | U32 multiplication inside a Nat expression and call argument | `7\n` | All exit 0, empty stderr, twice |
| [chain_spacing_rules](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_spacing_rules.bend) | Spaced modulo under a U32 suffix | `2\n` | All exit 0, empty stderr, twice |

These are **30/30 exact generated/native observations**. The originals also
contain tuple/list/array annotations, nested type closers and reusable Nat
binders. Their presence exercises loading/checking; functions not called by
main are not thereby qualified as runtime behavior. Their companion fixtures
remain separate inventory entries.

Direct upstream CLI entry is recorded separately. `adjacent_application` agrees
twice. The two IO fixtures each exit 1 with empty stdout and exactly
`TypeError: require is not a function\n`, twice. Pinned
[io_run](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1271)
passes Bun's `import.meta.require` to a generated function; this Node host does
not supply it. Separately emitted CommonJS programs run successfully under Node.
Thus four direct-entry observations are host-blocked, not passes or port defects.
The runner records `blocked_import_meta_require` and leaves whole-fixture
`matched` false for those rows. It rejects any different unclassified failure.

The historical strict-audit refusals of the two IO programs also do not describe
their executable behavior. The port's strict `check` loads its limited proof
Base, whereas execution loads the separate effect contracts. Preflight strict
checking accepts the Boolean program and refuses both IO programs with
`main: undefined name IO`. The regression explicitly retains this proof boundary;
no IO admission or false-proof acceptance was added. CLI `run` still emits raw
program bytes even when `--output-format json` is selected.

## Profiles and evidence

For these Linux CPU fixtures, upstream C uses the literal pinned
[cli_build](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253)
profile: `-std=c11 -O3 SOURCE -lpthread -lm -o EXECUTABLE`.
Execution uses `--threads 1 --gpu off`. No optional platform/GPU library is
needed. Both comparison runners now record the link flags as well as O3.
The earlier `20170de` matrices used O3 with `-pthread -lm -ldl`; they retain
those historical identities. Port C retains strict
`-Wall -Wextra -Werror -pedantic` and its existing link flags. Nothing suppresses
warnings or relaxes a comparison. Every child has a 60-second bound.

The renewed standard gate passes **807 tests, zero failed, 49 ignored**;
strict all-target/all-feature Clippy passes. The focused parser/effects/native
suite passes **34 tests**. All-feature CLI build and real Node-provider TCP/UDP
checks pass. The 49 ignored checks remain 47 GPU tests and two optional profilers.

The positive matrix has **338/350 exact primary executions**, with twelve known
Bun-FFI host failures; six additional direct-entry attempts give two exact and
four host-blocked results. Runner exit 0 means all outcomes were classified, not
that blocked cells passed. The six-fixture negative matrix remains **44/56 exact**:
eight resource-policy differences, two native foreign refusals and two Bun-host
failures remain open; runner exit 1. Ten authored demand controls remain
**92/100 exact**, six accepted C wildcard exceptions and two unresolved Nat-first
error-order differences; runner exit 1. No new waiver or diagnostic ordering
change was made. All repetitions were stable, with zero timeouts; the longest
recorded comparison child took 1.219 seconds.

The initial full gate ran while a commit/rebuild changed the shared CLI's embedded
revision. Its version test expected the preceding build's revision and failed;
799 tests passed before that single failure, 49 were ignored, and subsequent
tests were not run. The final gate was rerun with no overlapping builds or commits
and passed in full. Both logs and the focused post-correction run are retained;
no test assertion was weakened to handle the mixed-build failure.

Exact commands, after activating the existing environment and setting
`CARGO_NET_OFFLINE=true BEND_GPU=off`:

```sh
cargo test --features extended_observability --test cpu_parse_chains --test cli_effects --test parser
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/positive" -Suite all -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/negative" -Suite negatives -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/demand" -Suite demand -Repetitions 2 -OracleOptimization O3
```

`REFERENCE` names the clean pinned checkout; `EVIDENCE` is a fresh private
directory outside it. Receipts preserve literal commands, source and binary
hashes, stdout/stderr bytes, status, elapsed time, repetition and timeout fields.
No reference source was modified. Initial Clippy rejected `.err().expect()` in
the new test; it was replaced with `expect_err` and the affected tests/lint
rerun. Both logs are retained. No runtime expectation changed to make it pass.

Retained receipt SHA-256 values:

- positive: `8e05a47bc6e83a216eeb3e40347bfda40c4999815cd3a9716e806902bc0a79d7`.
- negative: `65849e00a4bd628c46b9197eee07e46ec3540bb91c1eec50ba562e6fde6562d5`.
- demand: `258b23d48b6e60c61dc8a56842b6a212c4d4e16080223d70bbbdf6646728474f`.
- Baseline `20170de` preflight: `4bc1a2e3e8c0444b69e7fce5fd03173e60b2700592311e5e519c469fac7bd237`.
- Clean-commit runtime binary: `bf33e7c0beef3237adc08ee3bc6aca438e46b95b6fa0a0d4cb62992b9656d80a`.
- Unchanged lockfile: `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

## Remaining acceptance and next family

The runtime ledger now links **54 originals**, and the unchanged interpretation
ledger links eighteen, for **72 distinct originals**. The remaining **1,230**
entries comprise 439 checker negatives, twelve check-only modules, 153
interpreter-only cases and 626 compiled candidates. These are applicability
categories, not a count of independent CPU runtime defects. Thirty-eight runtime
originals receive current receipts; only the three new direct-entry rows were
renewed, while existing direct-entry flags retain their earlier observations.
Sixteen comptime originals retain `208b343`
and their recorded no-optimization-flag profile. Interpretation stays at
`20170de`; ordering research and historical O2 columns retain `6e5bd61`.

Full CPU parity is still incomplete under the unchanged
[acceptance criterion](cpu-parity-acceptance.md). The competing-error ordering
discrepancy remains unresolved and unwaived. Only the three reviewed private
wildcard controls have an accepted upstream-C exception. Resource-policy,
native foreign, missing oracle-host and broader lane/API coverage requirements
remain open. The `KernelError` private-field and `run_main` `Ok(1)` downstream
consumer caveat remains unqualified. GPU, fresh Windows/macOS and broader worker
profiles were not run in this checkpoint.

Next independent ledger-selected family: `parse/chain_brace_suffix`,
`parse/chain_default_nat` and `parse/chain_suffix_u32`. Source inspection shows
Base-importing IO mains exercising brace annotations, default Nat precedence,
and U32 suffix propagation through nested expressions/call arguments/reusable
bindings. Their golden stdout is `5\n`, `7\n` and `16\n`, respectively; those are
source expectations, **not new execution receipts**. Establish actual printer
applicability and compare all available lanes before qualifying them. No new
dependency, GPU capability or pending semantic decision is required to start.
