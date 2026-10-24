# Proof case trees

Unchanged Apache-2.0 originals from Bend e6676b080f25b1bc1bf5b5b7d7a17e22f8022599:
`tests/state/eval_case_tree_001.bend`, `eval_case_tree_002.bend`, and
`eval_split_apply.bend`. Derived oracle graphs, text and diagnostics retain
upstream attribution. New tests and adapters use MPL-2.0.

Proof-function normal forms retain named calls. Base-importing function adapters
are unprintable. In contrast, sixteen fully applied equation proofs check,
normalize to reflexivity and have an applicable closed-equation printer. Printing
`{==}` is runtime output evidence, not a replacement for strict proof checking.

Inputs: all four Dir constructors, all eight P3 triples over MN depths 0/1, and
MN depths 0/1/2/4. The same inputs exercise 36 helper computations: turn/back (8),
rot/getx/gety (24), and wob (4). Complete constructor values, scalar projections,
proofs, graphs and runtime lanes are separate observations. Three false endpoint
claims must reject. No proof is inferred from a projection alone. Imported JS
printer failures remain unresolved where observed.
