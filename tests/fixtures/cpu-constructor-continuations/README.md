# Constructor values and captured continuations

The five original `.bend` files are unchanged Apache-2.0 sources from
[pinned Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten):
`ctor_construct_000`, `ctor_construct_001`, `ctor_continuation`,
`ctor_continuation_go` and `ctor_default`. Normal forms, lowered graphs,
application observations and refusal text are Apache-2.0-derived oracle data.
New adapters, wrappers and tests use MPL-2.0.

Original builds require Base. Separate `applicability/` adapters import Base
without changing the originals; their function/Type results still refuse printing.
The local `Io` datatype is a pure constructor tree, not Base IO or a host action.

There are 25 direct applications of the original definitions, six applications
of the normalized nested continuation fields, and 36 projected observations in
the closed wrapper mains. The latter comprise 34 calls of wrapper `apply` plus
two direct callback observations. Complete constructor values and binder structure
are checked separately from these printable projections. Wrappers observe only
three constructor layers; they do not introduce a recursive effect interpreter.
