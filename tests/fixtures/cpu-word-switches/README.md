# Nested word switches, decrement and canonical joint matching

Four unchanged Apache-2.0 originals from pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`:
`tests/flatten/nested_word_switch_000`, `_001`, `word_decrement_halt`
and `joint_fused_canonical_le`. Derived normal forms, complete graphs and
refusal diagnostics retain attribution. New tests/adapters/wrappers use MPL-2.0.

Originals require Base for standalone compilation. Separate typed Base-importing
adapters reach actual function-printer refusals. Each binary function uses all
16 ordered pairs of depths 0,1,2,4; decrement uses depths 0,1,2,4,8: 53 inputs.
Nested default arms preserve whole matched values; the first nested switch's
KP adds one constructor to its entire second argument. Canonical joint matching
checks less-or-equal via structurally smaller recursive fields. Decrement always
returns Z. Complete values and scalar projections are separate oracles: switch
000 uses [100], [1,p], [2,n], [3,n,m] for K100/KP/KN/KM; switch 001 uses tags 1..4;
less-or-equal uses T=1/F=0; decrement projects the resulting depth.

Full original graphs/text preserve point-free joint fusion and named recursion;
closed applications and actual executable wrappers are different evidence.
Prior upstream imported-constructor printer failures remain unwaived.
