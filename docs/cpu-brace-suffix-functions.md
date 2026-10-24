# Brace suffix functions and complete Nat presentation

Source checkpoint `f6c057c30a9708f9fdd173cde01b83cd0f764490` qualifies the
unchanged pinned `parse/chain_brace_suffix_f`, `_g` and `_h` originals. `_h`
exposed a presentation defect: actual core structure already agreed, but the
port printed `Succ{Zero{}}` where upstream prints `1n`. The term printer now
renders complete unqualified `Succ`/`Zero` chains as Nat literals. No evaluator,
checker, code generator, demand contract or safety budget changed.

## Exact results and applicable modes

| Original | Exact normalized value before final LF |
| --- | --- |
| [chain_brace_suffix_f](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_brace_suffix_f.bend) | `a => b => U32.add(a, b)` |
| [chain_brace_suffix_g](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_brace_suffix_g.bend) | `a => b => Nat.add(a, b)` |
| [chain_brace_suffix_h](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_brace_suffix_h.bend) | `a => b => {Nat.add(a, 1n) == b : Nat}` |

The pinned interpreter and native `run` match these bytes twice, with empty
stderr and exit 0. Six strict JSON `eval` reports carry matching values. Six
structural probes inspect the **actual checked normalized core**, independently
of exact raw-text checks. All three support strict proof checking; no executable
function fallback or proof exclusion is used. Existing F32 executable-output
comparison, its separate internal-core regression and its strict exclusions
remain distinct.

