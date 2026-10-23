# Float syntax and pattern fixtures

`chain_suffix_f32.bend`, `float_exponent.bend` and `float_pattern.bend` are
unchanged copies from [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.
Original comments remain verbatim; actual generation, not their commentary,
establishes current compiled-lane applicability.

`float-bits-and-patterns.bend` is an authored MPL-2.0 control, not an upstream
inventory entry. It observes exact binary32 bits, midpoint rounding, signed-zero
pattern distinction, fallback values, unsigned boundaries and an unselected
Nat-overflow branch. The literal conversion follows pinned `Math.fround(Number)`;
this is distinct from the runtime `F32.read` text conversion contract.
