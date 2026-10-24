# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
type K is Type:
  Box{left: N, right: N}
def f(k: K) -> N:
  match k:
    case Box{left, right}:
      match left:
        case Z{}:
          match right:
            case Z{}: Z{}
            case S{p}: p
        case S{p}: p
def main() -> N -> N:
  x => f(Box{Z{}, x})
