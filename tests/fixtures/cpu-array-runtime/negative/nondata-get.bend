# SPDX-License-Identifier: MPL-2.0
import Base
type Cell is Type:
  Cell{function: Unit -> U32}
def main() -> Array<Cell> & Cell:
  Array.get(Cell, ALeaf{Cell{u => 7}}, 0)
