# Wildcard functions and empty-match values

The three original `.bend` files are unchanged Apache-2.0 sources from
[pinned Bend](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval):
`underscore_print`, `wildcard_chain_print`, and `efq_arm_value`.
Exact normalized text and lowered graphs are Apache-2.0-derived oracle data.

The two separate MPL-2.0 wrappers apply the wildcard functions to U32 inputs
0, 7 and 4294967295 (three unary and nine binary applications). The chained
function also has a partial-application helper. These observations do not make
the original function-containing results executable-printable.

The empty-match fixture is normalized and its `pick` function is applied to
both Boolean constructors. No value of `Empty` is fabricated, no empty-match
function is applied, and no runtime success is claimed for that original.
