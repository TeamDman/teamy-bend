# SPDX-License-Identifier: MPL-2.0
import Base
import ../annotated_computed_scrutinee_000_band.bend as H
def token(n: Nat) -> H.Bl:
  match n:
    case 0n: H.T{}
    case 1n+p: H.F{}
def value(a: Nat, b: Nat) -> H.Bl: H.main(token(a), token(b))
def observe(x: H.Bl) -> List<U32>:
  match x:
    case H.T{}: [1]
    case H.F{}: [0]
def apply(a: Nat, b: Nat) -> List<U32>: observe(value(a, b))
def main() -> List<List<U32>>:
  [apply(0n, 0n), apply(0n, 1n), apply(1n, 0n), apply(1n, 1n)]
