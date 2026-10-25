# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{} S{p: N}
type Box is Data: B{v: N}
type Pair is Data: P{x: N, y: N}
def duplicate(b: Box) -> Pair:
  match b:
    case B{+v}:
      P{v, v}
def main() -> Pair: duplicate(B{S{Z{}}})
