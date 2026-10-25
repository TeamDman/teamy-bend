<!-- SPDX-License-Identifier: MPL-2.0 -->
# Radix sort and list/map folds

The two originals are byte-identical copies from upstream Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`: `tests/base/list_sort.bend` and
`tests/run/list_map_fold_ops.bend`. They retain upstream Apache-2.0 attribution
through NOTICE. The radix fixture implements its own 32-pass sort; it does not
call bundled List.sort. Project-authored companions are MPL-2.0.

These IO entries are executable contracts. Pure checking refuses IO by design;
that result is separately measured, not runtime qualification. Expected outputs
are compared against pinned C O3 and generated JS, separately from direct Node
interpreter host failures. No resource budgets or dependencies are changed.
