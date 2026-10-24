# Ragged match rows

Five unchanged originals from [pinned Bend flatten tests](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/flatten).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.

`applications.tsv` records 81 checked direct applications observed twice with the
pinned interpreter: constructor depths 0..2 across each two- or three-argument
function. For `_003`, W fuel and Nat depth vary independently. This covers all
source rows and returned constructor families, including overlapping default
priority and recursive fuel exhaustion. It is finite evidence, not an exhaustive
proof over arbitrary depths.

Original mains return functions. Normalized core/text, strict direct applications
and generated runtime results are distinct contracts. The authored MPL-2.0 wrappers
import the unchanged originals and project results into lists of U32: one-based
constructor ID in declaration order, followed by every returned field's depth.
They retain every result field rather than using a checksum. Exact `.stdout` files
were confirmed against pinned interpreter, generated C at O3, and generated JS.

Direct printing of imported datatypes has a separately retained oracle discrepancy;
these observation wrappers do not qualify or waive that printer contract.

The `io/` adapters call the same observation wrappers and print each value through
`List.show` and `IO.print`. Their entry kind is asserted as IO, exercising the
native executable runtime. Pure `run_main` instead normalizes a core term; it is
covered separately and never used as evidence for the native effect driver.
