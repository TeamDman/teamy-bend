# String literal pattern boundary fixtures

`original/string_pattern_sugar.bend` is byte-identical to
bendlang/bend e6676b080f25b1bc1bf5b5b7d7a17e22f8022599,
`tests/parse/string_pattern_sugar.bend` (Apache-2.0; see NOTICE).
It remains an **unresolved** differential case, not a passing runtime regression.

Positive companions cover exact empty/one-character ASCII and supplementary
Unicode patterns, misses, and rejection of a longer string sharing a prefix.
The three negative companions cover the unchanged 64-level pattern budget and
an invalid Nat scrutinee. `columns65` also lacks cases upstream; its port load
refusal is only a resource-guard control, not exact diagnostic parity.
New companions are MPL-2.0. See `docs/cpu-string-pattern-depth.md` for experiment,
complete structural evidence, and separate executable/checking outcomes.
