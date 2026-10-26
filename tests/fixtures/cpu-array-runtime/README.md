# CPU array runtime qualification

The four unchanged originals are from `bendlang/bend` at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`:
`tests/run/array_fill_fold.bend`, `tests/run/array_poly_elem.bend`,
`tests/reg/array_keep_cow.bend`, and `tests/reg/array_copy_pair.bend`.
They retain upstream Apache-2.0 licensing and attribution; see
`licenses/Apache-2.0.txt` and `NOTICE`. New controls and harness code are MPL-2.0.

The controls observe wrapping indices, depth-zero storage, clone independence,
retained read/displaced String owners, exact finite/negative-zero F32 bits, and
affine closure cells moved through `Array.swap`. Rejections preserve ordinary
affine use, element types, and the Data requirement on `Array.get`.

Expected outputs describe executable IO. They do not claim that strict proof
evaluation supports IO or F32 axioms. A separate renewed depth-32 witness retains
the known generated-C resource refusal difference; no guard is relaxed.
