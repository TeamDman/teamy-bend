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
def observe(x: H.R) -> List<U32>:
  match x:
    case H.XA{}: [0]
    case H.XC{}: [1]
    case H.XB{}: [2]
    case H.XD{w}: [3]
def apply(n: Nat) -> List<U32>: observe(value(n))
def main() -> List<List<U32>>:
  [apply(0n), apply(1n), apply(2n)]
