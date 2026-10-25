# Decimal read bounds and error controls

The originals are byte-identical to `base/read_bounds.bend` and `base/show_read.bend`
in bendlang/bend e6676b080f25b1bc1bf5b5b7d7a17e22f8022599 (Apache-2.0; see NOTICE).
Their stdout files retain the original expectations.

MPL-2.0 companions import the original formatters and cover decimal bounds,
leading zeroes, malformed ASCII/Unicode/NUL input, explicit accumulators and
unselected helper arithmetic. These are IO mains. Proof-mode exclusions are
separate from execution. Pinned upstream generated JS reaches the original's
File.open error then fails importing bun:ffi; preserve that host failure rather
than counting the original as a full five-lane match. The numeric-only controls
are distinct programs, not a substitute for the unchanged original.

`unresolved/word-edges.bend` retains the complete 16-observation aggregate and
its required output. Native currently exhausts its existing step budget after
five lines. It is intentionally excluded from the passing integration test;
its individual/prefix and aggregate differential receipts remain required.
The historical direct-literal helper experiment refused the existing 128-Nat
parser cap; final helper controls express large values through ordinary Base
arithmetic, without changing that guard or claiming literal acceptance.
