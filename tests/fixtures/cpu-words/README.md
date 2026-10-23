# U32 division regressions

These three unchanged fixtures come from [Bend commit e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests):
`run/word_div_mod.bend`, `base/u32_divmod.bend` and `base/u32_divmod_zero.bend`.
Copyright 2026 HigherOrderCO; Apache-2.0. See the repository NOTICE and
`licenses/Apache-2.0.txt`.

The complete programs cover total unsigned division/modulo, high-bit operands,
zero divisors and decimal formatting without shortening the upstream workload.
