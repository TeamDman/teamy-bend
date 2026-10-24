# Data and affine grades

`accept_data.bend` is byte-for-byte from HigherOrderCO Bend 2.0.5,
[`tests/grade/accept_data.bend`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/grade/accept_data.bend),
at the frozen target `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and
`licenses/Apache-2.0.txt`. New support, fragments, manifests and tests are MPL-2.0.
Original SHA-256: `38f65e002ae9dd58a14c4624b879f1b73245ac44b6ff1142caeb38f8b8ab3dd2`.

Ten original definitions have exact text and full normalized graphs. Twenty-six
bounded direct applications cover reusable Nat/list/pair/phantom-tag values,
reusable equality evidence, and weakening. Eight additional observations consume
each stored affine closure once. There is no fabricated inhabitant of Empty and
no claim that the phantom function type contains a live duplicated function.
Proofs and functions remain graph observations, not constructor-only results.

Positive assembly is the unchanged original, newline, `support.bend`, newline,
and a `fragments` file. The two aggregates include all 20 constructor-valued
direct cases and all eight consuming observations respectively. Their record is
`Type`: an affine list is not incorrectly claimed to be reusable Data. Expected
text/graphs come from the pinned interpreter; tests compare every graph child.

Negative assembly is the unchanged original, newline, and a `negative` fragment
without its first SPDX comment line. This preserves the precise oracle line
windows. The controls retain correct rejection of affine duplication, erased
live use, invalid kinds, false evidence and an impossible Empty argument, with
multiline/nested/local/completed-law spans. The independent `imported` module
checks provenance across a loader boundary.

The original lacks Base and upstream executable generation refuses it. Both
`module-probes` import orders reject in both implementations: root list sugar
does not become the imported module's List constructors. These are applicable
refusals, not executable successes. The port's strict pure native constructor
API and standalone C/JS backends separately execute the original main and both
aggregates, checking complete constructor values. That does not establish an
upstream executable C/JS match for the original.
