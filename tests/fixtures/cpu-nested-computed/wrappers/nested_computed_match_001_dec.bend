# SPDX-License-Identifier: MPL-2.0
import Base
import ../nested_computed_match_001_dec.bend as H
def token(n: Nat) -> H.Bl:
  match n:
    case 0n: H.T{}
    case 1n+p: H.F{}
def value(n: Nat) -> H.R: H.main(token(n))
def observe(x: H.R) -> List<U32>:
  match x:
    case H.K5{}: [5]
    case H.K6{}: [6]
    case H.K7{}: [7]
def apply(n: Nat) -> List<U32>: observe(value(n))
def main() -> List<List<U32>>:
  [apply(0n), apply(1n)]
