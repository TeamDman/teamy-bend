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

Tested clean source: `fff765af30bf3ee4bf4b5632a2c73962917586fa`. Production remains unchanged.
Measured research results, **not approved for production adoption**:

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
library, records source/library/binary hashes, repeats each prototype/oracle run twice,
and bounds each child to 30 seconds. It checks complete alpha-trees against the
pinned interpreter. A probe exit 0 means the diagnostic program completed; its
recorded normalization refusal is not a successful parity outcome.
The original plus one appended open-tail observation is clearly labelled as a
derived fixture, and the unchanged original remains a separate runtime cell.

Independent [list qualification](cpu-list-slice-pipelines.md) passes separately.
No proof-mode exclusion or successful control closes the string-pattern gap.

## Renewed original, review and precise limits

The unchanged original's SHA-256 is
`44b3907ace3402fa1e69f265baf39539e247efc4308e16a62bf43a9d6bde97ed`.
Its [14 bounded observations](cpu-compact-patterns-original-failure.json) retain:
upstream checking accepts, upstream interpreter/C O3/JS return `True{}\n`,
exit 0 and empty stderr; port strict/executable loading and check/eval/run/emission
refuse with the 64-level pattern diagnostic. Port generated C/JS executions are
**unrun because no artifact was emitted**. No candidate original CLI execution
is claimed. These failures are separate from the manual Book prototype's successes.

[Compact probe receipts](cpu-compact-patterns-evidence.json) preserve exact refusal
text, open-call differences, complete-tree hashes and all process statuses. Seven
processes complete without timeout: one standalone rustc build and two runs each
of direct-cut, explicit-call and upstream normalization. Twelve closed observations
per candidate run cover six inputs via two APIs; both candidates pass these, while
only the direct cut preserves the observed open call. Process exit 0 does not turn
the recorded normalization failure into a pass.

Two independent read-only reviews found no blocker to publishing this research,
and neither approved production adoption. The library used here was built by the
clean-source full gate before capture; the receipt records its fingerprint and
SHA-256. The research runner itself selects a matching-feature rlib by modification
time and relies on the documented preceding build: it does not independently prove
library-to-HEAD provenance. The comparator treats Ann as transparent. The observed
trees have no annotations other than the intentionally introduced binding Ann;
rejecting unexpected annotations is a prerequisite before generalizing this probe.
No claim is made about arbitrary annotated programs, metadata or pointer sharing.

The unchanged 64-level recursive/wide-pattern, 128-level structural and engine,
source-hole, host-stack, named-call and aggregate work-accounting regressions remain
in the standard gate. It passes **1,169 / 0 / 49** at the source above; this is not a
claim that the candidate meets those general obligations. No new guard exception,
early memo bypass, budget reset, cancellation behavior or demand policy was adopted.
In particular, no mid-normalization cancellation support is implied.

### Source basis

- [Pinned string-pattern original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse/string_pattern_sugar.bend).
- [Pinned match flattening and strong normalization](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2679): nested constructor decisions and complete Mat arms/fallbacks remain observable core structure; term_snf begins at line 3033.
- [Port flattening guard and inputs](https://github.com/TeamDman/teamy-bend/blob/fff765af30bf3ee4bf4b5632a2c73962917586fa/src/syntax/surface.rs#L332): patterns and variables are available here, but no expected continuation type telescope.
- [Input structural guard](https://github.com/TeamDman/teamy-bend/blob/fff765af30bf3ee4bf4b5632a2c73962917586fa/src/kernel/term.rs#L361) is distinct from the [Engine nesting guard](https://github.com/TeamDman/teamy-bend/blob/fff765af30bf3ee4bf4b5632a2c73962917586fa/src/kernel/check.rs#L610).
- [Let reduction](https://github.com/TeamDman/teamy-bend/blob/fff765af30bf3ee4bf4b5632a2c73962917586fa/src/kernel/reduce.rs#L111) substitutes bindings; [strong normalization](https://github.com/TeamDman/teamy-bend/blob/fff765af30bf3ee4bf4b5632a2c73962917586fa/src/kernel/reduce.rs#L347) then traverses expanded arms and fallbacks. This explains why compact input alone does not solve full normalization.
- [Let checking](https://github.com/TeamDman/teamy-bend/blob/fff765af30bf3ee4bf4b5632a2c73962917586fa/src/kernel/check.rs#L1485) infers bindings before checking the body. A general lowering must preserve error order and per-use provenance as well as matching results.

The concrete decision requested from Teamy is prioritization of this coordinated
typed-lowering/normalizer milestone, retaining the same observable normal forms
and all guards. No safety-policy change is proposed. Independent CPU qualification
can continue without that decision. [Validation and provenance](cpu-compact-patterns-validation.json)
retain exact tested source, frozen inputs and historical receipt hashes.