Actual upstream C and JS generation refuses all three returned function types,
twice each: twelve exits of 1, empty stdout and no emitted artifact. `_h`'s
codomain is Type, but its main remains a function. The pinned
[printer applicability rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
and [main dispatch](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L454)
support this classification. No compiled execution or C optimization profile is
claimed for these three function results.

The [pinned Nat printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1229)
recognizes constructor spelling without consulting a type registry. The fix
walks complete syntax iteratively; it never evaluates a field, strips an
annotation, guesses a namespace or changes constructors. Regression controls
cover zero, one, several successors, equality/application/constructor positions,
malformed arities, namespaced constructors, and retained open/annotated/hole
fields. Structural comparison's qualified vocabulary is unchanged. Existing
exact assertions now require compact Nat spelling; their values and safety
checks remain. This intentionally changes term-display text in pure results
and diagnostic contexts, with no Rust API signature change. The earlier
[downstream API caveat](cpu-nat-failure-contract.md#review-of-rust-api-consumers)
remains independent and unresolved.

Original bytes, comments and expectations retain Apache-2.0
[attribution](../tests/fixtures/cpu-normalization/README.md). No upstream or docs
source was modified. Dependencies, lockfile, toolchains and pin remain frozen;
no installation or push occurred.

## Validation and provenance

| Check | Result |
| --- | --- |
| New brace function family | Twelve exact raw values, six strict eval reports, six actual-core probes, twelve verified upstream printer refusals; two repetitions |
| Full normalization | 31 fixtures: 27 originals plus four authored controls; 242 successful observations, 64 printer refusals, six retained F32 proof exclusions; runner exit 0 |
| Affected focused tests | **114 passed, 0 failed, 0 ignored** |
| Standard repository gate | **820 passed, 0 failed, 49 ignored**; exit 0 |
| Strict all-target Clippy / build | Exit 0 / exit 0 |
| Adjacent executable originals | **30/30 exact** generated/native executions; six direct upstream IO host failures retained; runner exit 0 |
| Six runtime negatives | **44/56 exact** required observations; twelve unqualified; runner exit 1 |
| Ten demand controls | **92/100 exact**, six accepted C-wildcard exceptions, two unwaived error-order observations; runner exit 1 |
| Bounds / determinism | Zero comparison timeouts; two stable repetitions |


The negative matrix still has eight resource-policy refusals, two unsupported
native-foreign observations and two Bun-host failures. The demand matrix still
has six previously approved narrow C-wildcard exceptions and two unwaived
Nat-first diagnostic-order differences. Their runners correctly exit 1; neither
phase-only agreement nor a safety refusal becomes an exact pass. The three
adjacent executable originals are `chain_brace_suffix`, `chain_default_nat` and
`chain_suffix_u32`; all five generated/native lanes match, while their six direct
upstream IO entries retain the `import.meta.require` host failure. Primary C
runs use explicit pinned O3, `-lpthread -lm`, `--threads 1 --gpu off`.

An initial default-feature build ran out of disk before running tests. Only
inactive, rebuildable root test executables were removed, after recording their
hashes; source, CLI snapshots and every prior receipt were preserved. The next
candidate sweep recorded 792 passed, 28 failed and 49 ignored: old text
expectations plus a new test using terms outside the structural probe vocabulary.
The final gate below supersedes that candidate. The comparator vocabulary was
not expanded or weakened. Raw candidate/failure logs remain private evidence.

Preflight uses the preserved clean `105484a` binary and is not relabelled as
current. Final comparison snapshots identify clean source `f6c057c`; focused
checks precede that commit on the identical source tree. Current receipts renew
27 interpretation originals, six runtime negatives, three positive runtime
originals and ten authored demand controls. The other 66 runtime rows and
comptime/read/checking/audit/platform receipts retain their actual historical
identities. They do not qualify the new printer. No broad audit or complete
runtime corpus renewal is claimed here.

Reproduction after activating the existing environment, with
`CARGO_NET_OFFLINE=true` and `BEND_GPU=off`:

```sh
cargo test --features extended_observability --test compiler_c --test executable_check --test executable_unsafe --test gpu_marks --test kernel_base --test kernel_normalization --test parser --test parser_numeric --test parser_templates --test runtime_data --test runtime_review --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-normalization> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <clean-pin> -OutputDirectory <fresh-positive> -Suite parse-operators -OracleOptimization O3 -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <fresh-negative> -Suite negatives -OracleOptimization O3 -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <clean-pin> -OutputDirectory <fresh-demand> -Suite demand -OracleOptimization O3 -Repetitions 2
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Each comparison child has a 60-second deadline and exact command, output,
status and artifact records. Two repeats are deterministic; no comparison child
timed out. The 49 standard-gate ignores remain 47 CUDA checks and two optional
profilers; GPU and fresh desktop validation were not run.

| Receipt / input | SHA256 |
| --- | --- |
| preflight | `e96dd65417277d0e7391127d1b68aa8cb723fdc829e392dfde7164911d60823b` |
| normalization | `7e28b1665d2115ac7e73558d6fbb90f6fb3c72789627d785e1361e3856d24a69` |
| negative | `e5e8fea168b9111932d1e124015a6eecfa363f84e455113aa2e1e6ccada92245` |
| demand | `c07abf3d2141906fa67f17909e2ad7ebada123cc533f062044a10e7ae0db1288` |
| positive | `1df63ae3828fd6bd30b1d298cac8dcd2f6f2bb1af8b13a822998f7fad6ecd064` |
| open-nat | `28b28b8be4df9393b9306c3abed37189dc69ea4b3e7967db5906a1f2904fa76c` |
| CLI binary | `85ee22c8214fabdfbb9f7d755bae806b4e423148821f0ee07f194425187c80a5` |
| Structural probe | `1aee0cc234a29d41fe237be5a2e0dfb05cf97ee56ceea7bac275231e003c50ba` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |


## Remaining evidence and next gap

Full CPU parity remains incomplete under the unchanged
[acceptance criterion](cpu-parity-acceptance.md). Runtime/interpretation ledgers
now link 102 originals; 1,200 remain unlinked by those modes: 439 checker
negatives, twelve check-only, 153 interpreter-only and 596 compiled candidates.
These are applicability categories, not 1,200 runtime defects. Host, foreign,
resource, F32.read, error-order, API and platform requirements stay open.

An adjacent authored checked function isolates a remaining **presentation-only**
gap, repeated twice with equal actual core and exit 0 / empty stderr:

```bend
import Base
def main(n: Nat) -> Nat:
  Succ{n}
```

Upstream prints `n => 1n+n\n`; native `run` prints `n => Succ{n}\n`, and strict
eval carries that same unsugared value. This is not an accepted exact match.
The complete-literal fix deliberately leaves open-tail presentation unchanged;
its constructor/annotation structure remains tested. The pinned
[`eval/nat_literal_roundtrip`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval/nat_literal_roundtrip.bend)
is a concrete next original to qualify; its compound tuple/list/open-Nat output
needs actual printer-applicability review before any lane is claimed. The small
independent `parse/chain_default_nat_f` / `_g` family also remains unqualified.
No additional family was silently promoted from prior checker acceptance.
