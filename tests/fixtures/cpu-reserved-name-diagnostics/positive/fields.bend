# SPDX-License-Identifier: MPL-2.0
import Base
type Item is Data:
  Item{as: U32, match_value: U32}
def value(item: Item) -> U32:
  Item{as, match_value} = item
  (as + match_value : U32)
def main() -> U32: value(Item{3, 4})
