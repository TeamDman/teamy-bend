# Multi-scrutinee normal forms

The 17 unchanged originals `multi_scrutinee_000.bend` through `_016.bend` are from
[bendlang/bend flatten tests at e6676b080f25b1bc1bf5b5b7d7a17e22f8022599](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and
`licenses/Apache-2.0.txt`.

These function-valued mains exercise two/three-column matching, repeated binders,
constructor reconstruction and fallback order. Checking and exact normalized
core/text are qualified separately from standalone printer applicability. The
originals omit Base; upstream generation refuses that prerequisite before inspecting
the function printer. Authored probes add Base and prefix local datatype/constructor names to avoid Base collisions;
they establish the function-printer
refusal separately, without changing these originals.

`applications.tsv` records 100 concrete constructor-argument vectors and actual
pinned interpreter results, observed twice. They cover the zero/successor or
empty/present combinations of each two/three-column function, including fallback
reconstruction and repeated binder names. Rust regressions call the checked
original definitions directly; these results do not imply compiled executability.
