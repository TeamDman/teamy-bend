# SPDX-License-Identifier: MPL-2.0
import Base
def twice_tail(n: Nat) -> Nat:
  match n:
    case 0n:
      0n
    case 1n+ +k:
      Nat.add(k, k)
def main() -> Nat: twice_tail(3n)
