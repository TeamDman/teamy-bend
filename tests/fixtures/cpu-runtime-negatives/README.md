# Runtime-negative regression

`array_depth_bound.bend` is an unchanged copy of `tests/io/array_depth_bound.bend`
from [Bend commit e6676b0](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/io/array_depth_bound.bend).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.

The original fixture must fail at execution before printing its array element.
Native rejection is covered without claiming upstream diagnostic-byte parity.
