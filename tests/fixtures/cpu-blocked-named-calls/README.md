# Blocked named applications

`partial_app_snf.bend` is the unchanged Apache-2.0 original from
[pinned Bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval/partial_app_snf.bend).
The new MPL-2.0 cases minimize blocked constructor fields and cover fallback,
multiple fields, computed constructors, callee handoff and anonymous matches.

The exact text and lowered-graph oracle data were generated from that pinned
interpreter. Graph data is attributed as Apache-2.0-derived oracle data. The
bounded Shape vocabulary retains bound-variable distances, references,
applications, constructors and match arms. It does not parse printer-only `^n`
notation or erase names to hide an unfolded match.

Open normalization, closed applications and executable printer applicability are
separate contracts. The original fixture is not modified to obtain an executable.

The MPL-2.0 wrappers observe twelve concrete applications of the four minimized
functions and one double application of the original's returned lambdas. They
return complete List<U32> observations; their successful execution does not imply
that a function-valued original is printable.
