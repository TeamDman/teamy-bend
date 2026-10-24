# Parameterized list constructors

The five original `.bend` files are unchanged Apache-2.0 fixtures from
[bendlang/bend at e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten).
The 78 applications cover all lists of length 0..2 over F/T for `_000` and Nat
depths 0/2 for `_001..003`. `_004` uses both Nat depths and the five list shapes
[], [0], [2], [0,2], [2,0] independently in its two list arguments (50 cases).
This is bounded sampling, not exhaustive arbitrary-list coverage.

The kind-dependent List declaration and its erased parameters remain intact.
New MPL-2.0 wrappers preserve result tags and separate fields, including every
list element in order. Exact complete constructor values are checked separately;
upstream's Nil-to-[] presentation is retained without rewriting original sources.
These projections do not qualify the unresolved imported-constructor printer.

The normal-forms JSON records the pinned interpreter's actual lowered graph in
the existing bounded Shape vocabulary, using bound-variable distances. It is
Apache-2.0-derived oracle data. Structural comparisons retain shadowed references
without parsing the printer-only `^n` notation or replacing it as text.
