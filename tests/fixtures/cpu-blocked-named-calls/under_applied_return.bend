# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
def f(x: N) -> N -> N:
  match x:
    case Z{}: y => y
    case S{p}: y => p
def main() -> N -> N:
  x => f(x, Z{})
