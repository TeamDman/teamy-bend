# Compact matching research inputs

`original-open.bend` contains the pinned Apache-2.0
`tests/parse/string_pattern_sugar.bend` followed by one authored open-tail
observation. It is a derived probe, not the unchanged original fixture.
See NOTICE for upstream attribution. `helpers.bend` is MPL-2.0 prototype input.

`scripts/compact-patterns/prototype.rs` manually assembles an existing-core
Book using these helper definitions. It removes the helper declarations and
inserts an annotated affine local continuation into `is_hi`. This is research:
it does not implement original-source lowering, and whole-function normalization
still refuses. The explicit-call alternative intentionally demonstrates a
normal-form mismatch. Neither prototype is a production port correction.
