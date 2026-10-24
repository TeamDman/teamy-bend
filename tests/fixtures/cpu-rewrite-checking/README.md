# Rewrite checking diagnostics

The three `.bend` files are unchanged Apache-2.0 fixtures from
[Bend at e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse),
Copyright 2026 HigherOrderCO. See the repository NOTICE and
[Apache license](../../../licenses/Apache-2.0.txt).

`rewrite_non_equation` and `rewrite_ctor_brace` reject during checking;
`dead_motive_syntax` rejects during parsing/loading. Their comments and original
source bytes are preserved; no runtime success is implied.
