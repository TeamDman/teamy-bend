# Nat literal round-trip qualification

Checkpoint source `4083b32159abd90e2c87ba5ab0e851b813349d70` qualifies the unchanged
pinned `eval/nat_literal_roundtrip.bend`. Production changes are in `3ab89e9`;
`4083b32` adds the last affected text expectation. Final comparisons and the
standard gate identify the latter source explicitly. No push occurred.

## Result and actual applicability

Pinned interpretation, native `run`, strict JSON `eval`, and actual normalized
core agree on this result, twice, with exit 0 and empty stderr:

```text
(4n, 0n, [3n, 2n], n => 1n+n)
```

Before the fix, core structure already matched, while the port printed:

```text
Tuple{4n, Tuple{0n, Tuple{Con{3n, Con{2n, Nil{}}}, n => Succ{n}}}}
```

The [original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval/nat_literal_roundtrip.bend)
is copied byte-for-byte, with its comments and golden intact and
[Apache-2.0 attribution](../tests/fixtures/cpu-normalization/README.md). Its comment
about broken chains printing plainly is stale: the actual pinned printer and
golden use `1n+n`. Actual C and JS generation both refuse the compound result
containing a function. Two repetitions per target yield four exits of 1, empty
stdout and no output artifact. The refusal names the nested Pair/List/function
type; it is not inferred from the comment's claim about lists.

This is strict checking and direct observation of the normalized core, with
raw text checked independently. No executable-only fallback is used. The existing
F32 executable-output comparison, its separate internal-core regression and its
strict proof exclusions remain distinct. The normalized compound is not a
runtime data result; a regression retains the data evaluator's rejection of
its function field. No generated C/JS execution is claimed for this original.

## Presentation repair and regression boundaries

The pinned [Nat](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1229),
[list](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1241)
and [tuple](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1252)
formatters recognize constructor names and arities. The port now prints an open
successor spine as `Nn+tail`, retains closed Nat spelling, prints complete Con/Nil
lists as brackets, and flattens only the right-hand Tuple spine into tuple syntax.
These are reads of existing terms, not evaluation, arithmetic or type inference.
No parser, evaluator, checker, code-generator source, demand contract or resource
limit changed. Constructor formatting was extracted into a small helper to keep
strict Clippy's function-size rule; no warning was suppressed.

Open Nat tails retain the required grouping in applications, lambda/let/rewrite
tails and arrow domains. Tests compare core structure independently of output
spelling and retain malformed arities, namespaced constructors, annotations,
holes and binder identity. The comparison vocabulary is unchanged. Separate
checks still compute `dbl(0n) = 0n`, `dbl(1n) = 2n` and `dbl(3n) = 6n`.
The standalone checked function `main(n: Nat) -> Nat: Succ{n}` now prints
`n => 1n+n` in both implementations, twice, with equal actual core.

Forty-one syntax printer examples agree with direct calls to the pinned
`term_show(term_lower(term_higher(parsed)))`, without running normalization.
They cover complete/open Nats, nested calls, binding, malformed and namespaced
constructors, and tuple/list layout. The 64-successor internal-core control is
constructed directly: the equivalent deeply nested source still hits the
existing 64-level parser input guard. That refusal remains tested, and is not
claimed to match upstream's larger accepted input space. Compact display does
not waive evaluation, parser, comparison or output budgets.

