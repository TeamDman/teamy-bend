# SPDX-License-Identifier: MPL-2.0
import Base
import ../imports/defs.bend as D
def bad(~f: Nat -> Nat, x: Nat) -> Nat -> Nat:
  f = Nat.add(x)
  f
def main() -> Nat: bad(~D.inc, 10n)(1n)
