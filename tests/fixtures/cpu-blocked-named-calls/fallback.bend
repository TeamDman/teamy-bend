# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
type K is Type:
  Box{n: N}
def f(k: K) -> N:
  match k:
    case Box{S{p}}: p
    case Box{n}: n
def main() -> N -> N:
  x => f(Box{x})
