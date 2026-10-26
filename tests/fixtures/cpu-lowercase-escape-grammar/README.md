# Lowercase escape grammar qualification

These authored MPL-2.0 controls target pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. The two original reproducers are
existing authored lower-plus/lower-nine controls, not upstream test originals.
Five earlier fixtures are referenced without changing their source bytes.
Separately, the matrix renews byte-identical pinned unknown-escape and string
printer originals with their existing attribution.

All 34 negative controls are load refusals upstream. The baseline port accepted
14 and rejected 20. Both existing reproducers normalized to 65 and emitted C/JS
programs that executed to 65. `before.json.gz` preserves all 148 baseline
observations, including four actual generated executions. Root substitutions are
explicit; original raw stream hashes remain.

At b46f957 the corrected cohort had 19 exact diagnostics and 15 remaining differences.
The subsequent diagnostic-only correction qualifies the thirteen legacy cases,
so current tests require 33 exact / one differing diagnostic; the original
19/15 receipt remains historical.
Matching rejection is not exact diagnostic parity. Thirteen legacy malformed,
overflow and noncanonical non-scalar cases preserve their raw public errors,
recorded in `public-errors.json`. Those thirteen formatted diagnostics now match; their raw public errors remain
unchanged. The later-Char closing-quote now matches; the earlier grammar case remains open.

Newly invalid lowercase prefixes must reject before later U/q escapes; this
intentionally changes composed first-error results. Three earlier uppercase
family controls consequently become exact. Historical receipts remain unchanged.
Canonical one-to-eight-digit U32 codes remain valid, including non-scalars;
runtime scalar refusals and resource guards remain unchanged.

`positive-cases.json` selects nine controls for separate normalization/emission/
execution qualification, including one IO proof exclusion. Tests also preserve
full constructor graphs, 64-expression/128-literal guards, kernel input limits,
sharing, work accounting and the existing cancellation boundaries.

Character quote checkpoint 6c26f207 closes the retained char-second observation. Current metadata:33 exact / one different of 34; historical receipts retain their original counts.
