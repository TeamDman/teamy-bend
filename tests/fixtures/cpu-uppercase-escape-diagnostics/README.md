# Malformed uppercase escape diagnostics

These MPL-2.0 controls compare the pinned Bend revision
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. The manifest references 13 existing
fixtures unchanged and adds 26 authored grammar/ordering controls, including an
intentionally malformed import declaration and a separate valid import into a
malformed child. No authored case is presented as an upstream original.

`expected/*.stderr` contains exact pinned checking output for every case.
`exact: true` denotes the 28 cases qualified for exact diagnostics; the other
11 remain rejection differences. Matching status alone is not diagnostic parity.
The separate runtime matrix renews an unchanged upstream unknown-escape original.

The eligible failure follows a pinned-compatible literal prefix and is either
inside a String or the first Char code. ASCII escapes followed by a BMP character
can use the existing source diagnostic. Later Char codes, noncanonical lowercase
prefixes, an astral observed character, cross-line literals and the earlier
function-parameter grammar discrepancy remain unqualified. These controls preserve
public loader errors, first-error distinctions and artifact refusal. Two earlier
q cases now withhold misleading optional metadata; raw errors are unchanged.

`before.json.gz` retains 78 baseline observations with exact status and lossless
streams. Root substitutions are explicit, and each raw stream hash remains.
No accepted U32 code, runtime scalar boundary, resource budget or guard changes.
