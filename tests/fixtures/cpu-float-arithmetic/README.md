# Finite float arithmetic and conversion fixtures

The originals are unchanged copies from
[Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests):

| Local filename | Original path |
| --- | --- |
| base-float-roundtrip.bend | base/float_roundtrip.bend |
| compile-float-roundtrip.bend | compile/float_roundtrip.bend |
| float_arithmetic.bend | run/float_arithmetic.bend |

Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and
`licenses/Apache-2.0.txt`. Only the two colliding filenames are disambiguated;
source bytes and comments remain unchanged.

`float-arithmetic-observations.bend` is an authored MPL-2.0 control. It exposes
exact binary32 bits and text behind the originals' aggregate results, including
precision loss at 2^24, both adjacent even-tie conversions and positive truncation.
It neither uses runtime F32.read nor establishes NaN payload or other-host
behavior. Runtime-read cross-backend differences remain separately unresolved.
