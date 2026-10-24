# Concrete array syntax fixtures

`array_literal.bend`, `array_write_stmt.bend`, `array_read_annotation.bend` and
`array_read_write.bend` are unchanged copies of
[`tests/parse` at Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.

Both upstream generators accept their concrete array-containing results.
The CPU comparison runner retains exact output/status requirements, including
pure-native normalization of the original write fixture. `array_read_four.bend`
is an authored MPL-2.0 regression minimizing the same reduction-depth failure.
Pure CLI/strict eval and the distinct checked `evaluate_data` API are qualified
separately; passing one does not qualify the other.
Core regressions compare explicit ALeaf/ANode trees, independently of the array
display's bracket notation, which is also used for lists.

The two `_pure.bend` derivatives retain their original helpers and replace only
the IO main with `def main() -> U32: main.out()`. They are separately qualified
Apache-2.0 derivatives, not substitutes for the originals' IO execution or their
strict-proof exclusions. The read/write derivative remains a required pure-kernel
work-budget failure with expected result 97 in the CPU comparison runner; it is
not asserted as a passing resource-policy exception.
