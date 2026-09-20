# Numeric execution contracts

The execution-only Base supplies all 37 numeric primitive contracts from
Bend 2.0.5. They cover conversion, arithmetic, powers, transcendental functions,
rounding, comparisons, bits and F32 text conversion. The exact inventory and
signatures are in [the executable Base](../src/syntax/executable-base.bend),
along with ten ordinary F32 helpers. `F32.show` requires its upstream reusable
argument; `F32.read` returns exactly `Maybe<&2, F32>`.

Only the embedded loader grants these identities. A declaration with a familiar
name in a user file cannot obtain a host implementation. This registry is
separate from foreign IO imports. Executable checking validates signatures,
source ordering, quantities and ordinary proof bodies; the numeric laws remain
opaque to proof normalization. Strict `CheckedBook` checking still rejects
unfilled laws, including these execution-only contracts.

Native IO actions evaluate the operations through bounded continuation frames.
Closed literal words and numeric results use compact raw bits internally, with
lazy constructor views preserving the structural `U32`/`F32` Word interface.
Packing requires the exact checked, loader-sealed Base layouts. Ordinary
constructor fields remain lazy, and strict proof/data programs keep their
original representation. Float packing preserves all 32 bits, including NaN
payloads; it does not convert through a host floating-point value.
Arithmetic rounds to binary32; comparisons follow IEEE behavior, including
false equality and ordering comparisons with NaN. `F32.to_u32` truncates finite
values in the unsigned range and returns zero for negative, NaN, infinite or
overflowing inputs. Rust's saturating cast alone would not implement that rule.

`F32.bits` preserves all input bits, including signaling NaNs. Sign operations
also preserve the other payload bits. Upstream JavaScript promotes a source
binary32 value to a Number and quiets a signaling NaN on the round trip. For
example, a raw word of 2139095041 remains that word natively, while upstream
JavaScript returns 2143289345. No portable arithmetic NaN payload identity is
claimed. Thirty finite, signed-zero, infinity, conversion and comparison cases
match actual upstream JavaScript output; the separate signaling-NaN case records
this target difference explicitly.

The unchanged upstream `run/float_specials.bend`, `base/float_roundtrip.bend`
and `compile/float_compare.bend` now match. Allocation ablations localized
the last fixture's arena exhaustion to repeated ordinary U32 addition. The
native runtime optimizes sealed, checked Base `U32.add`, `U32.mul` and `U32.shl`
when their operands are compact words. It demands outer wrappers in the same
order as their source definitions. An ordinary Word field causes fallback to
the checked body without decoding unused bits. This also fixes an older eager
addition path: observing only the low result bit no longer forces an unused
input tail. Source bodies still check normally, proof reduction uses those
bodies, and these optimizations add no assumptions.
Boundary, partial-application, forged-origin, lazy-field and malformed-Word tests
cover this separate optimization; runtime limits remain unchanged. These results
establish the stated cases, not complete numeric program compatibility.

A 256-case integer comparison checks full/partial multiplication, constructor
round trips and shifts against actual upstream JavaScript and independent
modulo-2^32 arithmetic. Native execution now matches all 256 cases. Bounded
reclamation resolves the three reconstructed multiplications that exhausted
the previous append-only arena. Independent upstream normalization verifies
the lazy shift, zero-product and low-addition
observations, where eager generated JavaScript is not a laziness oracle.

Pure `run` normalizes without executing opaque numeric contracts, matching the
reference interpreter. Its surface printer remains incomplete: numeric literals
and application sugar still print as expanded core terms. Use an IO action to
execute these primitives natively. Pure evaluation, strict data calls and the existing pure
JavaScript/C compilers do not acquire numeric assumptions through this feature.

Executable JavaScript implements the same 37 contracts with native Number
values and binary32 rounding. Its signaling-NaN round trip matches the upstream
JavaScript target. All 31 generated numeric boundary cases and the three
unchanged upstream programs agree exactly on stdout, stderr and status.
Native execution agrees on 33 of those cases and retains the documented
signaling-NaN difference. Printable JavaScript pure entries also execute those
operations.

## Text and target behavior

F32 formatting follows the reference's search for a short decimal spelling
that rounds back to the same binary32 value, including `-0`, `inf`, `-inf` and
`nan`. Native formatting matches 20,012 bit patterns against the separately
compiled upstream C helper; JavaScript matches 20,000 against the actual
reference JavaScript helper.

The two upstream readers deliberately have different behavior. Generated
JavaScript uses the reference decimal/exponent, infinity and NaN grammar,
including leading JavaScript whitespace. Native Rust implements the C-locale
grammar with ASCII whitespace, hexadecimal floats and NaN payload syntax.
Both reject trailing whitespace. The native reader also preserves the C
helper's first-NUL behavior: a nonempty string starting with NUL reads zero,
and a valid numeric prefix before NUL can ignore following bytes. JavaScript
rejects embedded NUL. These execution contracts are not proof assumptions
about how untrusted external text should be validated by an application.

The safe Rust native reader matches Microsoft `strtof` on 14,862 boundary and
random inputs, including long hexadecimal mantissas, exact ties, overflow,
subnormals, signed zero and NUL tails. On MSVC targets, valid NaN spellings use
the observed all-one payload while preserving the sign. Other native targets
use the documented conventional payload policy; their host CRT NaN payload
behavior is not verified. Locale changes in a host process do not change this
reader's C-locale contract. JavaScript read matches 177,624 grammar/whitespace
inputs against the reference helper. Its equivalent linear regex also avoids
quadratic backtracking on long invalid digit strings.

Native scalar math follows the reference C lane's binary64 library operation
followed by a binary32 cast. JavaScript uses its own Math functions followed by
`Math.fround`. Domain NaNs and exceptional power cases can differ by target;
arithmetic NaN payload identity and universal cross-library bit equality are
not promised. Output comparisons therefore distinguish each target's oracle.
All 186 generated math cases match their respective C and JavaScript oracles;
finite values, infinities and signed zeros compare by bits, while NaNs compare
by classification. Fourteen unchanged upstream numeric IO programs also match
stdout, stderr and exit status on both targets.

## Ordinary integer readers

Nat and U32 decimal readers and their public helpers are ordinary checked Bend
definitions. U32 reading preserves the full unsigned range and rejects overflow.
Nat reading preserves the reference's 48-bit maximum, 281474976710655. Its
decimal guard avoids first expanding that huge unary bound for small inputs.
Generated JavaScript verifies the maximum, the first overflow and leading-zero
cases against upstream. Large native Nat values still exhaust the existing
representation limits explicitly; they are not reported as invalid input to
simulate a smaller accepted range. The ordinary Word/U32 addition commutativity
proof is now included and checked by structural induction.

Executable-only compact Nat now supports monotonic timestamps without unary
allocation. Values retain the reference native 48-bit bound and expose ordinary
Zero/Succ views on demand. Guarded fast paths cover Nat.sub, Nat.cmp, Nat.show,
U32.from_nat and U32.to_nat; they require loader-sealed origins and exact checked
bodies and transitive dependencies, modulo binder identities. Other bodies and
partially known arguments retain ordinary lazy evaluation. Strict proof/data
normalization does not use these fast paths, and materialized unary output still
obeys the original depth/node limits. General large-Nat arithmetic and reading
remain unfinished; this is not a claim of complete native Nat parity.

Remaining work includes native surface printing, executable C,
general large native Nat computation and structural word-pattern limits. There is
no separate signed integer language family in this reference revision.
