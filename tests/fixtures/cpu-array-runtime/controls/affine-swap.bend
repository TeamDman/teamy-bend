# SPDX-License-Identifier: MPL-2.0
import Base
type Cell is Type:
  Cell{function: Unit -> U32}
def call(c: Cell) -> U32:
  match c:
    case Cell{f}: f(Unit{})
def first(a: Array<Cell>) -> Cell:
  match a:
    case ALeaf{x}: x
    case ANode{left, right}: first(left)
def finish(r: Array<Cell> & Cell) -> IO(Unit):
  (a, old) = r
  do IO<Unit>:
    x : Unit <- IO.print(U32.show(call(old)))
    y : Unit <- IO.print(U32.show(call(first(a))))
    return Unit{}
def main() -> IO(Unit):
  finish(Array.swap(Cell, ALeaf{Cell{u => 7}}, 4294967295, Cell{u => 9}))
