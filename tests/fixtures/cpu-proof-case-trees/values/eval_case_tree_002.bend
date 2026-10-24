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
def main() -> List<Observation>:
  [value(0n, 0n, 0n), value(0n, 0n, 1n), value(0n, 1n, 0n), value(0n, 1n, 1n), value(1n, 0n, 0n), value(1n, 0n, 1n), value(1n, 1n, 0n), value(1n, 1n, 1n)]
