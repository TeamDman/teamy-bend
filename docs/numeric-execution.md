# Numeric execution contracts

The execution-only Base supplies 16 sealed numeric contracts: `U32.to_f32`,
`F32.to_u32`, `F32.add/sub/mul/div/mod`, `F32.neg/abs/bits`, and the six
`F32.is_eq/is_ne/is_lt/is_le/is_gt/is_ge` comparisons. Their source signatures
follow Bend 2.0.5. The reference has 37 numeric laws without ordinary bodies;
the remaining 21 operations are still required work.

Only the embedded loader grants these identities. A declaration with a familiar
name in a user file cannot obtain a host implementation. This registry is
separate from foreign IO imports. Executable checking validates signatures,
source ordering, quantities and ordinary proof bodies; the numeric laws remain
opaque to proof normalization. Strict `CheckedBook` checking still rejects
unfilled laws, including these execution-only contracts.

Native IO actions evaluate the operations through bounded continuation frames.
Their arguments and results retain the structural `U32`/`F32` Word encodings.
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

The unchanged upstream `run/float_specials.bend` and
`base/float_roundtrip.bend` also match. The longer
`compile/float_compare.bend` program still exhausts the native thunk arena;
the resource limit was retained. These results establish the stated cases,
not complete numeric program compatibility.

The current pure `run` entry still prints proof-normalizer output, so an opaque
numeric application there remains unevaluated. Use an IO action to execute
these primitives. Pure evaluation, strict data calls and the existing pure
JavaScript/C compilers do not acquire numeric assumptions through this feature.

Remaining work includes executable generated targets, all transcendental and
rounding functions, F32 text parsing/formatting, the rest of the Nat/U32 library,
large Nat representation, and structural word-pattern limits. There is no
separate signed integer language family in this reference revision.
