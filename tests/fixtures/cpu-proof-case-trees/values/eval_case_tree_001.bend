# SPDX-License-Identifier: MPL-2.0
import Base
import ../eval_case_tree_001.bend as H
def make(n: Nat) -> H.Dir:
  match n:
    case 0n: H.Up{}
    case 1n+a:
      match a:
        case 0n: H.Rt{}
        case 1n+b:
          match b:
            case 0n: H.Dn{}
            case 1n+c: H.Lf{}
def value(+n: Nat) -> List<H.Dir>: [H.turn(make(n)), H.back(make(n))]
def main() -> List<List<H.Dir>>:
  [value(0n), value(1n), value(2n), value(3n)]
