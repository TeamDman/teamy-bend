# SPDX-License-Identifier: MPL-2.0
import Base
import ../literal_default_004.bend as H
def from_nat(n: Nat) -> H.W:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def count(n: H.W) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def value(n: Nat) -> H.R: H.main(from_nat(n))
def main() -> List<H.R>:
  [value(0n), value(1n), value(2n), value(4n)]
