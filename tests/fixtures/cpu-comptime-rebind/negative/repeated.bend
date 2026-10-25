# SPDX-License-Identifier: MPL-2.0
import Base
def inc(n: Nat) -> Nat: 1n+n
def bad(~f: Nat -> Nat, x: Nat) -> Nat -> Nat:
  g = f(x)
  f = Nat.add(g)
  f
def main() -> Nat: bad(~inc, 10n)(1n)
