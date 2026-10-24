# SPDX-License-Identifier: MPL-2.0
import Base
import ../annotated_computed_scrutinee_000_sel.bend as H
def from_nat(n: Nat) -> H.W:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def value(a: Nat) -> H.W: H.main(from_nat(a))
def count(n: H.W) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def observe(x: H.W) -> List<U32>: [count(x)]
def apply(a: Nat) -> List<U32>: observe(value(a))
def main() -> List<List<U32>>:
  [apply(0n), apply(1n), apply(2n), apply(4n)]
