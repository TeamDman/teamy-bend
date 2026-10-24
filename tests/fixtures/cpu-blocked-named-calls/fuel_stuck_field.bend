# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
type K is Type:
  Box{n: N}
def f(fuel: N, k: K) -> N:
  match fuel:
    case Z{}: Z{}
    case S{g}:
      match k:
        case Box{n}:
          match n:
            case Z{}: Z{}
            case S{p}: f(g, Box{p})
def main() -> N -> N:
  x => f(S{S{Z{}}}, Box{x})
