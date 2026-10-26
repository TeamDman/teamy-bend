# SPDX-License-Identifier: MPL-2.0
import Base

type Cell is Data:
  OwnedText{value: String}
  Vacant{}

def show(c: Cell) -> String:
  match c:
    case OwnedText{x}: "text:" ++ x
    case Vacant{}: "empty"

def render(a: Array<Cell>) -> String:
  match a:
    case ALeaf{x}: "[" ++ show(x) ++ "]"
    case ANode{left, right}: render(left) ++ render(right)

def finish(r: Array<Cell> & Cell, old: Cell, empty: Cell, filled: Array<Cell>, snapshot: Array<Cell>) -> IO(Unit):
  (kept, read) = r
  do IO<Unit>:
    a : Unit <- IO.print("old:" ++ show(old))
    b : Unit <- IO.print("cleared:" ++ show(empty))
    c : Unit <- IO.print("read:" ++ show(read))
    d : Unit <- IO.print("filled:" ++ render(filled))
    e : Unit <- IO.print("snapshot:" ++ render(snapshot))
    f : Unit <- IO.print("original:" ++ render(kept))
    return Unit{}

def after_empty_clone(r: Array<Cell> & Array<Cell>, original: Array<Cell>, old: Cell, empty: Cell, +i: U32) -> IO(Unit):
  (snapshot, to_fill) = r
  filled = Array.set(Cell, to_fill, i, OwnedText{String.append("fi", "nal")})
  finish(Array.get(Cell, original, i), old, empty, filled, snapshot)

def after_empty_read(r: Array<Cell> & Cell, original: Array<Cell>, old: Cell, i: U32) -> IO(Unit):
  (after_read, empty) = r
  after_empty_clone(Array.clone(Cell, after_read), original, old, empty, i)

def after_swap(r: Array<Cell> & Cell, original: Array<Cell>, +i: U32) -> IO(Unit):
  (swapped, old) = r
  cleared = Array.set(Cell, swapped, i, Vacant{})
  after_empty_read(Array.get(Cell, cleared, i), original, old, i)

def after_clone(r: Array<Cell> & Array<Cell>, +i: U32) -> IO(Unit):
  (original, changed) = r
  after_swap(Array.swap(Cell, changed, i, OwnedText{String.append("sw", "ap")}), original, i)

def observe(a: Array<Cell>, i: U32) -> IO(Unit):
  after_clone(Array.clone(Cell, a), i)

def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- observe(ALeaf{OwnedText{String.append("o", "ld")}}, 0)
    b : Unit <- observe(ANode{ANode{ALeaf{OwnedText{String.append("l", "eft")}}, ALeaf{Vacant{}}}, ANode{ALeaf{OwnedText{String.append("ri", "ght")}}, ALeaf{Vacant{}}}}, 3)
    return Unit{}
