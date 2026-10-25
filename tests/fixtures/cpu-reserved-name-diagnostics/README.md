# Reserved-name diagnostic controls

The eleven `parse/*.bend` files are unchanged Apache-2.0 upstream originals at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; see NOTICE. Authored controls use MPL-2.0.

`negative-cases.json` lists nineteen exact load-refusal cases: nine original
reserved-name gaps, nine raw-character/import/template controls and the already
matching `reserved_def_name_000` duplicate-Base-name control. The latter uses U32,
a library name rather than a reserved keyword. Raw `.stderr` files retain the
pinned bytes, including carriage return, tab and newline observations. Do not
normalize those goldens or the CRLF/EOF source fixtures.

Four pure positives cover free `as`, keyword prefixes, qualified names, constructor
fields and imported template replay. Their normalized graphs/text and generated
C/JS execution are checked separately; pure run_main is normalization evidence.
Baseline emitted artifacts in `before.json` are not execution claims.

`public-errors.json` records baseline public loader error fields for twenty-two
cases. Optional source diagnostics must preserve those fields and the parser's
cursor, fresh IDs, depth, tokens and rejection. Newly formatted CLI failures use
the established Ok(CliOutput)/exit_code1 boundary; external callers must inspect
status and remain unaudited. No public loader signature or error-field change.

`unresolved-cases.json` retains the distinct upstream lambda-binder diagnostic gap.
`controls/term-keyword.bend` retains the separate expression-keyword gap, while
`controls/earlier-depth.bend` retains the earlier 64-level parser refusal. These
controls are not exact diagnostic parity claims or new semantic exceptions.

An early positive control omitted explicit U32 operator typing and upstream
correctly rejected it as Nat/U32; the control was corrected in both lanes. An early
Python receipt comparison normalized a raw CR byte; subsequent comparisons use
raw bytes. Both runner/authoring attempts remain local evidence, not port defects.
