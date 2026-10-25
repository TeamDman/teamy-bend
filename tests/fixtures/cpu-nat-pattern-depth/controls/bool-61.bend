# SPDX-License-Identifier: MPL-2.0
import Base
def pick(n: Nat) -> Bool:
  match n:
    case 61n: True{}
    case rest: False{}
def main() -> Bool: pick(0n)
