# SPDX-License-Identifier: MPL-2.0
import Base
import ../shared_scrutinee.bend as H
def make(n: Nat) -> H.N:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{make(p)}
def value(n: Nat) -> H.N: H.add(make(n), H.Z{})
def count(n: H.N) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def apply(n: Nat) -> List<U32>: [count(value(n))]
def main() -> List<List<U32>>: [apply(0n), apply(1n), apply(2n), apply(4n)]
