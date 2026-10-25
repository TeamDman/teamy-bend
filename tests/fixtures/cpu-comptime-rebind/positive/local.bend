# SPDX-License-Identifier: MPL-2.0
import Base
def inc(n: Nat) -> Nat: 1n+n
def good(~f: Nat -> Nat, x: Nat) -> Nat -> Nat:
  g = Nat.add(f(x))
  g
def main() -> Nat: good(~inc, 10n)(1n)
