# Annotated computed scrutinees and recursive list helpers

Seven unchanged Apache-2.0 originals from pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`:
`tests/flatten/annotated_computed_scrutinee_000`, `_band`, `_beq`, `_neg`,
`_sel`, `_str_eq` and `_str_len`. Derived forms, graphs and diagnostics retain
attribution. New tests, adapters and wrappers use MPL-2.0.

Every original main is a function. Original builds require Base; separate
Base-importing adapters reach function-printer refusals. The 77-input grid
uses seven lists (empty, T, F, TT, TF, FT, FF), all 49 equality pairs,
all Boolean helper cases, and selector depths 0, 1, 2, 4. Full graph/text,
complete closed constructor values, scalar projections and execution stay
separate. Earlier imported-printer failures remain unwaived.
