# Nested-array and split/join qualification

The unchanged originals come from `bendlang/bend` at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`:
`tests/reg/array_nested.bend` and `tests/run/array_split_join.bend`.
They retain upstream Apache-2.0 licensing and attribution; see
`licenses/Apache-2.0.txt` and `NOTICE`. New controls and harness use MPL-2.0.

The nested original covers the keep-free inner-array consumer regression.
The split/join original observes ordered halves, mutation and rebuilt indexing.
Authored controls transfer whole inner arrays, discard other owners through set,
mutate transferred String leaves, and observe all retained contents. A separate
control splits cloned U32 arrays of widths two and eight, changes the low-half
last cell and high-half first cell through wrapping indices, rebuilds, reads both
seam positions, and observes both the rebuilt and untouched original copy.

Negative controls retain the Data requirement on outer cloning and reject
reusing an affine split half. Existing affine swap and depth32 witnesses remain
under comparison. The port-C depth-refusal diagnostic remains unwaived.

These are logical ownership and ordered-value contracts, not identical physical
layouts, shared views, exact destruction timing or general memory-safety claims.
Selected generated-C normal-completion accounting is checked separately. Strict
proof IO exclusions do not stand in for executable qualification.
