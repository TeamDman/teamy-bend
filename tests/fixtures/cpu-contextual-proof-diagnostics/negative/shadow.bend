# SPDX-License-Identifier: MPL-2.0
import Base
def bad(x: Nat) -> Nat:
  +x = 1n
  {==}
def main() -> Nat: bad(2n)
