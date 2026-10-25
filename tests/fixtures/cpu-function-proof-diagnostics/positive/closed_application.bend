# SPDX-License-Identifier: MPL-2.0
import Base
def Arrow() -> Type: Nat -> Nat
def main() -> Nat:
  f = {(x => Nat.add(x, 1n)) : Arrow}
  f(11n)
