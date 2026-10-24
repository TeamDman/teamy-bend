# Wide scrutinee matches

The five original `.bend` files are unchanged Apache-2.0 fixtures from
[bendlang/bend at e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten).
Their comments and golden forms are preserved. The 96 applications use all
combinations of constructor depths 0 and 2 across four or five arguments.
This bounded sample covers nonadjacent discriminator columns, ignored columns,
default reconstruction and full payload forwarding; it is not arbitrary-depth
or exhaustive semantic coverage.

New observation/IO wrappers use MPL-2.0. Each result row preserves the constructor
tag (A=0, B=1, C=2) and every field's depth in order. Complete original constructor
values are checked separately. These projections supply no parity credit for the
unresolved imported-constructor printing discrepancy.
