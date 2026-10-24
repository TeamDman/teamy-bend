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
def main() -> List<H.W>:
  [value(0n), value(1n), value(2n), value(3n), value(4n), value(5n), value(6n)]
