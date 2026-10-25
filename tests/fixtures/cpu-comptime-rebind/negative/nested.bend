# SPDX-License-Identifier: MPL-2.0
import Base
type Wrap is Type: Wrap{f: Nat -> Nat}
def inc(n: Nat) -> Nat: 1n+n
def bad(~f: Nat -> Nat, x: Wrap) -> Nat:
  Wrap{f} = x
  f(1n)
def main() -> Nat: bad(~inc, Wrap{inc})
