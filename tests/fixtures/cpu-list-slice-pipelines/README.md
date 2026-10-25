# List slicing and pipeline controls

The originals are byte-identical to `base/list_slice.bend` and
`base/list_ops.bend` in bendlang/bend e6676b080f25b1bc1bf5b5b7d7a17e22f8022599
(Apache-2.0, see NOTICE). Their stdout files reproduce the original expectations.

MPL-2.0 edge companions import those originals unchanged and cover empty/zero
boundaries, unequal zip lengths, out-of-range updates, and empty any/all behavior.
These are IO mains; strict proof exclusions are separate from executable checking
and native/C/JS execution. Direct upstream Node's require failure is a host
limitation and is not counted as executable parity. These fixtures have no bearing
on whether the separate two-character string-pattern original executes.
