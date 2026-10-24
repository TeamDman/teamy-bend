# SPDX-License-Identifier: MPL-2.0
import Base
import ../nested_word_switch_001.bend as H
def from_nat(n: Nat) -> H.W:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def value(a: Nat, b: Nat) -> H.R: H.main(from_nat(a), from_nat(b))
def observe(x: H.R) -> List<U32>:
  match x:
    case H.K1{}: [1]
    case H.K2{}: [2]
    case H.K3{}: [3]
    case H.K4{}: [4]
def apply(a: Nat, b: Nat) -> List<U32>: observe(value(a, b))
def main() -> List<List<U32>>:
  [apply(0n, 0n), apply(0n, 1n), apply(0n, 2n), apply(0n, 4n), apply(1n, 0n), apply(1n, 1n), apply(1n, 2n), apply(1n, 4n), apply(2n, 0n), apply(2n, 1n), apply(2n, 2n), apply(2n, 4n), apply(4n, 0n), apply(4n, 1n), apply(4n, 2n), apply(4n, 4n)]
