# SPDX-License-Identifier: MPL-2.0
import Base
import ../dead_residue_elision_004.bend as H
def token(n: Nat) -> H.Sx:
  match n:
    case 0n: H.Ax{}
    case 1n+p: H.Bx{}
def from_nat(n: Nat) -> H.Cx:
  match n:
    case 0n: H.Zx9{}
    case 1n+p: H.Sx9{from_nat(p)}
def count(n: H.Cx) -> U32:
  match n:
    case H.Zx9{}: 0
    case H.Sx9{p}: (1 + count(p) : U32)
def value(a: Nat, b: Nat) -> H.Cx: H.f(token(a), from_nat(b))
def main() -> List<H.Cx>:
  [value(0n, 0n), value(0n, 1n), value(0n, 2n), value(0n, 4n), value(1n, 0n), value(1n, 1n), value(1n, 2n), value(1n, 4n)]
