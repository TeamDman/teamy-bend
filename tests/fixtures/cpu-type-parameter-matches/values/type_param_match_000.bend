# SPDX-License-Identifier: MPL-2.0
import Base
import ../type_param_match_000.bend as H
def from_nat(n: Nat) -> H.Nat:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def make_list(n: Nat) -> H.List<H.Nat>:
  match n:
    case 0n: H.Nil{}
    case 1n+p:
      match p:
        case 0n: H.Cons{H.S{H.Z{}}, H.Nil{}}
        case 1n+q: H.Cons{H.Z{}, H.Cons{H.S{H.Z{}}, H.Nil{}}}
def value(k: Nat, b: Nat, c: Nat) -> H.List<H.Nat>:
  match k:
    case 0n: H.main(H.Nat, make_list(b), make_list(c))
    case 1n+p: H.main((H.Nat -> H.Nat), make_list(b), make_list(c))
def main() -> List<H.List<H.Nat>>:
  [value(0n, 0n, 0n), value(0n, 0n, 1n), value(0n, 0n, 2n), value(0n, 1n, 0n), value(0n, 1n, 1n), value(0n, 1n, 2n), value(0n, 2n, 0n), value(0n, 2n, 1n), value(0n, 2n, 2n), value(1n, 0n, 0n), value(1n, 0n, 1n), value(1n, 0n, 2n), value(1n, 1n, 0n), value(1n, 1n, 1n), value(1n, 1n, 2n), value(1n, 2n, 0n), value(1n, 2n, 1n), value(1n, 2n, 2n)]
