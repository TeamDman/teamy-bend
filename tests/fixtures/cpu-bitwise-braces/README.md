# Dotted bitwise operators and brace forms

`bitwise_dot_operators.bend` and `brace_term_forms.bend` are unchanged copies of
[pinned upstream parse fixtures](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.

`bitwise_dot_operators_pure.bend` is an Apache-2.0 derivative that replaces only
the IO main with a pure U32 main calling the unchanged main.out helper. It does
not replace the original's execution or strict-proof applicability evidence.

Regressions preserve exact native output, strict normalized core and text,
annotation/equality/reflexivity forms, and the IO proof boundary. A negative
control changes only the brace equality's right endpoint from 3 to 4; the
unused false reflexivity declaration must still prevent checking the book.
