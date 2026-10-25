# SPDX-License-Identifier: MPL-2.0
import Base
def bad(+x: Nat) -> Nat:
  a = Nat.add(x, 1n)
  b = Bool.not(x)
  a
def main() -> Nat: bad(3n)
