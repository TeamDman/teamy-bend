# Base function companion qualification

Historical checkpoint follows. The subsequent [Nat-fuel qualification](cpu-nat-fuel-functions.md)
completes the next function group proposed here; these receipts remain at `4117071`.

Source checkpoint: `4117071081164cb250f8f919159ca45929327f02`, on local branch
`qualify/cpu-base-function-companions-20261023`, based on `c5573dc`.
Target remains [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
This is a coverage-only change: original fixtures, focused regressions and the
comparison runner. No production code, structural comparator, demand semantics,
resource budget, public API, dependency, lockfile or toolchain changed. No push.

## Establishing the applicable lanes

All three originals import Base, so their C/JS lanes are syntactic candidates.
However, pinned [test_probes](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L75)
omits compiled lanes when upstream reports an unprintable main type. The
[show_main descriptor builder](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1204)
rejects functions rather than printing their normalized terms.

Before any value comparison, actual upstream generation was attempted for both
C and JS. All six preflight commands exited **1**, produced empty stdout and
created no target file. Final qualification repeats each refusal twice before
the corresponding value comparison. Stderr is exactly this template, including
one final LF, with the type from the table substituted:

```text
Error: main's type <type> cannot be printed (a function, a Type, an erased or dependent field)\n
```

| Original under `flatten/` | Exact type in both C/JS diagnostics |
| --- | --- |
| `computed_recursion_001_main_dec.bend` | `@r:W -> @z1:Bl -> @z2:Bl -> W` |
| `computed_recursion_001_not.bend` | `@a:Bl -> Bl` |
| `computed_recursion_001_par.bend` | `@a:W -> Bl` |

These are verified oracle printer exclusions, not port failures, safety refusals,
Bun-host failures or permissions blockers. No C source reaches compilation, so
O3 and O0 execution profiles are **not applicable** to these originals. Future
applicable C-oracle runs should use pinned cli_build's O3 profile; the historical
Word O0 failure remains separate evidence in the [ordering report](cpu-failure-order.md).
No global runner flags or profile labels were changed here.

## Exact interpretation results

All three pinned-interpreter and port results agree in core structure and exact
text, twice. Both `run` programs exit **0**, stderr is empty, and stdout is the
following term plus one LF. Port `eval` retains its Facet envelope and the same
decoded value; the structural probe preserves binder identity, scope, arms,
defaults, applications and global references.

| Original | Exact normalized term |
| --- | --- |
| `_001_main_dec` | `r => \{T: \{T: S{r}; F: Z{}}; F: (z2 => r)}` |
| `_001_not` | `\{T: F{}; F: T{}}` |
| `_001_par` | `\{Z: T{}; S: (n => not(main(n)))}` |

No semantic or presentation defect was demonstrated. The copied upstream files
remain byte-identical, with original expectation records and Apache attribution.
New regressions load Base through the real loader, compare original goldens and
core shapes, and check ten complete applications: all four Boolean decision
pairs with a captured successor value, both flips, and recursion at depths zero
through three. These port application controls support regression coverage;
they are not substituted for the original function-main differential evidence.

## Validation and provenance

- Standard gate: **803 passed, zero failed, 49 ignored**, exit 0. Formatting,
  strict Clippy, all-feature build and the real Node provider also pass.
- Strict all-target/all-feature Clippy: exit 0. Focused normalization tests:
  **14 passed**, including the two added regression tests.
- Expanded normalization matrix: **19 cases**, fifteen original and four
  authored, two repetitions each. **168 commands**: 152 successful observations
  and sixteen expected upstream printer refusals across four originals.
  Exact text and structural comparison both pass; all repetitions are byte-stable,
  no command times out and refused generation leaves no target artifact.

The gate ran on the content committed as `4117071`. CLI and probe builds then
ran from that clean commit, before final receipts. No runtime positive/negative,
demand-ordering, comptime, multiworker or platform matrix is relabelled as a new
execution: this change does not modify their production implementation. Their
receipts retain their prior commits and explicit unresolved cells. The existing
`KernelError` private-field / `run_main` `Ok(1)` downstream API caveat remains.

Commands with the existing activated tools and exported
`CARGO_NET_OFFLINE=true`, `BEND_GPU=off`:

```sh
# Preflight, for each original and each c/js target, from the reference bend2 directory:
node main.ts <copied-original.bend> -o <fresh-target.c-or-js>
cargo test --features extended_observability --test kernel_normalization
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <reference> -OutputDirectory <fresh-results-dir> -Suite all -Repetitions 2 -RequireExactText
```

The runner also offers `-Suite base-functions` for this three-original family.
Private `cpu-base-function-companions` evidence retains preflight and final raw
streams/statuses, exact argument arrays, source/binary hashes, generated-artifact
checks, deadline/repetition results, test/build logs and binary snapshots.
Older evidence is preserved. Disk headroom was checked before building.

Final receipt SHA-256:
`3c254cd702385699815dc16e09eb638667041b63e55cf4c17149cf8162e954ca`;
preflight receipt:
`610979d3e86ec55e1e5fd69ff6f41976cec2873798549da7e6dc972156a8cd3e`.
Candidate binary:
`9e2f6287fdd17ff5ceaa8e056d30903ae552c6ac95cf9c70fb5cb00e26d56489`;
structural probe:
`b1d964c7a15c4b04d47a387796e8e8af058b9dc84afa473791ab5ece9d2c88c6`.
The longest final-receipt child took 496 ms under the 60-second deadline.

## Acceptance and next independent gap

The [interpretation ledger](cpu-interpretation-evidence.csv) now links fifteen
originals, including four Base candidates with actual upstream function-printer
refusals. With the unchanged 51-row runtime ledger, **66 originals** have some
execution/interpretation evidence. **1,236** lack those receipt kinds: 439 checker
negatives, twelve check-only modules, 153 interpreter-only cases and 632 compiled
candidates. These are distinct acceptance requirements, not a runtime defect
count. The separate checking ledger remains unchanged. The full
[acceptance criterion](cpu-parity-acceptance.md) is not met.

Competing-failure diagnostic selection remains unresolved and unwaived; no
additional semantics or exception follows from the ordering investigation.
Other resource, native-foreign, Bun-host, corpus/API and platform requirements
remain open. GPU and fresh cross-platform checks were not run, and no dependency
installation is needed to continue independent CPU work.

Next bounded group: `computed_recursion_000`, `_000_big`, `_001`. Their sources
combine Nat fuel, deeper nested patterns and a reusable `+` binding around
recursive/computed decisions. They are still unqualified function-main
candidates: first verify actual printer applicability, then compare normalized
structure/text and targeted complete applications. No outcome is claimed here.
