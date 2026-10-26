# Lowercase malformed-escape diagnostics

These MPL-2.0 controls target pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Fourteen existing authored sources
are referenced unchanged: thirteen legacy lowercase refusals and the earlier
`unicode-invalid` control. Sixteen adjacent authored cases cover first-error
ordering, character/string context, Unicode prefixes, imports, templates,
patterns and earlier grammar. No fixture is mislabeled as an upstream original.

`before.json.gz` retains 120 baseline observations at checkpoint `c99e74a`:
all 30 cases reject during loading; three have exact diagnostics and 27 differ.
Public-error probes cover both loaders and record raw cursor/message/Display.
They exit zero when successfully observing a refusal; that is not acceptance.
Paths use explicit placeholders and original stream hashes remain available.

The d79486e checkpoint measured 26 exact / four differing diagnostics.
Current tests require 27 exact / three differing diagnostics after the later-Char
correction. Cross-line literals, an astral observed unit and earlier function
grammar remain unqualified. All raw public errors must
remain equal to the measured baseline. The optional formatted diagnostic may
change `invoke` from Err to Ok(exit_code=1); external callers remain unaudited.

The separate matrix renews a pinned unknown-escape original, nine existing
positive controls, canonical U32 data, corresponding C/JS executions and native
baseline outputs. Refused programs are not executed. Pure native normalization
and actual IO remain distinct; strict IO checking is excluded. Complete upstream
normalized-graph equality is not claimed by that matrix.

Character quote checkpoint 6c26f207 closes the retained char-second observation. Current metadata:27 exact / three different of 30; historical receipts retain their original counts.
