<!-- SPDX-License-Identifier: MPL-2.0 -->
# Erased-binder diagnostics

The unchanged original is attributed in NOTICE. Nine negatives cover body-leading
minus, Nat/parenthesis/EOF/multiline/imported forms, nested blocks, a true argument
position and a missing binding value. Goldens retain exact pinned stderr.
`deep-body.bend` is a separate resource control: it must retain the expression
nesting refusal rather than receive the syntax diagnostic. Four positives cover
erased lets, reusable lets, subtraction and template replay. No unary-minus
support or acceptance change is introduced.
