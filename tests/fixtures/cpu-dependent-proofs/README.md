# Dependent pair and equation proofs

Unchanged Apache-2.0 originals from Bend e6676b080f25b1bc1bf5b5b7d7a17e22f8022599:
`tests/proof/dependent_pair.bend`, `multi_column_match.bend`, and
`shared_scrutinee.bend`. Derived oracle graphs, text and diagnostics retain
upstream attribution. New tests and adapters use MPL-2.0.

Eleven complete definition normal forms retain dependent type-family cases,
constructor fields, multi-column matching and recursive rewrite evidence/motive/body.
Twenty-two direct applications comprise eight equation proofs and fourteen
helper/type-family results. All four Bit pairs and Nat depths 0/1/2/4 are bounded.
The packed dependent pair and fst extraction are checked separately. Invalid
S/U pairing must reject because isZ(S Z) is empty; false endpoints also reject.

Original builds need Base. Typed main adapters then generate. The complete
DPair value normalizes but refuses executable printing; its fst observation is
printable. Closed equations print {==} only after checking. Complete graphs,
scalar projections and runtime output remain distinct. Existing imported-JS
printer discrepancies are not waived.
