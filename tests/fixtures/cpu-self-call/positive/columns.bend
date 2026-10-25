# SPDX-License-Identifier: MPL-2.0
import Base
def go(+carry: Nat, n: Nat) -> Nat:
  match n:
    case Zero{}: carry
    case Succ{p}: go(carry, p)
def main() -> Nat: go(2n, 3n)
