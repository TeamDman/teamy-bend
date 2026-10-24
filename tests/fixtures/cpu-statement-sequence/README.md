# Statement sequence and reflexivity

Four unchanged Apache-2.0 originals from pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`:
`tests/flatten/statement_sequence`, `_inc`, `_refl_f` and `_refl_w`.
Derived normal forms, complete graphs and diagnostics retain that attribution.
New tests, adapters and wrappers use MPL-2.0.

All originals require Base for standalone compilation. The two function adapters
reach actual function-printer refusals; the two nullary reflexivity adapters
compile. Eight inputs use W depths 0, 1, 2, 4 and check four sequential increments
versus one increment. Two nullary proof observations separately check reflexivity
of an identity function and of a constructor value. Complete graphs, text,
closed applications, projections and executable proof printing are distinct
contracts. A successful executable proof printer does not constitute proof
checking by a runtime. Historical imported-JS printer failures remain unwaived.
