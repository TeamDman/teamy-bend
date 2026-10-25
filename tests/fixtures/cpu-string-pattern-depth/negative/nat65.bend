# SPDX-License-Identifier: MPL-2.0
import Base
def f(n: Nat) -> Bool:
  match n:
    case 65n: True{}
    case n: False{}
def main() -> Bool: f(65n)
