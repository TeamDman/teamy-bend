# Float runtime fixtures

`float_builtin_agreement.bend`, `float_accumulation.bend` and
`float_unboxed_ctor.bend` are unchanged copies from
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/compile).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.
Their original comments and expected outputs remain verbatim.

`float-runtime-observations.bend` is an authored MPL-2.0 control, not an upstream
inventory entry. It pairs binary32 bits with text for the finite rounding/sign
operations, ten-step accumulation and F32 constructor/array storage. Adjacent
signed-zero cases preserve the sign through ceil, trunc, floor and storage;
abs clears it. Infinity signs and the textual NaN classification cover nonfinite
inputs to the same rounding intrinsics. Arithmetic NaN payload/sign parity is
not asserted. These observations supplement the original finite workloads;
they do not qualify every F32 operation or host math library.
