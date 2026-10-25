# SPDX-License-Identifier: MPL-2.0
import Base
type Cell is Data: Cell{a: U32, b: U32}
def read(c: Cell) -> U32:
  match c:
    case Cell{a,b}: (a + b : U32)
def main() -> U32: read(Cell{2,3})
