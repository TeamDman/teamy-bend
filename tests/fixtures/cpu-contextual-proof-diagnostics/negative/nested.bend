# SPDX-License-Identifier: MPL-2.0
import Base
def bad(x: Nat) -> Nat:
  match x:
    case 0n: 0n
    case 1n+p: {==}
def main() -> Nat: bad(2n)
