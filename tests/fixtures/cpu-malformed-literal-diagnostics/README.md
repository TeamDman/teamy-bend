# Malformed-literal diagnostics

Current manifests include 14 exact negatives and two unresolved UTF-16 controls.
The lowercase `unicode-invalid` source is now qualified without changing its raw
public error. The original batch notes below describe historical coverage; later
contextual, uppercase and lowercase checkpoints qualify additional cases. See
`cpu-contextual-lexical-diagnostics`, `cpu-uppercase-unicode` and
`cpu-lowercase-escape-diagnostics` for their distinct receipts.

## Original batch notes

The four parse originals are unchanged from the pinned Apache-2.0 source named
in NOTICE. `invalid_string_escape` and `word_overflow_literal` have exact load
rejection goldens. Thirteen exact cases include EOF, CRLF, leading zeros, BMP
characters, character literals, imported/template provenance and first-error
ordering. Three valid programs separately qualify complete normalized structure
and generated C/JS execution; native pure run is normalization.

`hex_no_digits` and `unclosed_char_code` remain unqualified. Their eager lexer
errors lack grammar context: declaration names and term positions require
different pinned expectations. Declaration-position and earlier-error controls
retain that disagreement. No context heuristic or diagnostic parser replay is
adopted. A general correction needs lexer/parser integration and a deliberate
first-error-order contract.

Other retained controls cover uppercase Unicode escapes (accepted upstream,
rejected here), malformed Unicode escapes and UTF-16 surrogate observations.
These are unwaived differences, not new semantic exceptions. The limited
unknown-escape metadata does not pretend they agree.

Public ParseError fields, Display, messages and next-token positions remain
unchanged. Private optional metadata enables the existing CLI Ok(CliOutput)
exit_code=1 boundary; external invoke callers remain unaudited. Resource limits,
sharing, runtime demand and cancellation boundaries are unchanged.

before.json retains compact original/adjacent failure evidence with machine roots
replaced. Goldens in unresolved cases record the upstream expectation only;
their existence does not mean the port matches it. Refused-source execution is
unrun. New authored fixtures/tests are MPL-2.0.
