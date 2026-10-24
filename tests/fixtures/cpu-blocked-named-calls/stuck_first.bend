# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
def f(x: N, y: N) -> N:
  match x:
    case Z{}: y
    case S{p}: p
def main() -> N -> N:
  x => f(x, Z{})
