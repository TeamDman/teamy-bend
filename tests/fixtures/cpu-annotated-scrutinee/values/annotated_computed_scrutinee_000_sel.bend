# SPDX-License-Identifier: MPL-2.0
import Base
import ../annotated_computed_scrutinee_000_sel.bend as H
def from_nat(n: Nat) -> H.W:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def value(a: Nat) -> H.W: H.main(from_nat(a))
def main() -> List<H.W>:
  [value(0n), value(1n), value(2n), value(4n)]
