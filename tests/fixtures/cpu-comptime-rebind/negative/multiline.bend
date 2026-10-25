# SPDX-License-Identifier: MPL-2.0
import Base
def inc(n: Nat) -> Nat: 1n+n
def bad(~f: Nat -> Nat, x: Nat) -> Nat -> Nat:
  f =
    Nat.add(x)
  f
def main() -> Nat: bad(~inc, 10n)(1n)
