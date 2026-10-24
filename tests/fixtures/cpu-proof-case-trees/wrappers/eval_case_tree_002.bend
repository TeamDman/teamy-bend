# SPDX-License-Identifier: MPL-2.0
import Base
import ../eval_case_tree_002.bend as H
def make(n: Nat) -> H.MN:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{make(p)}
type Observation is Data:
  Seen{rotated: H.P3, x: H.MN, y: H.MN}
def value(x: Nat, y: Nat, z: Nat) -> Observation:
  +p = {H.Mk{make(x), make(y), make(z)} : H.P3}
  Seen{H.rot(p), H.getx(p), H.gety(p)}
def count(n: H.MN) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def observe(o: Observation) -> List<U32>:
  match o:
    case Seen{H.Mk{x, y, z}, a, b}: [count(x), count(y), count(z), count(a), count(b)]
def apply(x: Nat, y: Nat, z: Nat) -> List<U32>: observe(value(x, y, z))
def main() -> List<List<U32>>:
  [apply(0n, 0n, 0n), apply(0n, 0n, 1n), apply(0n, 1n, 0n), apply(0n, 1n, 1n), apply(1n, 0n, 0n), apply(1n, 0n, 1n), apply(1n, 1n, 0n), apply(1n, 1n, 1n)]
