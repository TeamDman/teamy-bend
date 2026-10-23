# Application and operator-chain fixtures

These are unchanged copies of `tests/parse/adjacent_application.bend`,
`chain_paren_group.bend`, `chain_spacing_rules.bend`, `chain_brace_suffix.bend`,
`chain_default_nat.bend` and `chain_suffix_u32.bend` from
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/parse).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.
Original comments and output expectations are retained.

Qualification distinguishes pure interpretation and generated C/JS execution
from the direct upstream IO entry, which requires a different host capability.
The strict proof checker has a separate Base surface and does not admit IO;
its refusal is not a substitute for executable-mode verification.
