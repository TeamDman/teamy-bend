# Import-header load diagnostics

Four originals are byte-identical to the pinned upstream files listed in NOTICE.
The thirteen negative entries in `negative-cases.json` exercise malformed headers,
suffix refusal, invalid-alias precedence, comments, CRLF, EOF, imported children
and first-import priority. Their `.stderr` files are exact pinned observations.
They reject while loading; rejected-source execution is not a runtime pass.

Three positive imports preserve complete normalized graphs and execute through
generated C/JS. Native pure `run_main` is separately classified as normalization.
The public loader fields in `public-errors.json` were measured before this fix.
`before.json` preserves compact pre-fix streams with only machine roots replaced.

The controls retain path-normalization and hash/Unicode-tokenization uncertainty,
earlier filesystem failures, and blanked import lines in subsequent parser and
checker diagnostics. Those fallback diagnostics are not claimed as exact parity.
No import resolution, evaluator, term identity, sharing or resource rule changes.

Newly formatted CLI load refusals use the existing `Ok(CliOutput)` with
`exit_code = 1`; public ParseError fields and Display remain unchanged. Optional
private metadata and Debug output can change. External invoke consumers remain
unaudited. No new public field or semantic exception is introduced.
