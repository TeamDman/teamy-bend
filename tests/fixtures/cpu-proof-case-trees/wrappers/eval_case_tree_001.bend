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
def code(d: H.Dir) -> U32:
  match d:
    case H.Up{}: 0
    case H.Rt{}: 1
    case H.Dn{}: 2
    case H.Lf{}: 3
def apply(+n: Nat) -> List<U32>: [code(H.turn(make(n))), code(H.back(make(n)))]
def main() -> List<List<U32>>:
  [apply(0n), apply(1n), apply(2n), apply(3n)]
