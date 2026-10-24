# Constructor checking controls

These five files are unchanged from Bend pin
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`:

- `nat_literal_import.bend`: `tests/check/nat_literal_import.bend`
- `tuple_import_required.bend`: `tests/check/tuple_import_required.bend`
- `list_sugar.bend`: `tests/import/list_sugar.bend`
- `qualified_constructor.bend`: `tests/parse/qualified_constructor.bend`
- `ctor_list_mismatch_000.bend`: `tests/page/ctor_list_mismatch_000.bend`

Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and
`licenses/Apache-2.0.txt`. Actual phase probes establish checking failures;
the list_sugar comment claiming a parse failure is preserved unchanged.

The hyphenated files are authored TeamDman regression controls under MPL-2.0.
They cover field counts, a different declared family, non-inferrable known
constructors, context, parentheses and an open-field provenance limitation.
`context-open` must still reject; its raw diagnostic is not claimed to match.
Source excerpts and expected diagnostics are asserted by
`tests/cli_checker_diagnostics.rs`. No captured local host diagnostics are included.
