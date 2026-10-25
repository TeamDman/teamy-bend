<!-- SPDX-License-Identifier: MPL-2.0 -->
# Lambda-arm diagnostic controls

Five unchanged originals come from upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`: `check/erased_match_numeric`,
`check/undo_residual_lane`, `flatten/statement_beta`,
`flatten/statement_lam_absorb`, and `run/binop_probe_split`. See NOTICE.
They are source-load refusals, not executable numeric-match support.

`cases.json` contains those five plus nine authored negatives: multiline/imported
colons, constructor/function/application/annotation heads, a non-name after a valid
arm, and existing missing-body/closing-brace errors. `.stderr.json` stores exact
pinned output; `before.json` preserves compact pre-fix observations.

Four positive controls cover named/empty-call keys, parenthesized keys, a named
arm plus a function fallback, and numeric/function fallbacks without a colon.
Expected outputs are 7, 3, 4 and 7. The separate `shared-ref-head.bend` is a
parser-only macro/reference control: loading succeeds, but checking rejects the
undefined bare constructor value. It receives no checking/runtime pass credit.
Initial authored controls without required return types remain private failed
experiments; only corrected sources are qualified.
