# SPDX-License-Identifier: MPL-2.0
import Base
import ../dead_residue_elision_000.bend as H
def token(n: Nat) -> H.T:
  match n:
    case 0n: H.A{}
    case 1n+p:
      match p:
        case 0n: H.C{}
        case 1n+k: H.B{}
def value(n: Nat) -> H.R: H.main(token(n))
def main() -> List<H.R>:
  [value(0n), value(1n), value(2n)]
