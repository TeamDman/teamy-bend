# SPDX-License-Identifier: MPL-2.0
import Base
def inc(n: Nat) -> Nat: 1n+n
def bad(~f: Nat -> Nat, x: Nat) -> Nat -> Nat:
  f = Nat.add(x)
  f
def good(~f: Nat -> Nat, x: Nat) -> Nat: f(x)
