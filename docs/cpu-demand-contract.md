# Proposed executable demand and private-request contract

**Proposal only; no semantics change is approved or implemented here.**
This review targets [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
The diagnostic engine `76038aa090780ea42976361b0e2f7b0de5412d00` retains native
laziness and the port's private-request refusal. These five cases remain open
under the [CPU acceptance criterion](cpu-parity-acceptance.md).

## Reproducers and exact observations

Each linked file is a complete, independent, authored Bend program requiring
only Base. They remove unrelated definitions from the previous thirteen-case
probe. `limit()` constructs `2^48-1`, `half()` constructs `2^47`; `head` matches
zero/successor and returns a string without inspecting the predecessor.

| Complete reproducer | Native Rust | Port C | Port JS | Upstream C | Upstream JS |
| --- | --- | --- | --- | --- | --- |
| [Unused argument](../tests/fixtures/cpu-demand-review/unused_argument.bend): `discard(Nat.add(limit(), 1n))` | K | N | N | N | N |
| [Partial addition](../tests/fixtures/cpu-demand-review/partial_add_overflow.bend): `head(Nat.add(limit(), 1n))` | P | N | N | N | N |
| [Partial doubling](../tests/fixtures/cpu-demand-review/partial_double_overflow.bend): `head(Nat.double(half()))` | P | N | N | N | N |
| [Unused right error](../tests/fixtures/cpu-demand-review/unused_rhs_error.bend): `head(Nat.add(U32.to_nat(4294967295), invalid()))` | P | R | R | P | R |
| [Ordinary left constructor](../tests/fixtures/cpu-demand-review/ordinary_rhs_error.bend): `head(Nat.add(Succ{invalid()}, invalid()))` | P | R | R | P | R |

The outcomes mean exactly:

| Code | Status | stdout | stderr |
| --- | ---: | --- | --- |
| K | 0 | `kept\n` | empty |
| P | 0 | `positive\n` | empty |
| N | 1 | empty | `bend: a Nat past the largest immediate 2^48-1\n` |
| R | 1 | empty | `bend: runtime fail-stop\n` |

All five runtimes run each program twice: 50 executions, using Node 24.19.0,
Clang 19.1.7 and C profile `--threads 1 --gpu off`. Generated port C keeps
`-std=c11 -Wall -Wextra -Werror -pedantic`; upstream C uses its unmodified emitted
runtime. Each generation/build/execution has a 60-second bound. Source hashes,
commands, binary hash and raw streams/status are retained in the private demand
receipt. No program prints the synthetic request payload `SECRET`.

These cases reproduce the earlier results; they are not regressions from the
array/scalar diagnostic correction. Six native observations differ from both
compiled upstream lanes in the first three cases. In the last two, four port-C
observations differ from upstream C, while four native observations agree with
C but remain unqualified because the upstream oracles conflict. Eight upstream
observations belong to those conflicting cases. A shared nonzero status or
agreement with only one oracle is not sufficient to close them.

## What the pinned sources say

The [guide's Nat rule](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/guide/GUIDE.md#L111)
describes a machine-word runtime Nat and abort beyond `2^48-1`.
[Arithmetic lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L254)
maps add/double to immediate arithmetic followed by the guard; the
[C guard](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L432)
posts the Nat failure when it exceeds that bound. By contrast, the recursive
[Base definitions](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/base.bend#L526)
can expose a successor without completing the tail in our lazy evaluator.
Those mathematical definitions alone do not establish the executable demand
contract for invalid machine values.

The pinned [BendTT paper source](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/docs/BendTT/main.typ#L481)
ties its runtime usage argument to weak by-value reduction and explicitly
describes the compiler as strict. The
[BendRT compilation account](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/docs/BendRT/main.typ#L193)
turns intermediate calls into continuations; its
[JS section](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/docs/BendRT/main.typ#L512)
states that ordinary CLI execution uses the compiled JS backend in memory.
Together with both observed compiled lanes, this is strong evidence for strict
live executable values. It is not a proof that every evaluation-order corner
case or unsafe program has a unique portable outcome.

The wildcard cases concern another contract. Their `invalid` helper is:

```bend
def reject(op: IO.OP<Unit>) -> Nat:
  match op:
    case Emit{x}: 0n
    case _: 0n
def invalid() -> Nat: reject(IO.print("SECRET")(Unit, x => Emit{x}))
```

The pinned [request-out-of-band regression](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/io/request_out_of_band.bend#L1)
states that a private request reaching an IO.OP match must refuse in every lane,
without executing the effect. It does not exempt a wildcard.
[JS lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L3034)
guards private requests before selecting any arm. The
[C lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2657)
retains the explicit default arm. In the actual generated reproducers, C calls
`invalid`, then that arm sinks the request and returns zero. Its positive result
does **not** demonstrate lazy C execution. This is evidence of an uncovered C
wildcard-guard inconsistency with the pinned regression's stated rule.

The runtime papers and executable regressions are the available pinned sources.
Some fixture comments cite SPEC sections 1.7/1.8, but this checkout contains no
standalone SPEC document. No absent specification text is inferred or quoted.
The [effect-driver account](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/docs/BendRT/main.typ#L497)
also separates pure request construction from effect execution by the event loop.

## Recommended decision for review

1. **Use strict live-value semantics as the target for executable native CPU
   parity.** A live overflow must refuse even if the called function discards
   its argument or observes only the outer Nat constructor. Preserve erasure
   and unselected branches: this does not authorize evaluating every branch or
   proof-only field. The first three reproducers would then have outcome N in
   native execution, matching both compiled oracles.
2. **Keep private requests unmatchable outside the IO driver, including wildcard
   matches.** When demanded, the last two reproducers should have outcome R.
   Do not execute the request, treat it as an ordinary constructor or weaken
   the port C guard to imitate the pinned C anomaly. Native currently avoids
   that demand; item 1 plus the existing request guard would expose the refusal.
3. **Record the upstream C anomaly explicitly rather than claim byte parity.**
   Adopting this language contract would still leave the two exact pinned-C
   comparisons different. Before declaring parity, the acceptance owner must
   either approve a narrowly documented conflicting-oracle exception based on
   the pinned rule, or keep exact pinned-C parity blocked pending an upstream
   decision. No such exception has been applied to the current ledger.

This choice aligns with the pinned runtime description and prevents differing
success/failure across native and compiled execution. Its cost is real: some
existing native unused-error and lazy Word-field expectations would need an
explicit mode/contract review, and more demanded work may reach unchanged
budgets. A global eager-force patch would risk order, sharing, termination and
performance and is not recommended. The current compact Nat representation
could remain an internal mechanism, but its executable demand boundaries would
need a separate design and regression plan. Proof normalization and executable
runtime semantics must not be conflated.

Keeping native laziness as a separately named evaluator is a legitimate
alternative, but it cannot silently count as matching this executable CPU target.
It would require an approved scope change and distinct acceptance lanes. We do
not recommend copying the C wildcard anomaly: that would contradict the pinned
request-refusal rule and the existing safe boundary.

Before implementation, review this decision, inventory native entry points and
lazy regressions, define source-order behavior for competing failures and test
representative ordinary constructors/Word fields. Then stage bounded native
changes with the five reproducers, unused-branch controls, budget/cancellation,
writer/cleanup and cross-runtime checks. Preserve all present behavior until
that review. No dependency or newer-upstream adoption is needed for the review.

## Reproduction

With the existing activated tools, use one fixture as `<case>`, a fresh writable
output directory as `<out>`, and the frozen checkout as `<reference>`:

```text
teamy-bend run <case>
teamy-bend compile --executable <case> --target javascript --output <out>/port.js
node <out>/port.js
teamy-bend compile --executable <case> --target c --output <out>/port.c
clang-19 -std=c11 -Wall -Wextra -Werror -pedantic <out>/port.c -pthread -lm -ldl -o <out>/port-c
<out>/port-c --threads 1 --gpu off
# Run these generators with cwd <reference>/bend2 and absolute case/output paths:
node main.ts <case> -o <out>/upstream.js
node main.ts <case> -o <out>/upstream.c
node <out>/upstream.js
clang-19 -std=c11 <out>/upstream.c -pthread -lm -ldl -o <out>/upstream-c
<out>/upstream-c --threads 1 --gpu off
```

Retain stdout, stderr and status separately, repeat twice, and enforce the
60-second process deadlines. The private `cpu-array-scalar-diagnostics` demand
runner and receipts contain the exact executed commands and hashes. The
reference remains unmodified; Bun/direct-CLI and GPU execution are not claimed.
