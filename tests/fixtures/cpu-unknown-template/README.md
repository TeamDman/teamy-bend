<!-- SPDX-License-Identifier: MPL-2.0 -->
# Unknown-template diagnostics

Authored MPL-2.0 controls. `negative/template_missing.bend` and
`original/defs.bend` preserve the prior alias-fallback reproduction byte-for-byte.
Expected stderr is captured from unchanged upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599` using the checking-only adapter.

Four unchanged upstream originals (see NOTICE.md) and eight authored tilde negatives exercise standalone, missing/ordinary heads, imported and
multiline sources, template dry replay and nonleading markers. Missing-member and
unfinished-call controls preserve adjacent checking/load failures. Six successful
controls retain imported/raw template selection, repeated instances and ordinary
calls. Native pure results are normalization; generated C/JS execution is separate.

The public loader still returns the same ParseError fields and Display. Formatted
CLI failures now use the existing status-bearing Ok result with exit_code1.
Callers must inspect it; external invoke consumers remain unaudited.
