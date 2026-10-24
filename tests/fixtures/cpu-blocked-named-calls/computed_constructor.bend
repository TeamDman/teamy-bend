# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
type K is Type:
  Box{n: N}
def wrap(n: N) -> K: Box{n}
def f(k: K) -> N:
  match k:
    case Box{n}:
      match n:
        case Z{}: Z{}
        case S{p}: p
def main() -> N -> N:
  x => f(wrap(x))
