# Constructor-definition diagnostic provenance

The `check/`, `grade/` and `comptime/` originals are unchanged copies from
[bendlang/bend at e6676b080f25b1bc1bf5b5b7d7a17e22f8022599](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests).
Copyright 2026 HigherOrderCO; Apache-2.0. See the repository NOTICE and
licenses/Apache-2.0.txt. New controls, manifests and tests are MPL-2.0.

`qualified-cases.json` lists exact negative diagnostic controls. All load, then
reject checking. Expected stderr files were captured from the pinned CLI, not
inferred from fixture comments. Compilation must reject without an artifact.
`pending-cases.json` preserves five compile-time substitution location mismatches;
those tests assert rejection only, not exact diagnostic parity. Qualified dotted
references retain the caller's source location, unlike bare template arguments.

`positive/constructor-template.bend` has complete normalized graph/text captures
and strict-pure native/C/JS comparisons. Executable emission requires Base; it is
not inferred from pure compilation. Historical fixture and failure receipts are
preserved separately. See the corresponding diagnostic report for hashes,
commands, exact tested source and remaining limitations.
