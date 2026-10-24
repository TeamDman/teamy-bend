# Contextual diagnostic scope

`context_dedup_shadow.bend` and `arm_leaf_context.bend` are unchanged Apache-2.0
fixtures from [pinned Bend](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/page).
New MPL-2.0 sources exercise single and repeated shadowing, prior context-row
types, and imported source provenance. They reject during checking. Exact CLI
regressions distinguish outer `x^0` from inner `x` while preserving both rows;
none is counted as successful runtime execution.
