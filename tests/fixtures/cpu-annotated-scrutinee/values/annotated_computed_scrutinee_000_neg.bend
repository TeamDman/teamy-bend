# SPDX-License-Identifier: MPL-2.0
import Base
import ../annotated_computed_scrutinee_000_neg.bend as H
def token(n: Nat) -> H.Bl:
  match n:
    case 0n: H.T{}
    case 1n+p: H.F{}
def value(a: Nat) -> H.Bl: H.main(token(a))
def main() -> List<H.Bl>:
  [value(0n), value(1n)]
