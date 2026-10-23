# Float comparison, text round-trip and special-value fixtures

The unchanged originals are from
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests):
`run/float_text_roundtrip.bend`, `compile/float_compare.bend` and
`run/float_specials.bend`. Copyright 2026 HigherOrderCO; Apache-2.0.
See NOTICE and `licenses/Apache-2.0.txt`. Original comments/bytes are preserved.

The two authored controls use MPL-2.0. `float-special-observations.bend` exposes
finite/infinity/signed-zero bits and text, comparison predicates and U32
conversion boundaries. NaN is observed through text and comparison behavior;
its payload and sign bits are not read. Runtime F32.read remains a separate
unresolved contract.

`float-specials-combined-depth.bend` preserves the valid combined control that
upstream C/JS accept but all port executable paths refuse during checking at
the existing kernel nesting limit. Its factored companion produces identical
upstream output and runs in the port, without qualifying the combined program.
The open resource failure is reproducible with `-Suite float-specials-depth`,
which retains a failing exit. It is separate from `-Suite all`'s positive cases;
this separation grants no exception or resource-limit increase. The initial
invalid control missing reusable Nat binders also remains in private evidence.
