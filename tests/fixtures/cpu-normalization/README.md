# Normalization fixtures

These six unchanged `tests/flatten/computed_operator_match_002*.bend` and
`tests/flatten/computed_recursion_00{2,3,4}.bend` fixtures
come from [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.

Three selector/flip companions (`computed_recursion_002_sel`, `_003_flip` and
`_004_sel`) are also unchanged copies from the same flatten directory.

Three Base-importing function companions (`computed_recursion_001_main_dec`,
`_001_not`, `_001_par`) are unchanged copies from that directory as well. Actual
upstream C and JS generation refuses their function-valued results before any
output file is produced; the comparison runner verifies this applicability
boundary before comparing normalized values.

The unchanged `computed_recursion_000`, `_000_big` and `_001` originals add Nat
fuel, nested patterns and reusable bindings. Their actual upstream C/JS printer
refusals are checked before interpretation comparisons as well.

Three further unchanged fixtures come from the same pinned repository:
`eval/stuck_rewrite_print.bend`, `proof/context_rewrite_hypothesis.bend` and
`proof/context_rewrite_value.bend`. They preserve stuck rewrites and checked
transport proofs. Their original comments and expectation records are retained.

Qualification uses the interpreter/normalizer. The upstream gate omits compiled
lanes without Base; the Base-importing stuck-rewrite function is independently
verified to be unprintable by upstream C/JS data printers. Core-structure and
exact-output comparisons are separate checks.

The three unchanged `parse/chain_suffix_f32_f.bend`, `_g` and `_h` fixtures
come from the same pin and retain their Apache-2.0 attribution above. They
normalize executable F32 functions; strict proof mode deliberately excludes
their runtime primitives. Both upstream data printers reject the function
results. The executable comparison observes parsed output, while the in-crate
regression separately compares the actual normalized core against the pinned
expectation. No executable contract becomes a strict proof certificate.

The unchanged `parse/chain_suffix_u32_f.bend`, `_g` and `_h` companions also
come from this pin. Their U32 arithmetic, nested call suffixes and reusable
shift/bitwise let binding normalize in strict proof mode. The runner compares
actual normalized core as well as raw text, after independently verifying both
upstream function-printer refusals. The F32 executable-only mode is not used for
these U32 fixtures.

The unchanged `parse/chain_brace_suffix_f.bend`, `_g` and `_h` originals retain
the same pinned attribution. They cover annotated U32 operators, a bare Nat
chain inside braces and a Nat literal in an equality-type result. All three
support strict checking and actual-core normalization. Both upstream compiled
printers refuse their returned function types; the runner checks that boundary.

The unchanged `eval/nat_literal_roundtrip.bend` comes from the same pin. It
retains complete and open-tail Nat values inside tuple/list syntax. Actual
upstream C and JS generation refuses the compound result containing a function;
the source comment alone is not used to classify compiled applicability.

The unchanged `parse/chain_default_nat_f.bend` and `_g` companions come from
the same pinned source. They retain Nat arithmetic precedence and comparison
functions. Strict normalization and actual-core comparison remain separate from
the C/JS function-printer applicability checks; no function invocation is claimed.
