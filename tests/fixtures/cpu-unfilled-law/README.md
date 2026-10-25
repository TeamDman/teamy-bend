<!-- SPDX-License-Identifier: MPL-2.0 -->
# Live unfilled-law diagnostics

The six entries in `negative-cases.json` under `check/` and `compile/` are
unchanged pinned upstream fixtures; see NOTICE. Five authored controls exercise
multiline, imported, pattern-context, template and shared-reference source uses.
Their `stderr.json` strings are exact pinned checker output, not host reports.
All eleven reject during checking, before executable emission or execution.

Five `positive/` entrypoints cover filled declarations, erased use before filling,
structural recursion, imported completion and sealed Base numeric/IO contracts.
The first four normalize to `7`; executable C/JS print `7\n`. Native run_main for
these is pure normalization. `native-contract` executes IO and prints `3\n`;
strict proof checking excludes its IO type.

`adjacent-open-cases.json` contains two unchanged upstream negatives that never
use the open law live. Their final incomplete-proof inventory diagnostics remain
different. They are not included in the exact-match test or claimed fixed.
