# SPDX-License-Identifier: MPL-2.0
import Base
import ../nested_computed_match_001_dec.bend as H
def token(n: Nat) -> H.Bl:
  match n:
    case 0n: H.T{}
    case 1n+p: H.F{}
def value(n: Nat) -> H.R: H.main(token(n))
def main() -> List<H.R>:
  [value(0n), value(1n)]
