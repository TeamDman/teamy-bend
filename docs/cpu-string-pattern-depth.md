# String literal pattern depth investigation

The original `parse/string_pattern_sugar.bend` remains unresolved. Production
source is unchanged. This checkpoint preserves a rejected-for-production
experiment, bounded neighboring regressions, and the precise next blocker.
Final validation receipts are added after the clean-source gate completes.

The pinned original matches `"hi"` and returns `True{}` upstream. The port refuses
loading at its 64-level pattern-compilation guard. Literal expansion is
`SCon` / `Chr` / `U32` / 32 `WCon` decisions per character. Compiler frames for the
first character retain unfinished arms and fallbacks while processing the next.
The existing guard is a local recursion/pending-work bound, not a global count
of all simultaneously retained states; a single-character pattern can have 69.

The unaccepted [fragment experiment](cpu-string-pattern-depth-experiment.patch)
retires a completed field's compiler working states before lowering the next
field. It retains the numerical guards, uses an Rc-identity continuation, and
keeps immutable output. Review identified a nested deferred-error ordering risk;
the archived version disables nested fragmentation. It is **not applied** and
has not passed comprehensive graph/provenance/error-order or full-gate review.
It must not be treated as a production correction.

That candidate loads the unchanged original, but check/eval/run and both emitters
then refuse at the separate **128-level kernel input guard**. Complete alpha-tree
comparison with the pinned parser yields `is_hi`: **138 levels / 610 nodes**;
`main`: **37 / 139**. Comparison includes every node, constructor, child order and
bound-variable index; it excludes binder spelling, source metadata and Rc sharing.
The graph probe fails on any unsupported node tag. The complete canonical `is_hi`
stream SHA-256 is `5d4ecab6cb8eeab4ad1c637303f25c6fab361d1002cb92eec1339d5b85023c69`.

Graph-preserving iteration or sharing cannot shorten a root-to-leaf path. Neither
skipping shared nodes in validation nor changing the threshold is an acceptable
repair. The next substantive task needs a reviewed design for compact lowering
that changes the intermediate graph while preserving checking, normalization,
binder/provenance, evaluation and safety behavior. No such design or resource
policy change is implemented here. Independent review recommends archiving the
experiment and retaining the refusal until that work is justified.

The seven positive controls cover empty/ASCII/supplementary-character matches,
mismatches, and longer-prefix rejection. They do not qualify the two-character
literal. Negative controls retain the 65-Nat and 65-column compilation refusals
and rejection of a String pattern on a Nat. The wide-column control also lacks
cases upstream: that is a phase difference, not matching diagnostics.

Reproduction uses `scripts/string-pattern-depth/compare.py`, the standalone
`phase-probe.rs`, `graph-probe.rs`, and `upstream-depth.mjs`. Rust probes link to
the exact matching built library; no dependency is installed. Host debug=0 and
incremental=0 settings control cache size, not Bend semantics. C uses the pinned
upstream O3 profile, CPU threads=1/GPU=off; port C additionally uses strict warnings.

The raw instrumentation receipt has an incorrect `base` field. Its correct base
is `6c2ba98c964247fc424a1d57ef2772dbfe2a16eb`; raw bytes remain preserved with this
explicit correction. Dirty experiment builds are never attributed to a clean
commit. Raw binaries/logs remain outside Git; compact source, patches and unique
failure receipts are preserved in the reviewed checkpoint.
