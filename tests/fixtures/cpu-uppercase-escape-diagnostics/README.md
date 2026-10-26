# Malformed uppercase escape diagnostics

These MPL-2.0 controls compare the pinned Bend revision
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. The manifest references 13 existing
fixtures unchanged and adds 26 authored grammar/ordering controls, including an
intentionally malformed import declaration and a separate valid import into a
malformed child. No authored case is presented as an upstream original.

`expected/*.stderr` contains exact pinned checking output for every case.
`exact: true` denotes the 35 cases qualified for exact diagnostics; the other
four remain rejection differences. Matching status alone is not diagnostic parity.
The separate differential matrix renews an unchanged upstream unknown-escape original.

The eligible failure follows a pinned-compatible literal prefix and is either
inside a String or the first Char code. ASCII escapes followed by a BMP character
can use the existing source diagnostic. An astral observed character, cross-line literals and the earlier function-parameter grammar
discrepancy remain unqualified. Later Char codes now report the preceding closing-quote failure. Rejection and artifact refusal remain enforced.
Three formerly
unqualified lowercase-prefix controls now report the first malformed u escape,
changing the raw first error from a later U/q to u. Existing legacy per-escape
refusal messages remain unchanged. The earlier uppercase checkpoint remains
recorded as 28 exact/11 differing.

`before.json.gz` retains 78 baseline observations with exact status and lossless
streams. Root substitutions are explicit, and each raw stream hash remains.
Canonical U32 code values, runtime scalar boundaries, resource budgets and guards
remain unchanged; the separate lowercase repair rejects noncanonical spellings.

Character quote checkpoint 6c26f207 closes the four later-code Char observations. Current metadata:35 exact / four different of 39; historical receipts retain their original counts.