Open-list shorthand, other unqualified printer forms and arbitrary programs
are not covered by this result. Existing [API caveats](cpu-nat-failure-contract.md#review-of-rust-api-consumers)
and F32.read/error-order/resource/foreign/host/platform gaps remain open.

## Validation

| Check | Final result |
| --- | --- |
| Original Nat round-trip | Four exact interpreter/native outputs, two strict eval reports, two actual-core probes, four verified printer refusals; two repetitions |
| Full normalization | 32 fixtures: 28 originals + four authored controls; 250 successful observations, 68 printer refusals, six F32 proof exclusions; exit 0 |
| Standalone open Nat control | Eight successful commands; exact text and actual core agree twice |
| Direct pinned printer controls | 41 exact syntax renderings; exit 0 |
| Focused tests | **69 passed, 0 failed, 0 ignored** |
| Standard repository gate | **823 passed, 0 failed, 49 ignored**; exit 0 |
| Strict all-target Clippy / build | Exit 0 / exit 0 |
| Five-runtime baseline | **50/50 exact** executions across three originals and two authored regressions; exit 0 |
| Six runtime negatives | **44/56 exact**, twelve unqualified observations; exit 1 |
| Ten demand controls | **92/100 exact**, six approved C-wildcard exceptions, two unwaived error-order observations; exit 1 |
| Bounds / determinism | No comparison timeout; stable two repetitions |


The baseline retains the recovered drop-chain/nested-layout controls (`16`, `17`)
and original word/array/closure cases (`63`, `777113`, `64180`), each with final
LF, empty stderr and exit 0 across all five runtimes twice. The baseline does
not run the direct upstream CLI lane. Primary upstream C uses explicit O3,
`-lpthread -lm`, and `--threads 1 --gpu off`; port C keeps strict warnings.

The negative runner remains exit 1: eight resource-policy observations, two
unsupported native-foreign observations and two Bun-host failures are unqualified.
The demand runner also remains exit 1: six already-approved narrow C-wildcard
exceptions are separate from two unwaived Nat-first diagnostic-order observations.
Neither phase-only agreement nor an unavailable lane becomes an exact pass.
No additional exception or evaluation-order choice was made.

The first source checkpoint's normalization and runtime matrices passed their
applicable comparisons, but its full gate failed on one stale template expectation
(`Cons{5n, Nil{}}` versus `Cons{5n, []}`). Other candidate failures were old tuple/list
text expectations, the new test reaching the unchanged input guard, and the
Clippy function-size rule. Those receipts remain at their actual candidate
identities. The final clean-source gate and matrices were rerun after the test
correction. Preflight still identifies the preserved `f6c057c` binary.

The 49 ignored tests remain 47 CUDA checks and two optional profilers. GPU,
fresh Windows/macOS, broader comptime/read/checking audits and the complete
runtime corpus were not rerun. Current ledgers renew 28 interpretation originals,
nine runtime originals and ten authored demand controls. Other rows retain
their historical source/profile identities; they do not qualify the new printer.
Dependencies, lockfile, toolchains, upstream pin and safety budgets remain frozen.

After activating the existing environment, set `CARGO_NET_OFFLINE=true` and
`BEND_GPU=off`, then use fresh evidence directories:

```sh
cargo test --features extended_observability --test kernel_normalization --test kernel_base --test compiler_c --test runtime_review --test executable_check --test parser_templates --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-normalization> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-baseline> -Suite baseline -OracleOptimization O3 -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <fresh-negative> -Suite negatives -OracleOptimization O3 -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <fresh-demand> -Suite demand -OracleOptimization O3 -Repetitions 2
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

## Disk cause and prevention

The initial inspection found 10.2 GiB of root test executables within a roughly
13 GiB dependency-artifact directory. Multiple test variants remained; two parser
test binaries even had identical root feature/profile labels but different
dependency fingerprints. Selecting the same root feature alone is insufficient
to guarantee reuse. In addition, [build.rs](../build.rs) watches Git HEAD, index,
branch reference and source, and embeds revision/worktree status/build time.
Commits and staging can therefore invalidate the library and relink the large
statically linked test executables. Default-feature experiments had previously
added another set of these artifacts.

This checkpoint uses bounded cache management before broad rebuilds. Its saved
helper inventories and hashes only known root test executables, refuses cleanup
while a target product is running, protects the CLI and example binaries, and
never deletes source, dependencies, logs or comparison snapshots. When less than
12 GiB is free, it unlinks the inactive rebuildable test set and requires the
reserve to be met before permitting a build. Every deletion has a retained
hash manifest. The two source rebuilds used this preflight; no disk exhaustion
occurred during this checkpoint.

The standard gate is sampled every two seconds. The task-owned process group
would be stopped below 2 GiB workspace headroom, below 1 GiB temporary-filesystem
headroom, or after 1,200 seconds. This is a validation-process safeguard; Bend's
language/runtime budgets remain unchanged. Identical immutable CLI snapshots
are also deduplicated by verified hash, preserving every path and byte. Mutable
build outputs are never hard-linked into evidence.

| Observation | Workspace headroom |
| --- | --- |
| Initial inspection | 5.17 GiB |
| After first bounded cache cleanup | 15.26 GiB |
| Before final source rebuild, after cleanup | 14.98 GiB |
| At final standard-gate start | 13.43 GiB |
| Minimum during final gate | **4.89 GiB** |
| After validation and snapshot deduplication | 5.28 GiB |

The final gate took 178.41 seconds; its minimum temporary-filesystem
headroom was 4.85 GiB. Neither monitored gate reached a disk/timeout stop.
The first cleanup recorded 104 rebuildable test products (10.09 GiB); it left
all evidence intact. Two sets of four identical CLI snapshots now share their
respective immutable binaries, saving about 0.78 GiB while retaining all paths
and hashes. Candidate and final gate/cache manifests are separate records.


Future source/metadata changes must repeat the preflight instead of assuming
an incremental relink will fit. If 12 GiB cannot be recovered from the bounded
rebuildable set, the helper refuses the build; it does not delete evidence to
make room. Use the existing `extended_observability` test profile and existing
all-feature build profile, without introducing another toolchain/profile.

## Receipt identities and remaining acceptance

| Receipt / input | SHA256 |
| --- | --- |
| preflight | `97e210aeb395632a3edb1186f38036a2c707cc96256f4b797952485ed74e3bb0` |
| normalization | `a64de0deda132cba0f7ff7c4fb76bdb242fa0096f76159f7259c5aeb523928c2` |
| final-normalization | `8ab1565d301999cde14cd23b91dd4506bdd3a82dbf0b2533389f04054b2218d4` |
| final-positive | `0f4054022097604d8d956b884849c0af6adfa795511296930cd2e592df3a028c` |
| final-negative | `77db0c04d64856f4dbe5868862b37f493672fffe35d132025429035f5c24c0fb` |
| final-demand | `2eb5b1e2f08169e08bef8b82da56117eb6818814f2a6c62f5f6063e6f28d8a53` |
| final-open-nat | `15b5489e1a4274b9847d1a2dc2e4c2834384278c3ea6fbe269d6552a4494cd3a` |
| final-printer-oracle-receipt | `d19eae69da1db96f230dc4a33769a07b1e95bb471fa15302a774cfce0b1d1d9c` |
| cache-before | `88031be20eb219515218404d39475af24b6c54f211de3417677059b6472fb4ed` |
| cache-before-final-build | `7ad114ac7acbd327b016856581c9e961541eaf195d7bac616dc119979b78b8a7` |
| cache-before-final-gate | `c2797c5a8602b81b68b4de03c97c58eb5ff06d19e120d3a4aacc2dad1a6100a5` |
| final-gate-monitor | `993fc5ad5124a7f34e28d16ff0157f53d369b296289f9d079e7ea5d50105e41b` |
| snapshot-dedup | `86351c56b3e151cab8930106b5edb24fb7774c42c1477e5b2d235d8226b83bd3` |
| final-snapshot-dedup | `69e041968303980b6e037bf6e3b34fc8f4e273adac201fe8204c158f48fbe5dc` |
| Final CLI binary | `1e3e6864f3dc9bfbfe057e3c61e91dfd998803dd28ecb1d117ee445034b50930` |
| Final structural probe | `823cc6d6a5f1f4a9afd97b0951cf2ec77791f78f37eb2105d1ebde3a5a617149` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |


The unchanged [acceptance criterion](cpu-parity-acceptance.md) is not satisfied
by this checkpoint. Runtime/interpretation ledgers now link 103 originals;
1,199 remain unlinked by those modes: 439 checker negatives, twelve check-only,
153 interpreter-only and 595 compiled candidates. These categories are distinct
requirements, not 1,199 missing runtime tests.

The next ledger-selected gap is
[`eval/list_reverse_sort.bend`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval/list_reverse_sort.bend):
checking-only qualification of List.reverse and insertion-sort equality proofs.
It has no main; compiled/runtime lanes must not be manufactured for it. Its
historical acceptance is not a current normalized/diagnostic comparison. Source
was inspected here; execution is still unrun. The independent
`parse/chain_default_nat_f` / `_g` function family also remains unqualified.
