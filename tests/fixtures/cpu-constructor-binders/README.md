# Constructor continuations and minted binders

The three original `.bend` files are unchanged Apache-2.0 sources from
[pinned Bend](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval):
`printer_fixpoint`, `printer_fresh_binder` and `printer_fresh_binder_pick`.
Exact normalized text and lowered graphs are Apache-2.0-derived oracle data.
The IO original's oracle entry is `pick`; the other two use `main`.

The new MPL-2.0 wrappers are separate observations: four continuation applications
at Nat inputs 0, 1, 2 and 7, and five applications of each nested-pattern function
at inputs 0, 1, 2, 3 and 7. Constructor tags, original text payload and returned
payload are retained in the continuation observations. These bounded samples do
not make the function-valued originals executable-printable.

The IO original remains outside strict proof checking. Its helper normalization
is tested in the existing private execution-context test module; this introduces
no public proof API or executable-to-proof conversion. Actual IO execution and
pure companion normalization remain distinct evidence.
