# SPDX-License-Identifier: MPL-2.0
import Base
def keep(n: Nat) -> Bool: Nat.is_le(n, 3n)
def main() -> List<&2, Nat>:
  List.sort(~Nat, ~Nat.is_le, List.filter(~Nat, ~keep, [5n,2n,1n,3n]))
