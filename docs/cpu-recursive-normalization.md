# Recursive function normalization and match rendering

Historical evidence follows. The subsequent [rewrite-printer checkpoint](cpu-rewrite-printing.md)
at `571d1ce` closes the rewrite rendering gap identified here; the receipts below
remain tied to `3354e60`.

Candidate `3354e6078255c00752a1ca9f452ef60ae0b9a9ef` qualifies the unchanged
`flatten/computed_recursion_002`, `_003` and `_004` fixtures against
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Their normalized structures already agreed; a bounded match-printer correction
now makes all three exact. It also closes the two presentation differences from
the [preceding checkpoint](cpu-function-normalization.md). All six original
operator/recursion fixtures now agree structurally and byte-for-byte, twice.
This is a six-fixture qualification, not full printer or CPU parity.

## Root cause and scope

Pinned [term_show's match case](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1387)
walks consecutive fallback matches as an ordered arm list, omits a final empty
match and renders each branch at precedence 1. Its
[lambda case](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1347)
parenthesizes lambdas in that context. The port instead printed each fallback
as another nested match, included its final empty fallback and omitted branch
parentheses. The two earlier differences have an unambiguous source-backed fix.

`Term::Display` now follows those match rules, preserving arm order, explicit
nonempty defaults and nested matches inside arms. Low-precedence lambda, let
and rewrite branches are grouped; this does not rewrite their contents. The
change affects displayed match terms, including CLI values and diagnostics that
contain them. No public type/signature, evaluation rule, safety budget, effect
ordering, proof boundary or pending demand policy changes. The existing
`KernelError`/`run_main` downstream compatibility caveat remains; no new external
consumer qualification is claimed.

The structural comparator is unchanged. It still rejects unsupported forms and
does not evaluate terms, reorder arms, erase defaults or use text substitutions.
Regressions require both original golden text and core equality, preserve nested
arms/defaults/empty branches and let binding scope, and check complete recursive
applications at zero, one and two successor arguments. Existing arity, unsafe
recursion, cancellation, proof separation and serve controls pass.

## Exact normalized results

All displayed terms end with one LF in upstream and port `run` stdout. All exit
0 with empty stderr. `eval` keeps its Facet `{entry, value}` envelope and its
decoded value equals the same term. Both repetitions are byte-stable.

| Fixture under `flatten/` | Upstream and corrected port term |
| --- | --- |
| `computed_operator_match_002.bend` | `x => sel(dbl(x))` |
| `computed_operator_match_002_dbl.bend` | `\{Z: Z{}; S: (p => S{S{main(p)}})}` |
| `computed_operator_match_002_sel.bend` | `\{Z: R1{}; (y => RK{y})}` |
| `computed_recursion_002.bend` | `\{Zn: S{Z{}}; Sn: (p => sel(main(p)))}` |
| `computed_recursion_003.bend` | `\{Zn: T{}; Sn: (p => flip(main(p)))}` |
| `computed_recursion_004.bend` | `\{Zn: S{Z{}}; Sn: (p => sel(main(p)))}` |

Before this change the three recursive fixtures had the same structures, but
printed nested fallbacks; for example `_003` printed
`\{Zn: T{}; \{Sn: p => flip(main(p)); \{}}}`. Preflight retains all six original
results against the prior binary, not just the corrected outputs. No normalization
algorithm defect was demonstrated in this recursive family.

These originals have no Base import, so the pinned gate selects checking and
interpretation, not compiled C/JS printers. No compiled lane is manufactured or
waived for missing port support. The original sources and Apache attribution are
preserved. The interpretation ledger now links six rows, separately from the
51-row runtime ledger: **57 with some execution evidence, 1,245 without either
receipt kind**. The latter are 439 checker negatives, twelve check-only modules,
158 interpreter-only cases and 636 compiled candidates. These are distinct
requirements, not a runtime bug count; the acceptance criterion is unchanged.

## Validation and evidence

Standard gate: **787 passed, zero failed, 49 ignored**. Strict all-target,
all-feature Clippy passes. Initial focused CLI/kernel/parser/serve checks pass
45 tests; the added let-scope control is included in the full gate. The final
normalization receipt has **48 commands**: twelve upstream interpreter calls,
24 port CLI calls and twelve structural probes. All pass exact text, status,
stderr, structural and repeatability checks, with 60-second child deadlines and
no timeouts (longest child 172 ms).

The renewed positive matrix has **308/320 exact matches**, with twelve known
upstream-JS Bun-host failures (runner exit 0 with those cells explicitly blocked).
The renewed six-fixture negative matrix has **44/56 exact matches**, eight
resource-policy differences, two native-foreign gaps and two Bun-host failures;
its required-parity exit remains **1**. No new mismatch or timeout occurred.
Both use C `--threads 1 --gpu off` and two repetitions. These fresh receipts do
not renew older comptime or multiworker evidence.

Exact commands with the existing activated tools, `CARGO_NET_OFFLINE=true` and
`BEND_GPU=off`:

```powershell
cargo test --locked --test kernel_normalization --test cli_proofs --test parser --test serve_protocol
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo build --locked --features extended_observability --example compare_normalization --bin teamy-bend
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <reference> -OutputDirectory <fresh-normalization-dir> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
```

The final clean-candidate binary SHA-256 is
`4f1937593bda173f0d32b8dbc60440dd7b0792e1daf7d49584479434aa94c3fd`;
probe `ab87655dfc201a759415bfe03a948f5dbc581c6ffbd07043b88e5b2bd3e88887`.
The normalization receipt SHA-256 is
`de54f39ebda64ed1ffba42add884ca02ad1fb9d0c7f0145fb15937673c104784`.
Positive receipt SHA-256: `92e5b36412a7a85a17f5dea071c93bb6cb19aaade4a93f170a8e81faf475be3d`.
Negative receipt SHA-256: `403e61b80826860fa4ffa77c1a5fc46b72bfb8ebe4e17b51e94b026dc7ca15f1`.

Private `cpu-recursive-normalization` evidence retains raw streams/status/timings,
commands, hashes, preflight snapshots and initial failures. A full filesystem
interrupted the first clean-commit receipt attempts before fixture execution;
removing only rebuildable Rust incremental cache recovered space. Source,
commits, validation binaries and prior evidence remain intact. Fresh retry
directories hold complete receipts. No install or toolchain change was needed.
An exploratory rewrite-printer control also failed; its separate gap below is
retained, not converted into a six-fixture pass or a comparator exception.

## Next independent gap: rewrite-term rendering

This small checked program needs no Base, host effects or pending demand choice:

```bend
type N is Data:
  Z{}
def main(e: {Z{} == Z{} : N}) -> N:
  %e : N
  Z{}
```

Actual pinned upstream prints `e => %e : N; Z{}`. Both the preserved preceding
binary and this candidate print `e => %e: _ =>  => N; Z{}`. Each exits 0 with
empty stderr; the separate bounded probe retains exact bytes. This authored
reproducer does not increase the upstream inventory count. It demonstrates a
pre-existing rendering gap, not a proven reduction mismatch.

The parser stores two internal motive binders. Upstream's
[rewrite printer](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1411)
opens them and prints the source-level optional evidence binder; the port's
rewrite printer currently exposes the internal lambda telescope. A next bounded
step is to correct this boundary with named/unnamed motive-binder, shadowing and
round-trip controls, then compare selected original rewrite/proof fixtures in
their proper checking/normalization lanes. The current structural comparison
vocabulary deliberately does not include rewrites; do not extend qualification
by treating them as opaque text or dropping their evidence/motive.

Earlier comptime and multiworker/platform receipts remain at their actual
commits. Broader coverage, resource-policy/foreign/oracle-host gaps, the API
inventory and pending demand choices remain open. GPU and fresh cross-platform
checks were not run. Dependencies, lockfile, toolchains and upstream/docs pins
are frozen; local commits only, no push.
