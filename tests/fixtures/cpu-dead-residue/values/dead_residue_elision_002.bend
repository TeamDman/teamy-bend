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
def main() -> List<H.R>:
  [value(0n, 0n), value(0n, 1n), value(0n, 3n), value(1n, 0n), value(1n, 1n), value(1n, 3n), value(3n, 0n), value(3n, 1n), value(3n, 3n)]
