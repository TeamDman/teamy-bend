# SPDX-License-Identifier: MPL-2.0
import Base
import ../dead_residue_elision_002.bend as H
def from_nat(n: Nat) -> H.Nat:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def count(n: H.Nat) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def value(a: Nat, b: Nat) -> H.R: H.f(from_nat(a), from_nat(b))
def observe(x: H.R) -> List<U32>:
  match x:
    case H.A{a, b}: [0, count(a), count(b)]
    case H.B{b}: [1, count(b), 0]
    case H.C{c}: [2, count(c), 0]
def apply(a: Nat, b: Nat) -> List<U32>: observe(value(a, b))
def main() -> List<List<U32>>:
  [apply(0n, 0n), apply(0n, 1n), apply(0n, 3n), apply(1n, 0n), apply(1n, 1n), apply(1n, 3n), apply(3n, 0n), apply(3n, 1n), apply(3n, 3n)]
