# Typed definitions and list literals

`def_typed.bend` and `list_literal_sugar.bend` are unchanged copies of
[the pinned parse fixtures](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and licenses/Apache-2.0.txt.

The `_pure` derivatives change only each IO main to return its original U32
expression. They support strict normalization independently of the IO originals
and add no upstream inventory entries. The same attribution applies.
