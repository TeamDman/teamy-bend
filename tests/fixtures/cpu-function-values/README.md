# Function-valued normal forms

The three original `.bend` files are unchanged Apache-2.0 sources from
[pinned Bend](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval):
`eta_expand_print`, `toplevel_ref_print` and `hoas_binder_refresh`.
The exact text and lowered graphs are Apache-2.0-derived pinned oracle data.

The graph comparison preserves the distinction between a bare reference and an
eta-expanded lambda/application. Bound-variable distance identifies the inner
binder in the shadowing case; printer spelling alone is insufficient evidence.

The new MPL-2.0 wrappers are separate closed observations. Each Nat identity is
applied to 0, 1, 2 and 7. Shadowing uses all nine pairs from U32 values 0, 7 and
4294967295, and must return the second argument. Successful wrapper execution
does not make a function-valued original printable or establish arbitrary-input
equivalence.
