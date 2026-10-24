# Binder priority

The three original `.bend` files are unchanged Apache-2.0 fixtures from
[bendlang/bend at e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten).
Their comments and golden forms are preserved. The 36 application rows cover
ordinary predecessor matching (depths 0..3), cross-binder shadowing (all three
independent depths 0..2), and catch-all priority (depths 0..4). The first
fixture's historical sharing comment describes syntax absent from the source;
this batch does not qualify sharing or Own behavior.

New observation/IO wrappers use MPL-2.0. Row tags preserve K10/KN/K99 distinctions;
complete original constructor values are checked separately. These projections
supply no parity credit for unresolved imported-constructor printing.
