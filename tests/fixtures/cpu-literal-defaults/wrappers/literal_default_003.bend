# SPDX-License-Identifier: MPL-2.0
import Base
import ../literal_default_003.bend as H
def from_nat(n: Nat) -> H.W:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def count(n: H.W) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def value(n: Nat) -> H.R: H.main(from_nat(n))
def observe(x: H.R) -> List<U32>:
  match x:
    case H.K100{}: [0, 0]
    case H.K101{}: [1, 0]
    case H.K102{}: [2, 0]
    case H.KD{k}: [3, count(k)]
def apply(n: Nat) -> List<U32>: observe(value(n))
def main() -> List<List<U32>>:
  [apply(0n), apply(1n), apply(2n), apply(3n), apply(4n), apply(5n)]
