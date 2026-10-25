# SPDX-License-Identifier: MPL-2.0
import Base
def id(n: Nat) -> Nat: n
def inc(n: Nat) -> Nat: 1n+n
def twice(~f: Nat -> Nat, n: Nat) -> Nat:
  match n:
    case 0n: 0n
    case 1n++++k: Nat.add(f(k), f(k))
def main() -> List<Nat>:
  [twice(~id, 3n), twice(~inc, 3n), twice(~id, 4n)]
