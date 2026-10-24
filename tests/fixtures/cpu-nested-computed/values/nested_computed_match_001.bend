# SPDX-License-Identifier: MPL-2.0
import Base
import ../nested_computed_match_001.bend as H
def from_nat(n: Nat) -> H.Nt:
  match n:
    case 0n: H.Zn{}
    case 1n+p: H.Sn{from_nat(p)}
def value(n: Nat) -> H.R: H.main(from_nat(n))
def main() -> List<H.R>:
  [value(0n), value(1n), value(2n), value(3n), value(5n)]
