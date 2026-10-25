# Compact string-pattern design: no production adoption

The unchanged original still refuses at the 64-level pattern compiler guard.
The earlier field-fragment experiment only moved failure to the 128 input guard.
This follow-up investigates an existing-core typed local continuation, without
changing parser, checker, runtime, compiler, limits or dependencies.

The prototype is deliberately a **manual transformation of a Book**. It does
not make the original source load or execute. It extracts two one-character
matchers, removes their helper declarations, and inserts an annotated affine
local binding into `is_hi`. Its direct cut replaces the original continuation
subtree with the bound variable in function position. There are no generated
global definitions, additional declaration budget resets or unchecked types.

```text
Let k = Ann(suffix_matcher, String -> Bool)
    prefix_matcher_with_Var(k)_at_the_original_continuation
```

This is schematic core syntax, not a new Bend language feature.

Measured before production adoption:

| Requirement | Direct continuation cut |
| --- | --- |
| Compact core structure | depth 72 / 616 tree nodes, root 0 |
| Administrative-let expansion | exact full upstream alpha-tree, depth 138 / 610 nodes |
| Strict checking | accepts through existing annotation and affine-usage rules |
| Closed main | `True{}` |
| Six bounded closed inputs through pure and data APIs | expected True/False results |
| Blocked open call | exact upstream alpha-tree retained |
| Whole function normalization | **refuses existing normalization-depth 128 guard** |
| Unchanged original CLI/native/generated execution | **not implemented by this prototype** |

The alternative explicit `k(tail)` application also passes closed examples but
changes the blocked open call into an anonymous matcher application. It is a
reproduced normalization mismatch, so it is unsuitable as a transparent lowering.
Administrative expansion comparisons include all nodes and bound-variable indices,
not just top-level shape or counts. They intentionally remove the introduced
Let/annotation; provenance, binder spelling and pointer sharing are separate,
unqualified obligations.

## Architectural boundary and recommendation

Independent review recommends preserving the prototype as evidence and continuing
independent CPU qualification. No production correction is approved.

A complete design needs **typed compact continuations plus canonical normalization**.
The current flattener has patterns/variables but no expected type telescope;
it cannot safely invent continuation types for aliases, captures, recursive
functions, erased arguments or dependent motives. The normalizer then expands
ordinary lets before recursive strong normalization, recreating the depth 138
function. Neither a parser-only cut nor passing closed inputs closes that gap.

The next decision for Teamy is whether to prioritize that coordinated compiler/
normalizer milestone. The recommended contract remains exact pinned matching and
open normalization with all numerical guards retained. This is broader work than
a local depth-counter repair; it is not a request to raise limits. Investigation
of the existing-term design needs no semantic-policy exception. Any proposal to
change what a depth guard counts, exempt generated terms, or accept different
open normal forms must return for a separate explicit decision.

Before adoption, a design must establish resolved constructor/type identities,
closed affine continuation ownership, checking/error order and source locations,
structural descent and definition identity, complete normal forms, unchanged
lazy/strict demand boundaries, and honest work/depth/cancellation accounting.
Do not mark function values reusable or memoize before ordinary charge/enter
checks merely to make this example pass. General captured/recursive/dependent
factoring remains out of scope for this prototype.

## Reproduction

Build the frozen source with `cargo build --locked --features extended_observability`
after activating the provisioned environment, using debug=0/incremental=0 cache
settings. Then run from a clean checkout:

```bash
python3 scripts/compact-patterns/run-prototype.py \
  --reference <pinned-bend-checkout> --output <new-evidence-directory>
```

The runner compiles the standalone research probe against the matching feature
library, records source/library/binary hashes, repeats each observation twice,
and bounds each child to 30 seconds. It checks complete alpha-trees against the
pinned interpreter. A probe exit 0 means the diagnostic program completed; its
recorded normalization refusal is not a successful parity outcome.
The original plus one appended open-tail observation is clearly labelled as a
derived fixture, and the unchanged original remains a separate runtime cell.

Independent `base/list_slice` and `base/list_ops` qualification is recorded
separately in the following checkpoint. No proof-mode exclusion or successful
list control is evidence that the original string-pattern gap has closed.
