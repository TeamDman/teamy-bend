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
def main() -> List<H.Bl>:
  [value(0n, 0n), value(0n, 1n), value(0n, 2n), value(0n, 3n), value(0n, 4n), value(0n, 5n), value(0n, 6n), value(1n, 0n), value(1n, 1n), value(1n, 2n), value(1n, 3n), value(1n, 4n), value(1n, 5n), value(1n, 6n), value(2n, 0n), value(2n, 1n), value(2n, 2n), value(2n, 3n), value(2n, 4n), value(2n, 5n), value(2n, 6n), value(3n, 0n), value(3n, 1n), value(3n, 2n), value(3n, 3n), value(3n, 4n), value(3n, 5n), value(3n, 6n), value(4n, 0n), value(4n, 1n), value(4n, 2n), value(4n, 3n), value(4n, 4n), value(4n, 5n), value(4n, 6n), value(5n, 0n), value(5n, 1n), value(5n, 2n), value(5n, 3n), value(5n, 4n), value(5n, 5n), value(5n, 6n), value(6n, 0n), value(6n, 1n), value(6n, 2n), value(6n, 3n), value(6n, 4n), value(6n, 5n), value(6n, 6n)]
