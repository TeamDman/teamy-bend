# SPDX-License-Identifier: MPL-2.0
import Base
import ../annotated_computed_scrutinee_000_neg.bend as H
def token(n: Nat) -> H.Bl:
  match n:
    case 0n: H.T{}
    case 1n+p: H.F{}
def value(a: Nat) -> H.Bl: H.main(token(a))
def observe(x: H.Bl) -> List<U32>:
  match x:
    case H.T{}: [1]
    case H.F{}: [0]
def apply(a: Nat) -> List<U32>: observe(value(a))
def main() -> List<List<U32>>:
  [apply(0n), apply(1n)]
