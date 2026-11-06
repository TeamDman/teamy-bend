# Stack-safe substitution and explicit release — 2026-11-06

The user has directed that parser and kernel depth thresholds be treated as
temporary implementation constraints, not as upstream language limits. They
should be retired as their implementation paths become stack-safe, while
retaining meaningful work and memory protections. This change is an initial
step; it does not remove those thresholds or claim end-to-end deep-term support.

`kernel::substitute` now traverses core terms with explicit heap work frames
instead of recursively calling itself. It preserves the prior binder-shadowing
rules and reuses unchanged constructor and application allocations. Frame
storage is proportional to pending work; substitution retains its existing
infallible API and does not introduce a new input-size allowance. Its callers
still have their existing validation and work limits.

The additive `kernel::release_term_iteratively` consumes an owned `TermRef`
and releases uniquely owned children with an explicit heap work list. Shared
children remain alive while another owner exists. It has no semantic checking
side effects and does not alter the public `Term` enum or `Rc` representation.
It is **explicit**: ordinary `Rc<Term>` destruction remains recursive, and an
external owner of a deep shared graph must also release its last root safely.
The test's unwind cleanup uses an explicit guard; ordinary kernel failure paths
were not converted in this earlier slice. A [subsequent cleanup slice](cpu-stack-auto-cleanup.md)
guards selected substitution, child-mapping and memo paths automatically; it
does not change ordinary public `Rc` destruction. The work list uses memory proportional
to pending edges; no arbitrary depth threshold was added.

## Evidence and remaining boundaries

The test-only recursive reference implements the previous substitution rules.
The new implementation matched it in **6,000 deterministic term/target
comparisons**, including binder shadowing, varied term families and root identity
checks. Separate controls substitute through a 20,000-node chain, release a
100,000-node shared graph, test every composite edge, and explicitly clean up a
100,000-node graph during a deliberately caught panic. These run on threads
requesting a 64 KiB stack. They exercise the new APIs directly, not a complete
checker or evaluator at those depths.

For contrast, a disposable isolated process using ordinary `Rc<Term>` drop
completed at depths 128 and 10,000, but aborted with host stack overflow at
100,000 on this Linux host; core dumps were disabled for the probe. A toy
automatic iterative destructor handled the same depth and shared ownership,
including caught-panic cleanup, but adding `Drop` directly to the public `Term`
enum would prevent Rust consumers from moving fields out of owned terms. That
toy was not integrated and does not establish an API-compatible automatic
teardown design. The observed 100,000 threshold is host-specific, not a safe
new limit.

With the final candidate library, the earlier direct-cut construction of the
original matcher again produced the exact upstream alpha-tree (depth 138, 610
occurrences) and matched the observed open-call alpha-tree, twice. The unchanged
original still returns `True{}` upstream and fails port loading at the pattern
compiler's 64-level guard; whole-function normalization of the manual prototype
still fails the separate 128-level engine guard. Neither is a passing original
execution. The unchanged pin is `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

The next steps are iterative pattern compilation with bounded work and pending
state; iterative freshening, strong normalization, comparison and checking;
and stack-safe automatic ownership/error cleanup, including public `Rc<Term>`
consumers. The input inspector itself already uses a heap stack, but passing a
deeper term onward before the other paths are safe can still overflow. Derived
`Debug`, source-provenance operations, parser and compiler traversals also need
review. Work budgets, allocation accounting and cancellation need deliberate
coverage; no mid-normalization cancellation behavior is claimed here. The
original witness and substantially deeper successful and negative cases should
be renewed as each guard becomes unnecessary.

Focused comparison subprocesses had 30-second limits. The outer runner uses a
1,200-second deadline and free-space floors of 11 GiB before launch, 10 GiB
while active and 1 GiB temporary storage. No extra OS memory cap was imposed.
Rust 1.96.0, dependencies and the upstream pin were unchanged. GPU and other
host platforms were not tested.

The final-source focused run passed 11 tests across the iterative-substitution,
shared-input and String-rendering targets, with zero failures and zero ignored;
strict targeted all-feature Clippy passed.
The complete final-source offline test set passed in bounded batches:
**1,382 passed / 0 failed / 49 ignored**. It comprises all 200 integration
targets in two batches, plus library, binary and documentation tests. This is
not an uninterrupted standard-gate result.
The final-source standard gate passed formatting, strict production Clippy,
the all-feature build and real Node TCP/UDP provider check, then timed out at
1,200 seconds during tests. Its 136 completed groups had 952 passed, zero
failed and 48 ignored; the active group was interrupted and later groups were
unrun. The bounded test-set pass does not turn that gate into a pass.
The matching port executable SHA-256 was
`93c74b4121971c43edd3e8a8c0aa069cc27c61053e1e852cf87edeeded3ca23d`, and the
feature-matched library SHA-256 was
`13ec9e82c869c37c20306a6df06eb49123002d6dc24da2a3b819f1cb31244e08`.
The tested core term source SHA-256 was
`f5c8b056976b14ea52e54a154e87804dce16b8a1d75784f27d9d26289909c289`, and the
iterative-substitution test source SHA-256 was
`ba90433503d4b961b43ac5de691a3decb245fb6307833245355b1415be5afa60`.
