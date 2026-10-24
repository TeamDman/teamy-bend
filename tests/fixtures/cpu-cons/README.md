# Cons terms and patterns

`cons_operator.bend` and `cons_plus_pattern.bend` are unchanged copies of
[the pinned parse fixtures](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and licenses/Apache-2.0.txt.

`cons_operator_pure.bend` changes only the IO main to return the same main.out
U32 value. It supports strict normalization independently of the original IO
program and adds no upstream inventory entry. It retains the same attribution.
