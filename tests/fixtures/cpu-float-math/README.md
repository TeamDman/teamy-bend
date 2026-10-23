# Float math fixtures

`float_unary_math.bend`, `float_trig.bend` and `float_tanh.bend` are unchanged
copies from [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/compile).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.
Original comments and expected outputs remain verbatim.

`float-math-observations.bend` is an authored MPL-2.0 control, not an upstream
inventory entry. It records binary32 bits alongside text for the twelve original
result expressions, plus square-root negative zero, log-zero negative infinity,
exponential overflow and the textual NaN result of square root of a negative
value. Exact bits and raw bytes are compared in the declared host profile; no
epsilon or ULP tolerance is applied. Arithmetic NaN payload/sign and results on
unrun host math libraries are not qualified by these observations.
