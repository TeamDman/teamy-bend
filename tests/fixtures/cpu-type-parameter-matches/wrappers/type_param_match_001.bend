# SPDX-License-Identifier: MPL-2.0
import Base
import ../type_param_match_001.bend as H
def from_nat(n: Nat) -> H.Nat:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def value(k: Nat, b: Nat, c: Nat, d: Nat) -> H.R:
  match k:
    case 0n: H.main(H.Nat, from_nat(b), from_nat(c), from_nat(d))
    case 1n+p: H.main((H.Nat -> H.Nat), from_nat(b), from_nat(c), from_nat(d))
def count(n: H.Nat) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def observe(x: H.R) -> List<U32>:
  match x:
    case H.F{x, y}: [0, count(x), count(y)]
    case H.G{x, y, z}: [1, count(x), count(y), count(z)]
def apply(k: Nat, b: Nat, c: Nat, d: Nat) -> List<U32>: observe(value(k, b, c, d))
def main() -> List<List<U32>>:
  [apply(0n, 0n, 0n, 0n), apply(0n, 0n, 0n, 1n), apply(0n, 0n, 0n, 2n), apply(0n, 0n, 1n, 0n), apply(0n, 0n, 1n, 1n), apply(0n, 0n, 1n, 2n), apply(0n, 0n, 2n, 0n), apply(0n, 0n, 2n, 1n), apply(0n, 0n, 2n, 2n), apply(0n, 1n, 0n, 0n), apply(0n, 1n, 0n, 1n), apply(0n, 1n, 0n, 2n), apply(0n, 1n, 1n, 0n), apply(0n, 1n, 1n, 1n), apply(0n, 1n, 1n, 2n), apply(0n, 1n, 2n, 0n), apply(0n, 1n, 2n, 1n), apply(0n, 1n, 2n, 2n), apply(0n, 2n, 0n, 0n), apply(0n, 2n, 0n, 1n), apply(0n, 2n, 0n, 2n), apply(0n, 2n, 1n, 0n), apply(0n, 2n, 1n, 1n), apply(0n, 2n, 1n, 2n), apply(0n, 2n, 2n, 0n), apply(0n, 2n, 2n, 1n), apply(0n, 2n, 2n, 2n), apply(1n, 0n, 0n, 0n), apply(1n, 0n, 0n, 1n), apply(1n, 0n, 0n, 2n), apply(1n, 0n, 1n, 0n), apply(1n, 0n, 1n, 1n), apply(1n, 0n, 1n, 2n), apply(1n, 0n, 2n, 0n), apply(1n, 0n, 2n, 1n), apply(1n, 0n, 2n, 2n), apply(1n, 1n, 0n, 0n), apply(1n, 1n, 0n, 1n), apply(1n, 1n, 0n, 2n), apply(1n, 1n, 1n, 0n), apply(1n, 1n, 1n, 1n), apply(1n, 1n, 1n, 2n), apply(1n, 1n, 2n, 0n), apply(1n, 1n, 2n, 1n), apply(1n, 1n, 2n, 2n), apply(1n, 2n, 0n, 0n), apply(1n, 2n, 0n, 1n), apply(1n, 2n, 0n, 2n), apply(1n, 2n, 1n, 0n), apply(1n, 2n, 1n, 1n), apply(1n, 2n, 1n, 2n), apply(1n, 2n, 2n, 0n), apply(1n, 2n, 2n, 1n), apply(1n, 2n, 2n, 2n)]
