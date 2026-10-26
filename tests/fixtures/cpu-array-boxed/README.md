# Boxed-array clone qualification

The unchanged originals are from `bendlang/bend` at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`:
`tests/reg/array_clone_boxed.bend` and `tests/reg/array_heap_element.bend`.
They retain upstream Apache-2.0 licensing and attribution; see
`licenses/Apache-2.0.txt` and `NOTICE`. Authored controls and harness code are MPL-2.0.

The first original exercises intrinsic Array.clone of shared String leaves;
the second preserves sum tags through swap and its own recursive Arr.clone.
Controls add intrinsic cloning with owning String/nullary constructor changes,
Array.set owner disposal, unread clone disposal, leaf/depth-two storage, and
independently mutated surviving branches. Complete tagged strings and array
contents are observed, including values retained across later operations.

Negative controls preserve the Data requirement on clone and affine use of its
results. The existing affine swap control remains a positive Type-element check.
The existing depth-32 guard is renewed with its unwaived port-C diagnostic
mismatch. No limit changes are implied, including physical layout/stride limits.

Expected outputs are executable IO contracts. They do not claim strict proof IO,
identical physical layouts, exact destruction timing, or general memory safety.
Selected generated-C normal-completion counters are checked separately.
