# SPDX-License-Identifier: MPL-2.0
import Base
def count(n: Nat) -> U32:
  match n:
    case 0n: 0
    case 1n+p: U32.add(1, count(p))
def main() -> U32: count(7n)
