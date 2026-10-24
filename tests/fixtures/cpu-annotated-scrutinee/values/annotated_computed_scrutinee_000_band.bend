# SPDX-License-Identifier: MPL-2.0
import Base
import ../annotated_computed_scrutinee_000_band.bend as H
def token(n: Nat) -> H.Bl:
  match n:
    case 0n: H.T{}
    case 1n+p: H.F{}
def value(a: Nat, b: Nat) -> H.Bl: H.main(token(a), token(b))
def main() -> List<H.Bl>:
  [value(0n, 0n), value(0n, 1n), value(1n, 0n), value(1n, 1n)]
