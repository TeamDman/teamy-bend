# SPDX-License-Identifier: MPL-2.0
import Base
def app(~f: Nat -> Nat, x: Nat) -> Nat:
  f(
    f(x)
  )
def main() -> Nat:
  app(~later, 3n)
def later(n: Nat) -> Nat: n
