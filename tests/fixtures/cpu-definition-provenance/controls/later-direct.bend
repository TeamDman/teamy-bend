# SPDX-License-Identifier: MPL-2.0
import Base
def keep(~f: Nat -> Nat) -> Nat -> Nat:
  f
def main() -> Nat:
  keep(~later)(3n)
def later(n: Nat) -> Nat: n
