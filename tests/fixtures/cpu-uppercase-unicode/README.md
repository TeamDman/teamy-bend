# Uppercase Unicode escape qualification

The pinned parser accepts `u` or `U` followed by braces and one to eight ASCII
hex digits. The initial correction added only valid Unicode scalars to the port's
uppercase path. The later U32-backed literal correction also permits canonical
non-scalar codes. Limits and malformed-uppercase public errors remain unchanged.

`cases.json` records authored checking controls. `positive-cases.json` separates
upstream text, existing port normal-form text and C/JavaScript output. Tests
compare complete normalized structures as well as each recorded text; they do
not equate constructor display with a quoted String. The explicit IO control
exercises native execution; pure `run_main` remains normalization.

The two upstream originals are byte-identical to the pinned source (see NOTICE).
There is no uppercase-escape original in the pinned test inventory. The earlier
`uppercase-unicode.bend` authored reproducer is copied unchanged.

Unresolved observations, not approved policy exceptions:

- Surrogates and values above U+10FFFF are now accepted as U32 character codes
  under the pinned escape grammar; historical receipts retain the old refusal.
  JavaScript runtime scalar refusal remains separate from literal acceptance.
- Lowercase `u{+41}` and nine-digit leading-zero forms are accepted by the port
  but rejected upstream. This change does not broaden uppercase syntax to them.
- Native String normal forms use constructor text. The historical C printer discrepancy in
  `show/string_escape` is corrected by the separate C escape-printer checkpoint;
  its current C expectation now equals upstream. Historical receipts retain the
  pre-fix output. Native constructor text remains distinct.

No runtime budget, safety guard, cancellation boundary, dependency or toolchain
is changed. The checking census is historical until explicitly rerun.
