<!-- SPDX-License-Identifier: MPL-2.0 -->
# Declaration-boundary diagnostic controls

The nine `original/` files are unchanged from upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`: seven `parse/` fixtures and
`flatten/arrow_bind_error_000.bend` / `_001.bend`. See NOTICE.
Their actual applicability is source-load rejection, including fixtures whose
comments describe obsolete features. No execution is claimed for rejected input.

`cases.json` adds five authored negative controls for global input, punctuation
before/after constructor rows, direct arrows and imported provenance. Exact pinned
stderr is in `.stderr.json`; compact pre-fix observations are in `before.json`.
The original files and rejection status are preserved. This does not add GADTs.

Three `positive/` controls exercise empty and multiple datatypes, law completion,
inline rows and constructor fields with results 7, 5 and 5. `unsafe-boundary.bend`
is separately a parser-only decorator/empty-datatype control; it is not claimed as
a strict-proof or executable pass. Initial authored positive-source mistakes are
preserved in private receipts, not counted as qualified controls.
