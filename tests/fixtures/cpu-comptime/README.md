# Constructor-tail regressions

`hand.bend` and `tmpl.bend` are unchanged copies of `tests/comptime` fixtures
from [Bend commit e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/comptime).
Copyright 2026 HigherOrderCO; Apache-2.0. See the repository NOTICE and
`licenses/Apache-2.0.txt`. Both whole programs print `50005000` and a newline.

The whole programs cover generated-JS constructor tails and native numeric
chains without shortening the upstream workload.
