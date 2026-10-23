# Runtime-negative regression

`array_depth_bound.bend` is an unchanged copy of `tests/io/array_depth_bound.bend`
from [Bend commit e6676b0](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/io/array_depth_bound.bend).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.

The original fixture must fail at execution before printing its array element.
Native rejection is covered without claiming upstream diagnostic-byte parity.

`nat_overflow.bend` is an unchanged copy of `tests/run/nat_overflow.bend`
from the same [Bend pin](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/nat_overflow.bend).
It shares the copyright/license above and exercises the first product beyond
2^48-1, without shortening the upstream workload or increasing a port limit.

`request_out_of_band.bend` is an unchanged copy of
[`tests/io/request_out_of_band.bend`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/io/request_out_of_band.bend)
from the same pin and shares the copyright/license above. Native, emitted JS and
emitted C regressions require stdout `3\n`, stderr `bend: runtime fail-stop\n`
and status 1; the effect must never execute.
