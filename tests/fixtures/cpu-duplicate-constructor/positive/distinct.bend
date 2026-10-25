# SPDX-License-Identifier: MPL-2.0
import Base
type A is Data:
  One{value: U32}
type B is Data:
  Two{value: U32}
def add(a: A, b: B) -> U32:
  match a, b:
    case One{x}, Two{y}: (x + y : U32)
def main() -> U32: add(One{3}, Two{4})
