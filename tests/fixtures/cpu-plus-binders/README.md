# Reusable plus binders

The five `.bend` files are unchanged originals from the
[pinned Bend parse corpus](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.

Four IO mains qualify execution and strict-proof exclusion separately.
`plus_binder_term` must reject an unbound marker during checking. The accompanying
`normalization.tsv` records ten pure helper normal forms from the unchanged IO
books, observed twice with the pinned interpreter. Internal executable-checker
regressions compare actual normalized core and exact text. This does not grant
IO books strict proof certificates or claim that their function values can be
printed by standalone executable mains.
