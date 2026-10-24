# SPDX-License-Identifier: MPL-2.0
import Base
import ../annotated_computed_scrutinee_000_str_eq.bend as H
def token(n: Nat) -> H.Lst<H.Ch>:
  match n:
    case 0n: H.Nl{}
    case 1n: H.Cs{H.Ch{H.T{}}, H.Nl{}}
    case 2n: H.Cs{H.Ch{H.F{}}, H.Nl{}}
    case 3n: H.Cs{H.Ch{H.T{}}, H.Cs{H.Ch{H.T{}}, H.Nl{}}}
    case 4n: H.Cs{H.Ch{H.T{}}, H.Cs{H.Ch{H.F{}}, H.Nl{}}}
    case 5n: H.Cs{H.Ch{H.F{}}, H.Cs{H.Ch{H.T{}}, H.Nl{}}}
    case 6n+p: H.Cs{H.Ch{H.F{}}, H.Cs{H.Ch{H.F{}}, H.Nl{}}}
def value(a: Nat, b: Nat) -> H.Bl: H.main(token(a), token(b))
def observe(x: H.Bl) -> List<U32>:
  match x:
    case H.T{}: [1]
    case H.F{}: [0]
def apply(a: Nat, b: Nat) -> List<U32>: observe(value(a, b))
def main() -> List<List<U32>>:
  [apply(0n, 0n), apply(0n, 1n), apply(0n, 2n), apply(0n, 3n), apply(0n, 4n), apply(0n, 5n), apply(0n, 6n), apply(1n, 0n), apply(1n, 1n), apply(1n, 2n), apply(1n, 3n), apply(1n, 4n), apply(1n, 5n), apply(1n, 6n), apply(2n, 0n), apply(2n, 1n), apply(2n, 2n), apply(2n, 3n), apply(2n, 4n), apply(2n, 5n), apply(2n, 6n), apply(3n, 0n), apply(3n, 1n), apply(3n, 2n), apply(3n, 3n), apply(3n, 4n), apply(3n, 5n), apply(3n, 6n), apply(4n, 0n), apply(4n, 1n), apply(4n, 2n), apply(4n, 3n), apply(4n, 4n), apply(4n, 5n), apply(4n, 6n), apply(5n, 0n), apply(5n, 1n), apply(5n, 2n), apply(5n, 3n), apply(5n, 4n), apply(5n, 5n), apply(5n, 6n), apply(6n, 0n), apply(6n, 1n), apply(6n, 2n), apply(6n, 3n), apply(6n, 4n), apply(6n, 5n), apply(6n, 6n)]
