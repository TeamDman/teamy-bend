<!-- SPDX-License-Identifier: MPL-2.0 -->
# Local string operations and bundled round trips

Originals are byte-identical to upstream `tests/base/string_ops.bend` and
`tests/eval/string_roundtrip.bend` at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, with Apache-2.0 attribution through
NOTICE. The MPL-2.0 companions import them unchanged.

Both original mains return IO(Unit), despite the round-trip fixture's Pure comment.
Their executable qualification is separate from strict proof-mode IO exclusions.
The local string_ops helpers include search, split, slicing, ASCII uppercasing and
space-only trim. Bundled String.trim also recognizes other ASCII whitespace; the
companion checks the distinction directly. Supplementary-Unicode separators,
field lengths and alternate-separator joins prevent a vacuous round-trip check.
NUL is kept internal and observed through equality/length, not host NUL printing.
The fixture-local Maybe search sentinel 99 does not denote a runtime budget error.
Goldens are independently compared with pinned executable C O3 and JS.
