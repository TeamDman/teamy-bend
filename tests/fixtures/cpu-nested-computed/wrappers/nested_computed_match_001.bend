# SPDX-License-Identifier: MPL-2.0
import Base
import ../nested_computed_match_001.bend as H
def from_nat(n: Nat) -> H.Nt:
  match n:
    case 0n: H.Zn{}
    case 1n+p: H.Sn{from_nat(p)}
def value(n: Nat) -> H.R: H.main(from_nat(n))
def observe(x: H.R) -> List<U32>:
  match x:
    case H.K5{}: [5]
    case H.K6{}: [6]
    case H.K7{}: [7]
def apply(n: Nat) -> List<U32>: observe(value(n))
def main() -> List<List<U32>>:
  [apply(0n), apply(1n), apply(2n), apply(3n), apply(5n)]
