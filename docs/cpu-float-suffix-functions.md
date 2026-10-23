# Float-suffix function normalization

Coverage source `09df250e835607b48d803c100fe4549651b0edcd` adds three unchanged
originals from the pinned Bend target. No parser, checker, runtime demand,
numeric or printer defect was demonstrated. This checkpoint extends comparison
tooling and regressions; it does not change production language behavior.

## Exact normalized results

Each raw result below has one final LF, empty stderr and exit 0. Both the pinned
interpreter and native `run` produce it twice.

| Pinned original | Exact result before LF |
| --- | --- |
| [chain_suffix_f32_f](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_suffix_f32_f.bend) | `a => b => c => F32.add(a, F32.mul(b, c))` |
| [chain_suffix_f32_g](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_suffix_f32_g.bend) | `a => b => F32.is_lt(a, b)` |
| [chain_suffix_f32_h](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/chain_suffix_f32_h.bend) | `a => b => F32.is_eq(a, b)` |

These are function values, not calls with float arguments. The evidence covers
operator precedence, the comparison's operand-type suffix, equality's explicit
primitive, binding structure and rendering. It does not newly qualify floating
rounding, NaN payloads or runtime F32.read semantics. Fixture bytes, original
comments and expectation records are unchanged; Apache-2.0 attribution is
retained in the [fixture README](../tests/fixtures/cpu-normalization/README.md).

## Applicable modes and structural evidence

Actual pinned C and JS generation refuses all three function-valued results,
twice per target: **12 refusals**, exit 1, empty stdout and no output artifact.
The exact diagnostic states that main's function type cannot be printed.
This agrees with pinned [show_main](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204).
These compiled lanes are inapplicable by observed upstream behavior. No C
compiler or optimization profile is exercised by this family. The pinned
[book_run](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L454)
normalizes a non-IO main as a value, retaining its parameters.

The previous comparison example always used the strict proof loader/checker.
For these sources that path refuses `F32.add`, which belongs to sealed executable
numeric contracts. The initial probes retained this refusal even though native
`run` already matched upstream. It is a comparison-mode gap, not evidence of a
language normalization defect. The new runner retains **six strict `eval`
refusals**, labelled `strict_proof_exclusion`; they are not runtime parity passes.

The example's explicit `--executable-function` mode checks the executable source,
requires a pure function result and captures the existing `run_main` output.
It compares parsed output structure and preserves raw text separately. Its
`core_term_observed=false` field makes that observation boundary explicit; older
strict cases still compare actual normalized core with `core_term_observed=true`.
No public proof-normalization API is added to `ExecutableBook`.

A separate [in-crate regression](../src/kernel/executable_normalization_tests.rs)
checks the **actual normalized core** of each original against its unchanged
pinned expectation, using the same structural vocabulary. The differential
runner independently confirms that expectation against the pinned interpreter.
The regression also checks exact text/native writer output and preserves strict
proof rejection. This provides core evidence without pretending serialized
output alone exposes the engine's internal term. Every binder/reference,
application argument and supported rewrite field remains structurally checked.

The comparison controls distinguish the contracts: changing `F32.add` to
`F32.mul` gives structural mismatch and exit 1; consistent alpha-renaming keeps
structure equal but raw text unequal; the exact oracle passes both. No tolerance,
normalization of raw output or weakening of structural comparison is introduced.

## Validation

| Check | Result |
| --- | --- |
| New family | Twelve exact interpreter/native value outputs, six structural-output comparisons, twelve expected upstream C/JS printer refusals and six expected strict-proof refusals; two repetitions |
| Full normalization matrix | 25 fixtures: 21 originals and four authored controls; 194 successful observations, 40 expected printer refusals and six strict-proof exclusions; exit 0 |
| Core regression and focused tests | **33 passed, 0 failed, 0 ignored** |
| Standard repository gate | **819 passed, 0 failed, 49 ignored**; exit 0 |
| Strict all-target Clippy and build | Exit 0 |
| Comparison controls | Wrong operator rejected; alpha-renaming distinguished from exact raw text; exact oracle accepted |
| Determinism / bounds | Repeated results stable; zero timeouts; longest comparison child 0.624 seconds |

Existing runtime, negative, demand, F32.read, checker audit and comptime receipts
retain `2f349c8` and their previous open cells. They are not rerun or relabelled:
production behavior is unchanged, and the only production-module diff registers
a `cfg(test)` module. Historical platform and worker receipts retain their own
identities. The new standard gate and normalization suite qualify `09df250`.

The [interpretation ledger](cpu-interpretation-evidence.csv) now links **21
originals**: eleven interpreter-only cases and ten compiled candidates whose
function result is actually refused by upstream printers. Together with 75
runtime originals, **96** have these receipts; **1,206** remain unlinked by these
modes: 439 checker negatives, twelve check-only modules, 153 interpreter-only
cases and 602 compiled candidates. Historical applicability snapshots are not
rewritten, and these counts are not a parity percentage.

## Reproduction and retained evidence

After sourcing the existing activation script, use `CARGO_NET_OFFLINE=true` and
`BEND_GPU=off`. `$reference` is the clean pinned checkout and `$evidence` a fresh
private evidence directory. Exact expanded commands, raw streams, source/binary
hashes and per-child deadlines are retained in private `cpu-float-suffix-functions`
receipts. The runner uses 60-second child bounds; the standard gate has a
1,200-second outer bound. Probe controls use 30-second bounds.

```powershell
cargo test --locked --features extended_observability --lib float_suffix_functions
cargo test --locked --features extended_observability --test kernel_normalization --test executable_check --test parser_numeric --test cpu_parse_floats
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo build --locked --features extended_observability --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference $reference -OutputDirectory "$evidence/normalization" -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
python3 "$evidence/probe-controls-final.py"
```

| Receipt / input | SHA256 |
| --- | --- |
| preflight | `dbb34d812c1c07a4efc354e89b1333767e5dd2e4bf6eba65cebc0b1bdead703b` |
| candidate | `167ca2b63ee8f65cbd814b089cd0082e24c8fdcb134881406863ff476802d780` |
| normalization | `6602b0578bedf426a50012488651f5b53db9dfc9a8fa33042b0539773370eec8` |
| probe-controls | `03c6065dfb4eddd8a838fd5e577f9410ec37f2d8cc2d262ad11c6bed937f777e` |
| probe-controls-final | `46f96c95a08d284ef413792ab7a6f6b8cc7b32e0aba1b69c3cf1619bc6ec2d61` |
| CLI binary | `9dbd9a6b0dec3efeb0a1603ed4e1861a1477f159b07cf834a9fd1f6da3c9d2b8` |
| Structural probe | `fafba472fc4c47b8e5c79894bee792bdae000676da305c9cd33bd4c7a40c590d` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |

The [CPU acceptance criterion](cpu-parity-acceptance.md) remains unmet. F32.read's
three cross-backend contracts, competing-error order, resource/foreign/host
limits, wider lane/API coverage and the existing downstream API caveat remain
open. No demand-policy extension, new exception, dependency/toolchain/budget
change, installation or push occurred. Current Windows/macOS and GPU validation
remain unrun; the 49 ignores are 47 CUDA tests and two optional profilers.

Next independent family: `parse/chain_suffix_u32_f`, `_g` and `_h`. Their sources
exercise nested arithmetic/call suffixes and a reusable binding containing
shift/bitwise operations. Source inspection and prior strict acceptance are not
normalization or printer-applicability evidence; those comparisons remain unrun.
