# Nat-fuel functions and application printing

Implementation: `20170de53a0e735812866f4be4bda615ecb78e71`, local branch
`qualify/cpu-nat-fuel-functions-20261023`, based on `0ad312c`.
Target remains [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
This checkpoint fixes a presentation defect, qualifies three original functions,
and records fresh applicable C-oracle runs with explicit O3 optimization.
Dependencies, lockfile, tools, safety limits and approved demand semantics are
unchanged. No push or upstream/docs-reference source modification.

## Applicability and demonstrated defect

Actual upstream C and JS generation was attempted before value comparison.
All three originals are function-valued mains: both targets exit **1**, emit no
stdout and create no target file. The exact diagnostic is
`Error: main's type <type> cannot be printed (a function, a Type, an erased or dependent field)\n`.
The types are `@f:Nat -> @+n:Nt -> W` for `_000`, `@a:Nt -> Bl` for
`_000_big`, and `@f:Nat -> @n:W -> W` for `_001`.
Pinned [test_probes](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L75)
therefore selects checking/interpretation and excludes compiled execution after
the verified [show_main refusal](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204).
No C optimization profile applies to these three ungenerated programs.

Baseline preflight against `4117071` found full structural agreement for all
three. `_000_big` was already byte-exact. The other two printed nested call
parentheses: `main(g)(p)` instead of `main(g, p)`, and
`main.dec(main(g)(m0))(par(m0))(par(m0))` instead of
`main.dec(main(g, m0), par(m0), par(m0))`. All interpretations exited 0 with empty
stderr; these were presentation differences, not evidence of reduction failure.

Pinned [term_show's App branch](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1353)
uses [term_unapply](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L668),
then prints one argument list in order. The port's `Term::Display` now follows
that rule, traversing only the function side of consecutive applications.
Nested argument calls stay nested; annotations stop traversal. Grouping around
lambda/let/rewrite/arrow heads and let arguments preserves their precedence.
No term is evaluated, substituted, reordered or erased by the printer.

This changes textual terms in pure CLI output and diagnostics, including type
applications such as `Pair(A, B)`. The parser still accepts the prior chained
call syntax. Existing exact type expectations were updated to the new spelling
while retaining quantities, field types and source arity checks. No public API
signature or Facet envelope changed; the prior `KernelError` private-field and
`run_main` `Ok(1)` downstream-consumer caveat remains open.

## Exact normalized results

Pinned interpreter, port `run`, decoded `eval` value and structural probe agree
twice for all three. `run` stdout is the following term plus one LF; status is
0 and stderr is empty. The original sources and expectation bytes are unchanged.

| Original under `flatten/` | Exact normalized term |
| --- | --- |
| `computed_recursion_000.bend` | `\{Zero: (n => Z{}); Succ: (g => \{Zn: Z{}; Sn: \{Zn: Z{}; Sn: (p => S{S{main(g, p)}})}})}` |
| `computed_recursion_000_big.bend` | `\{Zn: F{}; Sn: \{Zn: F{}; Sn: (p => T{})}}` |
| `computed_recursion_001.bend` | `\{Zero: (n => Z{}); Succ: (g => \{Z: Z{}; S: (m0 => main.dec(main(g, m0), par(m0), par(m0)))})}` |

Fifteen additional complete port-application controls cover zero/exhausted fuel,
zero/one/two-or-more nested constructors, increased fuel, and both parity choices
around the reusable binding. They exercise the original definitions but are not
claimed as a separate upstream applied-function differential matrix. Adjacent
printer controls retain nested argument order, binding, rewrite grouping,
annotation boundaries and let scope. The structural comparator is unchanged.

## Current validation and explicit C profiles

- Standard gate: **805 passed, zero failed, 49 ignored**, exit 0. Formatting,
  strict Clippy, all-feature build and real Node-provider checks pass.
- Strict all-target/all-feature Clippy passes. Focused parser/template/normalizer
  tests pass **54** cases; the four affine host-contract checks also pass.
- Normalization renewal: **22 cases**, eighteen original and four authored,
  **204 commands**: 176 successful observations and 28 expected function-printer
  refusals across seven originals. Exact text, structure and repeated bytes pass.
- Positive runtime renewal: **308/320 exact**, twelve known upstream-JS Bun-host
  failures; 32 fixtures twice, runner exit 0 with blocked cells retained.
- Negative renewal: **44/56 exact**, eight resource-policy differences, two
  native-foreign gaps and two Bun-host failures; required-parity runner exit **1**.
- Demand renewal with primary O3: **92/100 exact**, six explicitly accepted C
  wildcard exceptions and two unresolved Nat-first diagnostic differences;
  required-parity runner exit **1**. The Word control now executes and matches
  twice in O3; historical O0 compilation failures remain separate evidence.

All repetitions were byte-stable and no child timed out (longest final child
under 0.84 seconds). No new runtime mismatch was observed. The sixteen comptime
originals retain their `208b343` receipts and no-optimization-flag C profile.
The demand ledger now identifies historical O2 source/receipt provenance
separately from current O3 results; a prior profile pass is not relabelled.

The initial focused run exposed an invalid let spelling in a new test, corrected
to the existing parenthesized syntax. The first full gate exposed two old
type-display expectations; those strings were updated without dropping any
assertions. Both failed logs are preserved alongside the final passing gate.

The positive/negative runners now accept `-OracleOptimization O0|O1|O2|O3`, default
to **O3**, and record `upstream_c_optimization` in each receipt. This follows the
optimization used by pinned [cli_build](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253).
The existing POSIX link flags remain `-pthread -lm -ldl`; exact argument arrays
are retained rather than claiming literal identity with the CLI link command.
Port C retains `-Wall -Wextra -Werror -pedantic`; no warning policy was weakened.
Historical no-optimization-flag/O0 receipts are neither overwritten nor relabelled.

The source was committed before clean CLI/probe builds and final comparisons.
Older comptime, multiworker, direct-CLI, checking and platform evidence remains
at its recorded source/profile. New receipts and counts appear below; the
separate failure-order investigation's O0/O1/O2/O3 receipts remain historical.

Commands with the activated frozen tools and exported `CARGO_NET_OFFLINE=true`,
`BEND_GPU=off`:

```sh
# Preflight from the pinned reference's bend2 directory, each original and target:
node main.ts <copied-original.bend> -o <fresh-target.c-or-js>
cargo test --features extended_observability --test kernel_normalization --test parser --test parser_templates
cargo test --features extended_observability --lib contracts_keep
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <reference> -OutputDirectory <fresh-normalization-dir> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2 -OracleOptimization O3
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-demand-dir> -Suite demand -Repetitions 2 -OracleOptimization O3
```

The normalization runner also supports `-Suite nat-fuel`. Private
`cpu-nat-fuel-functions` evidence retains preflight, exact stream/status/timing
receipts, source/generated artifacts, binary snapshots and initial/final logs.
Disk headroom was checked before builds. No prior evidence was removed.

Receipt SHA-256 values:

| Receipt | SHA-256 |
| --- | --- |
| Normalization | `ed5ca909e9c6a8fc0234ccdd64ad36a2426f2361a2554b432356f5012ee72b9b` |
| Positive O3 | `7110bfe6880707d13f1126678a2ec5a0315765dffb1e4e5378616ab86eab7100` |
| Negative O3 | `a498fa7657a7457667407308c6b5377282ec0af79ea1fa4ef0244e0ac4b59376` |
| Demand O3 | `c7acc18c7327b791b44f6b34354290a9927c84877244fa10ac07f63edf9c0cff` |

Candidate binary SHA-256:
`d621cf3d1c16f0ff2c51172642ff76be9750bf7cfbb4473d35a18ecaf7186a11`;
structural probe:
`5b6be152596f9609e15e6eab05c31161a4ec97a6c051d0f86c126d181523eeb9`.

## Remaining acceptance gap

The [acceptance criterion](cpu-parity-acceptance.md) remains unmet. Runtime and
interpretation ledgers now link **69 originals**: 51 runtime, eighteen
interpretation. **1,233** lack those receipt kinds: 439 checker negatives,
twelve check-only modules, 153 interpreter-only cases and 629 compiled candidates.
The separate checking ledger and its unresolved diagnostic cells are unchanged.
Authored controls add no original-fixture coverage. Missing checking evidence
is not described as a missing CPU executable.

The next independent group, selected from the
[applicability ledger](cpu-applicability.csv), is `parse/adjacent_application`,
`parse/chain_paren_group`, and `parse/chain_spacing_rules`. All have main functions
and Base imports and are classified as interpretation plus compiled candidates;
none has runtime/interpretation/checking receipts in the current ledgers. The
historical strict audit accepted the first and refused the latter two; those
status-only observations do not establish their behavior in the intended pure/IO
modes. Their source goldens are `True{}`, `7`, and `2`, respectively. Verify
actual applicability, then qualify the original output contracts with O3 where
C compilation applies; do not infer a runtime defect from the strict audit alone.

The competing-error discrepancy stays unresolved and unwaived. Resource-policy,
native-foreign, Bun-host, broader corpus/API and platform requirements remain.
No new demand policy or exception was implemented, and no GPU or fresh platform
qualification is claimed.
