# SPDX-License-Identifier: MPL-2.0
import Base
import ../annotated_computed_scrutinee_000.bend as H
def token(n: Nat) -> H.Lst<H.Ch>:
  match n:
    case 0n: H.Nl{}
    case 1n: H.Cs{H.Ch{H.T{}}, H.Nl{}}
    case 2n: H.Cs{H.Ch{H.F{}}, H.Nl{}}
    case 3n: H.Cs{H.Ch{H.T{}}, H.Cs{H.Ch{H.T{}}, H.Nl{}}}
    case 4n: H.Cs{H.Ch{H.T{}}, H.Cs{H.Ch{H.F{}}, H.Nl{}}}
    case 5n: H.Cs{H.Ch{H.F{}}, H.Cs{H.Ch{H.T{}}, H.Nl{}}}
    case 6n+p: H.Cs{H.Ch{H.F{}}, H.Cs{H.Ch{H.F{}}, H.Nl{}}}
def value(a: Nat) -> H.W: H.main(token(a))
def count(n: H.W) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def observe(x: H.W) -> List<U32>: [count(x)]
def apply(a: Nat) -> List<U32>: observe(value(a))
def main() -> List<List<U32>>:
  [apply(0n), apply(1n), apply(2n), apply(3n), apply(4n), apply(5n), apply(6n)]
