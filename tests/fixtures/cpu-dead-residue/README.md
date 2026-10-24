# Dead residue and default-row matching

Four originals are unchanged Apache-2.0 copies of pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599` tests/flatten/dead_residue_elision_000,
_001, _002 and _004. Oracle-derived forms, graphs and diagnostics retain that
attribution. New tests, adapters and wrappers use MPL-2.0.

Actual original mains: _000 is a function; the other three are concrete values.
All original upstream builds require Base. Separate Base-importing adapters
refuse function printing for _000 and generate artifacts for the concrete mains.
The original no-Base port acceptance bug is preserved in historical receipts.

The 29 inputs comprise 3 enum values, two 9-pair Nat grids and 8 state/counter
pairs. Main and worker-f normalization, direct applications, full imported
results and scalar projections are compared separately. No field-bearing
upstream-JS printer timeout is treated as complete runtime parity.
