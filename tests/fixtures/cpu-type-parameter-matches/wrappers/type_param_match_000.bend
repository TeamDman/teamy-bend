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
def count(n: H.Nat) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def observe(xs: H.List<H.Nat>) -> List<U32>:
  match xs:
    case H.Nil{}: []
    case H.Cons{h, t}: Con{count(h), observe(t)}
def apply(k: Nat, b: Nat, c: Nat) -> List<U32>: observe(value(k, b, c))
def main() -> List<List<U32>>:
  [apply(0n, 0n, 0n), apply(0n, 0n, 1n), apply(0n, 0n, 2n), apply(0n, 1n, 0n), apply(0n, 1n, 1n), apply(0n, 1n, 2n), apply(0n, 2n, 0n), apply(0n, 2n, 1n), apply(0n, 2n, 2n), apply(1n, 0n, 0n), apply(1n, 0n, 1n), apply(1n, 0n, 2n), apply(1n, 1n, 0n), apply(1n, 1n, 1n), apply(1n, 1n, 2n), apply(1n, 2n, 0n), apply(1n, 2n, 1n), apply(1n, 2n, 2n)]
