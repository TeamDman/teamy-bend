# SPDX-License-Identifier: MPL-2.0
import Base
import ../ctor_construct_001.bend as H
def from_nat(n: Nat) -> H.Nat:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def count(n: H.Nat) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def apply(n: Nat, m: Nat) -> U32: count(H.main(from_nat(n), from_nat(m)))
def main() -> List<U32>:
  [apply(0n, 0n), apply(0n, 1n), apply(0n, 3n), apply(1n, 0n), apply(1n, 1n), apply(1n, 3n), apply(3n, 0n), apply(3n, 1n), apply(3n, 3n)]
