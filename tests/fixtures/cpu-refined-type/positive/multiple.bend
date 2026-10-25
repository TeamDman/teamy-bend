# SPDX-License-Identifier: MPL-2.0
import Base
type T is Data:
  A{}
  B{}
  C{value: U32}
def loop(t: T) -> U32:
  match t:
    case A{}: 0
    case B{}: 1
    case C{value}: value
def main() -> U32: loop(C{7})
