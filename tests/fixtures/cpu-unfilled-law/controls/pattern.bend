# SPDX-License-Identifier: MPL-2.0
import Base
type LawPair is Data: P{left: U32, right: U32}
law ghost: U32
def bad(pair: LawPair) -> U32:
  match pair:
    case P{left, right}: U32.add(left, ghost)
def main() -> U32: 0
