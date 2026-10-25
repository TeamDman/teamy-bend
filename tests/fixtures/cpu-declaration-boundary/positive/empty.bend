# SPDX-License-Identifier: MPL-2.0
import Base
type Nothing is Data:
law identity:
  for x: Unit
  {x == x : Unit}
def identity(x): {==}
def main() -> U32: 7
