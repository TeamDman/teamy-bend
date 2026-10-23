# Native Nat multiplication checkpoint

Target: unchanged [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Engine candidate: `099ffd23f4e4ff57c9c23bbe8725c4be83affc30`.
This closes the isolated native multiplication work-budget gap identified in
the [runtime-negative comparison](cpu-runtime-negatives.md). CPU parity and exact
negative diagnostic parity remain incomplete. No dependency, toolchain, safety
limit, upstream source or GPU implementation changed.

## Upstream law and native cause

Pinned [Nat.mul](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L551)
recurses through Nat.add in its checked source body. It matches the left operand
first; zero returns without using the right operand. A successor result may expose
its outer constructor without demanding either operand's remaining tail.

The compiled implementation uses compact Naturals. Its
[C helper](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L432)
checks multiplication against `NAT_IMM = 2^48-1` without wrapping a host integer;
its [JS path](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L262)
multiplies BigInts and applies the same Nat bound. Both emit
`bend: a Nat past the largest immediate 2^48-1\n` and status 1 for the original
[`run/nat_overflow.bend`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/nat_overflow.bend).
Its `2^24 * 2^24` is exactly the first disallowed product.

Native Rust already packed the U32-to-Nat operands but had no multiplication
fast path. It unfolded the Peano multiply/add bodies; U32.from_nat then demanded
the complete result, expanding successors until the two-million-step budget
was exhausted. The earlier result was a bounded refusal, not a wrapped number,
but it reached the wrong failure cause. This was avoidable algorithmic work,
not an intentional policy that rejects that mathematical input earlier.

## Correction and demand controls

The sealed native Nat dispatcher now handles multiplication of demanded packed
operands with checked host multiplication and the unchanged 48-bit result bound.
It returns zero immediately for a packed zero left operand. It never forces a
right operand merely because it might be zero; an invalid left operand still
fails first. Any ordinary constructor operand takes the original checked-body
fallback, preserving partial successor results and unused tails.

The existing guards require exact checked bodies/types and sealed transitive
origins, including Nat.add. Strict proof execution has no new fast path. The
operation remains charged, cancellable and subject to all existing arena/frame
limits. No broad arithmetic or CLI diagnostic rewrite was made.

Six new regressions cover the original unchanged overflow fixture, ordinary and
full-width products, products exceeding even u64, the exact maximum, partial and
higher-order application, zero/unused operands, lazy constructor tails, error and
side-effect order, cancellation and work exhaustion. Existing sampled comparisons
now include Nat.mul against the ordinary checked evaluator; origin guards also
cover a modified Nat.add. Forced collection tests retain both operands safely.

## Evidence

The standard gate passes **750 tests, zero failures, 49 ignored**; strict
all-target/all-feature Clippy passes. The Nat unit group passes 18 tests. The
original native overflow program now exits 1, emits no stdout and reports the
48-bit law. Its Eyre stderr wrapper still differs from upstream; the ledger cell
changes from `safety_limit` to `diagnostic_mismatch`, never to `pass`.

The repeated six-fixture negative matrix records **56 executions**: 26 exact
matches, 18 diagnostic mismatches, eight safety refusals, two unsupported native
foreign calls and two upstream-JS `bun:ffi` failures. Four C observations remain
inapplicable because the scalar fixture has only a JS companion. The runner
still exits 1 because required compatibility gaps remain. All observations are
deterministic, with zero timeouts and no residual execution-directory files.

A separate authored seven-line product control matches all five runtimes twice:
**10/10 exact executions**. Its stdout is
`0\n0\n21\n4294967295\n281474959933440\n281474976710655\n281474976710655\n`,
with empty stderr and exit 0. It covers partial application and nested products
through the inclusive maximum. This local control does not increase the count
of qualified upstream fixtures.

All previous ledger receipts were renewed: the 19-case positive/control suite
retains 184 matches plus six known upstream-JS host failures over 190 executions;
the 16-positive-comptime matrix retains 160 matches over 164 executions, including
supplemental upstream-JS stack-4096 runs. Four default upstream-JS stack failures
remain recorded. Every port comptime execution passes, including original hand
and tmpl. The same binary snapshot was used across all four comparison groups.

## Remaining mismatches are different kinds of gaps

| Observation | Classification and remaining requirement |
| --- | --- |
| Native Nat multiplication exhausted work before reporting overflow | Corrected algorithmic compatibility gap under unchanged limits. |
| 18 diagnostic observations differ despite matching refusal/order | Observable language/CLI contract gaps; not resource-policy exceptions and not exact passes. |
| C array class limit 17; three port stack limits | Explicit safety/resource-policy differences. These eight refusals stay unqualified; safe failure does not waive required compatibility. |
| Native arbitrary JS foreign `echo` unsupported | Missing implementation, not a proof-mode exclusion or a resource-policy choice. |
| Upstream JS `bun:ffi` unavailable; direct IO CLI needs Bun import.meta.require | Oracle-host blockers. No installation or reference-source workaround was attempted. |
| Default upstream JS stack fails on two positive comptime programs | Recorded oracle-profile limit; stack-4096 observations pass. Not a port CPU defect. |

No aggregate “all remaining failures are just budgets” claim is supported.
The correction also does not qualify every Nat operation or arbitrary lazy
constructor shape; the full lane/API inventory remains open.

The next small CPU step is exact failure-contract qualification for
`io/request_out_of_band.bend`: scope a language fail-stop path that retains cleanup
and output order while keeping host/tool errors on their existing reporting path.
Do not globally rename error prefixes to make the matrix green. Broader resource,
foreign, multiworker, host and corpus coverage remains separately required by the
[acceptance criterion](cpu-parity-acceptance.md).

## Reproduction

```powershell
cargo test --locked --lib runtime::nat::tests
cargo test --locked --test runtime_numeric
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 -Reference <reference> -OutputDirectory <fresh-negative-dir> -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
```

Private `cpu-nat-mul` evidence preserves exact commands, before/after streams,
fixture and binary hashes, authored product controls and receipt summaries.
Earlier evidence is retained. GPU, fresh Windows/macOS and multiworker profiles
were unrun; none is needed to continue the next bounded CPU task.
