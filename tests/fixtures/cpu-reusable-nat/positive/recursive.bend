# SPDX-License-Identifier: MPL-2.0
import Base
def copies(n: Nat) -> Nat:
  match n:
    case 0n: 1n
    case 1n++k: Nat.add(copies(k), copies(k))
def main() -> Nat: copies(4n)
