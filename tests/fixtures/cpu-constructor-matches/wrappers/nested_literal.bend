# SPDX-License-Identifier: MPL-2.0
import Base
import ../nested_literal.bend as H
def from_nat(n: Nat) -> H.W:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def count(n: H.W) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def token(n: Nat) -> H.Token:
  match n:
    case 0n: H.Stop{}
    case 1n+p: H.Num{from_nat(p)}
def value(n: Nat, m: Nat) -> H.R: H.main(H.Env{token(n), token(m)})
def observe(x: H.R) -> List<U32>:
  match x:
    case H.K100{}: [0, 100]
    case H.KP{p}: [1, count(p)]
    case H.KK{p}: [2, count(p)]
    case H.K0{}: [3, 0]
def apply(n: Nat, m: Nat) -> List<U32>: observe(value(n, m))
def main() -> List<List<U32>>:
  [apply(0n, 0n), apply(0n, 1n), apply(0n, 2n), apply(0n, 4n), apply(1n, 0n), apply(1n, 1n), apply(1n, 2n), apply(1n, 4n), apply(2n, 0n), apply(2n, 1n), apply(2n, 2n), apply(2n, 4n), apply(4n, 0n), apply(4n, 1n), apply(4n, 2n), apply(4n, 4n)]
