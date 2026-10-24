# Field matching and destructuring

These eighteen unchanged originals come from [pinned Bend flatten tests](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and
`licenses/Apache-2.0.txt`.

Sixteen function-valued mains qualify exact normalized core/text and direct
applications. `field_match_in_order.bend` instead has an executable IO main;
`destructure_let_001.bend` must reject during source flattening. These are
separate contracts, with no generated execution claimed for function results.

`applications.tsv` contains 399 pinned interpreter vectors observed twice.
Inputs include five U32 values (zero, one, two, seventeen and the maximum),
zero/asymmetric/wrap-sensitive cases for each multiply worker, Nat depths zero through four and
lists of lengths zero through three. Exact values and core structures are
checked against the original definitions. Finite vectors are bounded evidence.
