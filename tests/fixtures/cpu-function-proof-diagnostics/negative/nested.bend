# SPDX-License-Identifier: MPL-2.0
import Base
def bad(n: Nat) -> Nat:
  match n:
    case 0n: 0n
    case 1n+p: x => p
def main() -> Nat: bad(2n)
