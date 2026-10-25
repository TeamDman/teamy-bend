# Synthesized-reference source origins

Four files are unchanged from HigherOrderCO Bend at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`:

- `tests/check/do_missing_bind.bend`
- `tests/check/error_window_assert_exists.bend`
- `tests/check/error_window_inline_exists.bend`
- `tests/import/empty_neq.bend`

Copyright 2026 HigherOrderCO, Apache-2.0; see NOTICE and licenses/Apache-2.0.txt.
New controls, expected observations and tests use MPL-2.0. Originals retain their
comments, including historical descriptions that must not replace actual phase
observations. All four parse/load and then fail checking on undefined synthesized
references in both implementations. No compiled execution is applicable after
those failures.

The negative manifest includes eight additional controls for multiline `exs`,
`where`, inline `&`, the `!=` token, missing monadic `pure`, statement-form `bind`,
and imported source modules. Exact stderr is captured from the pinned upstream
CLI. Checks assert status 1, empty stdout and no compilation artifact for all
public checking/evaluation/executable-compilation boundaries.

Two small positive controls define the required names and exercise the same
lowerings. Their complete constructor graphs and text agree with the pinned
interpreter. The logical control intentionally gives its local `Exists` a live
first argument; it tests fixed-name desugaring, not a replacement for Base's
existential type. Strict-pure C/JS compare typed constructor output separately
from executable compiler applicability. The existing real-IO fixture remains a
strict-proof exclusion and has separately retained Node host failures.

Origins are optional weak side-table metadata. The tests compare ordinary and
tracked books, checker failures/work/depth/freshness, and successful values after
metadata is dropped. Core representation, resolution, evaluation, demand, safety
limits and cancellation boundaries remain unchanged.
