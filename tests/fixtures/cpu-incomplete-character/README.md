# Incomplete character diagnostic controls

Authored MPL-2.0 source exercises terminal diagnostic metadata against pinned
Bend e6676b080f25b1bc1bf5b5b7d7a17e22f8022599. Expected stderr is the exact pinned
checking output. Source bytes and original raw port errors are retained in
cases.json and differential evidence. CRLF originals and expected streams have
explicit -text attributes.

The two direct-argument controls deliberately use an unresolved reference: the
literal refusal occurs during loading before name checking. They are negative
parser controls, not accepted runtime programs. Earlier-grammar remains first.

Argument cases with a preceding parameterized function remain non-exact. A trace
confirms that the original corresponding argument stops before function lowering.
BMP/astral cases can fail legacy lexing even earlier and lose the corrected prefix;
escaped/astral observations also lie outside the helper's metadata contract.
These are separate unqualified limits, not relaxations.
No escape is decoded at the terminal metadata boundary. The original depth
refusal remains unwaived in the neighboring cohort.
