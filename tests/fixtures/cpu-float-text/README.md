# Float operations and text fixtures

`float_ops.bend`, `float_printing.bend` and `float_text.bend` are unchanged copies
from [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.
Their comments and expected outputs remain verbatim; actual code and captured
outputs determine the current formatting contract.

The two remaining files are authored MPL-2.0 controls, not upstream inventory
entries. `float-text-observations.bend` supplements the originals with paired
operation bits/text, read bits, signed zero, invalid strings returning None and
a finite text round-trip. Exact bytes are compared; no tolerance is applied.

`float-read-boundaries.bend` is a separate unresolved-contract probe, executed by
`scripts/compare-float-read.py`. Pinned C and JS differ for the decimal rounding,
hexadecimal and embedded-NUL inputs. Both reject the final-newline input. Native
Rust and port C match upstream C; port JS matches upstream JS. The probe retains
exit 1 while the upstream split remains, even when each port matches its backend.
It is excluded from the single-expectation positive suite and recorded explicitly
in the parity checklist. This grants no exception or new input/rounding policy.
NaN payload behavior and other host libraries are not qualified here.
