# Bounded sharing experiment for string normalization — 2026-11-06

On port production source `13b54a2c7d72242a26f79a3588d4ec688e504712` and
pinned upstream `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, the unchanged
two-character string-pattern original again returns `True{}` upstream and is
refused by port loading at 64 pattern-compilation levels. Two repetitions of
each observation matched. This attempt does not make that original execute.

## Hypothesis and bounded prototype

The hypothesis was that a shared representation of repeated continuations
might permit inexpensive validation while preserving the existing limits. The
test-only `guarded_summary` prototype memoizes, per shared node, its maximum
unfolded height and its **expanded occurrence count**, not its unique-node
count. It can fast-accept only a small hole-free subset of existing `Term`
variants when height is at most 128 and occurrences are at most 2,000,000.
Holes, unsupported variants, excessive counts and excessive depth yield no
decision: an integrated design would have to fall back to the existing ordered
inspection. The prototype has a 128-level host-recursion bound, checked
arithmetic and a 2,000,000-unique-entry ceiling. It is confined to
[the tests](../tests/cpu_shared_input_limits.rs); no production checker,
normalizer or resource budget was changed.

Four focused adversarial regressions passed:

1. One subtree is first visited at a shallow depth and then through a deeper
   path. The real input guard accepts the constructed depth-128 control and
   rejects the depth-129 input. A simulated naive unique-node traversal would
   accept the invalid case, even when it checks the current depth before
   skipping a previously seen node. The summary refuses a fast decision.
2. A binary shared DAG has 20 distinct nodes but 1,048,575 unfolded occurrences;
   its depth and size pass input inspection. One more layer has 21 distinct
   nodes and 2,097,151 occurrences, so the existing size guard rejects it.
   Unique-node deduplication would wrongly accept it; the summary does not.
3. Reversing the order of a hole and an over-depth branch changes the existing
   first error from unfinished hole to input nesting. The summary declines
   both, preserving ordered inspection as the required fallback.
4. The expanded core of the earlier direct-cut string prototype has depth 138
   and 610 tree occurrences. When supplied as an input term, the real checked
   API rejects it at the distinct 128-level **input** guard. The test-only
   summary also makes no decision; it does not support the matcher's full set
   of term variants, independently of its depth. The term constructed by the
   test was also independently compared twice with the pinned upstream
   canonical alpha-tree and matched exactly.

The previous direct-cut prototype was separately rebuilt against the freshly
built feature-matched library. Twice, its entire expanded alpha-tree and its
blocked open-call alpha-tree matched the pinned upstream normal forms. The
expanded canonical stream SHA-256 was
`5d4ecab6cb8eeab4ad1c637303f25c6fab361d1002cb92eec1339d5b85023c69`.
Whole-function normalization still refused at the 128-level **engine** guard.
This distinct refusal does not prove how a future alternative representation
must behave; it shows that the tested existing-core output remains too deep to
resubmit through today's input API even if it could be returned by a different
normalizer. The corresponding compact prototype is research, not lowering.

## Outcome and remaining obligations

The summary experiment did not produce a sound repair of the string pattern.
Sharing alone does not reduce the unfolded maximum path. The restricted test
model does not establish an optimization for all term variants, provenance,
cyclic or hostile inputs, proof holes, cancellation, or exact work/error order.
It also declines variables and binders, so it does not compute the highest
binder identifier required by the real inspector's fresh-identity accounting.
Any real fast path would need its own bounded accounting, a complete fallback,
and proofs or adversarial tests that it cannot turn a refused input into a
successful check. None is installed. Nor does this attempt establish a
representation that both preserves canonical open normal forms and crosses
the input and engine boundaries without changing their safety contracts.
Typed lowering, demand/laziness, source provenance and error order remain open.

## Validation and limits

The four test cases above passed with zero failures and zero ignored, and
targeted all-feature Clippy passed with warnings denied. The original comparison
and both prototype/upstream comparisons had 30-second per-process limits.
The checked test source SHA-256 was
`03e85bf58e86e58c204062094e633fcdd51937db0db0cb6de04ed6df1420a077`;
the feature-matched prototype library SHA-256 was
`72340add0fd364943591f36c43fcb2e0dd8bbf493dd8b975918f38a880de1639`.
The bounded outer runner has a 1,200-second limit, with 11 GiB workspace free
before launch, 10 GiB during execution, and 1 GiB temporary-storage headroom.
No extra OS memory cap was imposed. Rust 1.96.0, the pinned reference, existing
dependencies and all Bend guards remain unchanged. GPU and non-Linux hosts
were not tested. The standard gate passed formatting, strict production Clippy,
the all-feature build and the real Node TCP/UDP provider check, then hit its
1,200-second deadline during tests. Its 135 completed test groups had
**950 passed / 0 failed / 48 ignored**; the active group was interrupted and
subsequent groups were unrun. It is not a gate pass. The new four-test target
passed separately. No new full test-set run was performed for this research-only
change. Previous complete bounded test-set and interrupted standard-gate
results are recorded separately in [the boundary report](cpu-string-normalization-boundary.md).
