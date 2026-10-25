# SPDX-License-Identifier: MPL-2.0
import Base
def pick(n: Nat) -> U32:
  match n:
    case 0n: 7
    case rest: 9
def main() -> U32: pick(0n)
