# Framed dependent worker

`framed_cell_dep.bend` is byte-identical to HigherOrderCO Bend 2.0.5
[`tests/reg/framed_cell_dep.bend`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/reg/framed_cell_dep.bend).
Copyright 2026 HigherOrderCO, Apache-2.0; see NOTICE and licenses/Apache-2.0.txt.
SHA-256: `1be042ae06857025a7be7688cb1d6159caecb5aeacca6221d9214df6d169211d`.
New adapters, expected observations and tests are MPL-2.0.

The original prints `3` through IO. Its two worker definitions have complete
normalized graphs: the worker retains both binders, and the erased evidence
binder does not capture the earlier Nat argument. Four bounded inputs (0, 1, 3,
8) are tested through both the worker and caller. Adapters import the unchanged
original; no source definition is replaced. Graph fixtures retain every node.

Executable-core unit observations are not strict proof certificates. Strict
checking rejects the imported IO name; executable checking succeeds. Function
adapters normalize, but actual executable generation rejects their unprintable
function types. The two negative adapters distinguish a witness for the wrong
Nat from false reflexivity; both reject during executable checking.

`framed_cell_dep.host-stderr.json` records the separate upstream direct-CLI
Node host failure (`require` unavailable). Generated upstream C/JS execute the
unchanged original successfully; the host failure is not a program rejection.
No failed or unrun oracle lane is converted into an execution pass.
