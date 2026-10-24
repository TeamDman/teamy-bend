# Open composition and blocked computed calls

The three original `.bend` files are unchanged Apache-2.0 sources from
[pinned Bend](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval):
`open_closure`, `open_closure_pipe`, and `stuck_split_mint`.
The text and lowered graphs are Apache-2.0-derived oracle data. The IO original's
observed helper is `pipe`; the pure originals use `main`.

Separate MPL-2.0 wrappers exercise 27 compositions for each helper: identity,
increment and double in both positions, applied to U32 inputs 0, 5 and 4294967295.
Five Nat inputs 0, 1, 2, 7 and 16 exercise the blocked-call original when closed.
The corresponding complete Bool values are compared separately from the wrapper's
U32 projection. Wrapper execution does not make function-valued originals printable.

The IO original remains outside strict proof checking. Its helper is tested in
the existing private execution-context test module without adding a public proof
API. Its unchanged main is separately executed. Type-printer oracle cases retain
nested function domains, right-associative codomains and dependent binder identity.
