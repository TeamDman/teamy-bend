# SPDX-License-Identifier: MPL-2.0
import Base
import ../word_decrement_halt.bend as H
def from_nat(n: Nat) -> H.W:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def value(n: Nat) -> H.W: H.main(from_nat(n))
def count(n: H.W) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def observe(x: H.W) -> List<U32>: [count(x)]
def apply(n: Nat) -> List<U32>: observe(value(n))
def main() -> List<List<U32>>:
  [apply(0n), apply(1n), apply(2n), apply(4n), apply(8n)]
