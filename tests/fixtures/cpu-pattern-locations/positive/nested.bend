# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{} S{p: N}
type Box is Data: B{v: N}
def tail(b: Box) -> N:
  match b:
    case B{S{p}}:
      p
    case B{Z{}}:
      Z{}
def main() -> N: tail(B{S{S{Z{}}}})
