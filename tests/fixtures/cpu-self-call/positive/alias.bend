# SPDX-License-Identifier: MPL-2.0
import Base
def go(n: Nat) -> Nat:
  match n:
    case Zero{}: 7n
    case Succ{p}:
      +q = {p : Nat}
      go(q)
def main() -> Nat: go(3n)
