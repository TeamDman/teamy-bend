# Character closing-quote diagnostic controls

Authored MPL-2.0 controls for pinned Bend `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
The six retained earlier Char controls are referenced unchanged. Expected files preserve exact upstream stderr, including trailing spaces in source excerpts.

The 31 negative controls distinguish first-code escape failure from the immediate closing-quote check, with BMP/astral, U32, whitespace, import, template and parser/resource neighbors. The invalid-template/import probes intentionally retain their originally observed grammar failures; valid contexts are separate files. `exact` means exact CLI load diagnostic qualification, not successful execution. Raw public parser errors are recorded independently and must remain unchanged. Astral successor, empty/unterminated/multiline literals, length guard and unrelated grammar discrepancies remain explicit residuals.

Positive controls reuse existing scalar/non-scalar and IO fixtures and add four single-code Char controls. Generated C/JS execution, pure normalization and strict IO exclusion are separate observations. The pinned upstream originals remain byte-identical in their existing fixture directories, with their original attribution.

Current measured result at 6c26f207:25 exact / six different across 31 cases. The 23 exact flags are mandatory assertions; the two additional matches are unchanged invalid-template/import grammar probes. The compact receipt corrects their conservative labels without changing observations.
