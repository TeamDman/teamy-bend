# SPDX-License-Identifier: MPL-2.0
import Base
def cons(h: Nat, t: List<&2, Nat>) -> List<&2, Nat>: h <> t
def main() -> Nat & List<&2, Nat>:
  (List.foldl(~&2, ~Nat, ~Nat, ~Nat.add, [1n,2n,3n], 10n), List.foldr(~&2, ~Nat, ~List<&2, Nat>, ~cons, [1n,2n,3n], [9n]))
