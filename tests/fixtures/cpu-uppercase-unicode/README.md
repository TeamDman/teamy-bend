# Uppercase Unicode escape qualification

The pinned parser accepts `u` or `U` followed by braces and one to eight ASCII
hex digits. This correction adds only valid Unicode scalars to the port's
uppercase path. The original lowercase path, limits and invalid-uppercase
public errors remain unchanged.

`cases.json` records authored checking controls. `positive-cases.json` separates
upstream text, existing port normal-form text and C/JavaScript output. Tests
compare complete normalized structures as well as each recorded text; they do
not equate constructor display with a quoted String. The explicit IO control
exercises native execution; pure `run_main` remains normalization.

The two upstream originals are byte-identical to the pinned source (see NOTICE).
There is no uppercase-escape original in the pinned test inventory. The earlier
`uppercase-unicode.bend` authored reproducer is copied unchanged.

Unresolved observations, not approved policy exceptions:

- Surrogates and values above U+10FFFF are accepted as U32 character codes by
  upstream parsing; this port's literal decoder still requires Rust scalars.
- Lowercase `u{+41}` and nine-digit leading-zero forms are accepted by the port
  but rejected upstream. This change does not broaden uppercase syntax to them.
- Native String normal forms use constructor text. In `show/string_escape`, the
  existing C printer uses unbraced control escapes and a raw DEL; JavaScript
  matches pinned upstream. These outputs are retained separately, not exact
  cross-runtime parity.

No runtime budget, safety guard, cancellation boundary, dependency or toolchain
is changed. The checking census is historical until explicitly rerun.
