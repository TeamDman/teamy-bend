# U32-backed character literals

Authored MPL-2.0 controls for the pinned Base Char representation and escape
parser. `values.json` compares complete literal/constructor graphs, including
adjacent surrogate escapes as two codes. Canonical one-to-eight ASCII-hex escapes
are numeric character data; no host Unicode scalar is constructed by parsing.

Malformed/noncanonical non-scalars remain refused. Existing lowercase scalar
syntax differences remain unresolved. The 129-code literal limit and 95-code
kernel-depth failure are retained independently of successful shorter controls.

Native/C raw IO and JavaScript scalar refusal are separate contracts. Their
literal and explicit-constructor controls must agree within each runtime; this
does not claim cross-runtime agreement for non-scalar characters.
