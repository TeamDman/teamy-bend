# SPDX-License-Identifier: MPL-2.0
import Base

law add_zero:
  for n: Nat
  {Nat.add(n, 0n) == n : Nat}

def add_zero(n):
  match n:
    case Zero{}:
      {==}
    case Succ{p}:
      %add_zero(p): {Succ{Nat.add(p, 0n)} == Succ{_} : Nat}
      {==}

def main() -> Nat:
  Nat.add(2n, 3n)
