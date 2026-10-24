# U32 suffix function normalization

Coverage source `105484a91c4d5fd134a93edfd7a760543f5160ff` qualifies three
unchanged pinned originals. No language implementation defect was demonstrated;
only fixture coverage, the normalization suite selection and its regression
inputs changed. Production source, the structural probe, dependencies, toolchains,
upstream pin, demand policy and safety budgets are unchanged. No push occurred.

## Exact results and applicable modes

| Original | Exact normalized value before final LF |
| --- | --- |
| [chain_suffix_u32_f](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_suffix_u32_f.bend) | `a => b => c => U32.add(a, U32.mul(b, c))` |
| [chain_suffix_u32_g](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_suffix_u32_g.bend) | `a => b => c => U32.add(U32.add(a, b), c)` |
| [chain_suffix_u32_h](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_suffix_u32_h.bend) | `a => b => c => U32.mul(U32.sub(U32.or(U32.shl(U32.shl(a)), b), c), a)` |

The pinned interpreter and native `run` produce these bytes with empty stderr
and exit 0 twice: **twelve exact raw value outputs**. Strict JSON `eval` produces
six successful reports with the same `main` value. Six structural comparisons
observe the **actual checked normalized core**, with `core_term_observed=true`;
raw text is checked separately. These U32 definitions are supported in strict
proof mode. No executable-only comparison flag or strict-proof exclusion is
used for this family.

The fixtures cover operator precedence, a suffix inside a call argument and a
reusable parameter used through a let-bound shift/bitwise expression. `_h`
normalizes the two-bit shift to two nested `U32.shl` calls and retains both uses
of `a`. These are returned functions; this checkpoint does not apply numeric
arguments or add evidence about overflow, foreign calls or float parsing.

Both actual upstream data printers refuse each function type, twice per target:
**twelve refusals**, exit 1, empty stdout and no emitted artifact. The diagnostic
includes the declared function type, including reusable `+a` for `_h`. The
pinned [printer rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
and [interpreter main dispatch](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L454)
support this distinction. Compiled lanes are inapplicable by observed upstream
behavior, not an accepted port exception. No C compiler or O3 execution is
claimed for these returned-function fixtures.

Original source bytes, comments and expectations are retained with Apache-2.0
attribution in the [fixture README](../tests/fixtures/cpu-normalization/README.md).
The existing Base-function regression now includes these three originals and
compares actual core structure plus exact text. No existing assertion is removed.

## Validation and provenance

| Check | Result |
| --- | --- |
| New family | Twelve exact interpreter/native value outputs, six matching strict JSON eval reports, six actual-core comparisons and twelve upstream printer refusals; two repetitions |
| Full normalization matrix | 28 fixtures: 24 originals and four authored controls; 218 successful observations, 52 expected printer refusals and six retained F32 proof exclusions; exit 0 |
| Focused normalization/executable/numeric tests | **30 passed, 0 failed, 0 ignored** |
| Standard repository gate | **819 passed, 0 failed, 49 ignored**; exit 0 |
| Strict all-target Clippy and build | Exit 0 |
| Determinism and bounds | Stable repetitions, zero timeouts |

The full gate count stays at 819 because three new fixture inputs extend an
existing regression test. Counts alone are not the coverage claim. The full
normalization matrix preserves the separate F32 executable-output mode and its
in-crate actual-core regressions; its six strict-proof exclusions still concern
F32, not this U32 family. Structural comparison rules are unchanged.

The interpretation ledger now links **24 originals**, renewed at `105484a`:
eleven interpreter-only cases and thirteen compiled candidates with actual
upstream function-printer refusals. With 75 runtime originals, **99** have
runtime/interpretation receipts; **1,203** remain unlinked by those modes:
439 checker negatives, twelve check-only modules, 153 interpreter-only cases and
599 compiled candidates. Historical applicability classifications remain intact.
These are distinct requirements, not a CPU-runtime defect count or percentage.

Runtime, negative, demand, F32.read, checker-audit and comptime receipts retain
`2f349c8`; they were not rerun or relabelled. Production behavior did not change.
All existing cross-backend read, first-error-order, resource, foreign, oracle-host,
API and platform gaps remain open. No broader demand-policy interpretation is
introduced. Windows/macOS, GPU and additional worker profiles remain unrun.

## Reproduction

Source the existing activation script, with `CARGO_NET_OFFLINE=true` and
`BEND_GPU=off`. `$reference` denotes the clean pinned checkout; `$evidence` is a
fresh private evidence directory. Exact expanded command arrays, raw streams,
source/binary hashes and bounds are retained in private `cpu-u32-suffix-functions`
receipts. Comparisons use 60-second child deadlines; the full gate has a
1,200-second outer deadline. Prior evidence is preserved.

```powershell
cargo test --locked --features extended_observability --test kernel_normalization --test executable_check --test parser_numeric
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo build --locked --features extended_observability --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference $reference -OutputDirectory "$evidence/normalization" -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

For a bounded family-only comparison, select `-Suite u32-suffix` and a new output
directory. The committed-source full matrix includes this family; the earlier
family-only candidate receipt remains explicitly identified as a dirty precommit
probe. The initial unchanged-binary preflight identifies `09df250`, not the new
coverage source.

| Receipt / input | SHA256 |
| --- | --- |
| preflight | `12b8890b168da50e06650a3072e5084b1cfc24f84464564a35c7407dd527f67c` |
| candidate | `ad17ae0a77fc7616b4d298275dd4ca6babcaf4198e1a5b534d9ba331d807be95` |
| normalization | `29e6f630f168315a9aa506b8e81e3bf27e09bfff4cbe362d47a61a6c1d4327ec` |
| CLI binary | `68fbd63cd8c5c7056cf30bb5dc4268d377e9b2dfe78924b280791bc1f85cbb80` |
| Structural probe | `213d0d5861d4a8f7376ace635d8b41bc138b66b4a56684936291f49c3ea23e78` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |

The [acceptance criterion](cpu-parity-acceptance.md) remains unmet. Next independent
family: `parse/chain_brace_suffix_f`, `_g` and `_h`, covering brace annotation,
Nat operator defaults and an equality-type result. Source inspection and prior
strict acceptance are not normalized-result or printer-applicability evidence;
those comparisons are still unrun.
