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
its required output. At checkpoint e381cc7, native exhausted its existing step budget after five
lines. The path and full original remain unchanged for historical traceability.
The packed subtraction correction now includes this aggregate in the integration
test; individual/prefix and aggregate comparisons are renewed separately.
`sub-edges` adds modular subtraction, ordinary wrappers and partial application.
The historical direct-literal helper experiment refused the existing 128-Nat
parser cap; final helper controls express large values through ordinary Base
arithmetic, without changing that guard or claiming literal acceptance.

The two subtraction negatives preserve `before` on stdout, exact language stderr
and exit1 for a private request and overflowing Nat operand. Neither the hidden
effect nor the following output may execute. `.exit` files record oracle status.
