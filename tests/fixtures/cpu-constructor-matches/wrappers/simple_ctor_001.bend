# SPDX-License-Identifier: MPL-2.0
import Base
import ../simple_ctor_001.bend as H
def from_nat(n: Nat) -> H.Nat:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def count(n: H.Nat) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def maybe(n: Nat) -> H.MaybeNat:
  match n:
    case 0n: H.None{}
    case 1n+p: H.Some{from_nat(p)}
def value(n: Nat) -> H.MaybeNat: H.main(maybe(n))
def observe(x: H.MaybeNat) -> List<U32>:
  match x:
    case H.None{}: [0, 0]
    case H.Some{p}: [1, count(p)]
def apply(n: Nat) -> List<U32>: observe(value(n))
def main() -> List<List<U32>>:
  [apply(0n), apply(1n), apply(2n), apply(3n), apply(5n)]
