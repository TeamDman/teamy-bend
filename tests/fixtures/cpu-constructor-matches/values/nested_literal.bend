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
def main() -> List<H.R>:
  [value(0n, 0n), value(0n, 1n), value(0n, 2n), value(0n, 4n), value(1n, 0n), value(1n, 1n), value(1n, 2n), value(1n, 4n), value(2n, 0n), value(2n, 1n), value(2n, 2n), value(2n, 4n), value(4n, 0n), value(4n, 1n), value(4n, 2n), value(4n, 4n)]
