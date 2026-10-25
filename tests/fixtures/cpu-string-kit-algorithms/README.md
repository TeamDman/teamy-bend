<!-- SPDX-License-Identifier: MPL-2.0 -->
# Bundled string kit and local string algorithms

Originals are byte-identical to `tests/base/string_kit.bend` and
`tests/run/string_algorithms.bend` from upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, under the Apache-2.0 attribution
in NOTICE. MPL-2.0 companion wrappers import them unchanged.

The kit exercises bundled String/Char functions. Case mapping/classes and trim
are ASCII-based; strings otherwise retain Unicode code points. Split preserves
empty fields, lines splits LF without removing CR, and NUL remains internal in
the companion. Segment lengths and a join round trip distinguish empty fields
and preservation without invoking the host's NUL display contract.

The other original defines its own splice, split/join and bounded occurrence
algorithms. Occurrences overlap, require a final failed search to finish, and use
99 when their explicit fuel ends while active, including empty-needle cases.
This fixture-level fuel is separate from the frozen runtime safety budgets.
The splice uses an explicit index; an out-of-range index appends replacement.

These IO entries do not establish pure normalization or proof-mode availability.
Goldens are independently compared against pinned executable C O3 and JS.
