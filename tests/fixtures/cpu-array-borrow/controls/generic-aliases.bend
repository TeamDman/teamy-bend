# SPDX-License-Identifier: MPL-2.0
import Base

type Cell<-A: Data> is Data:
  BorrowLeft{value: A, words: List<&2, U32>}
  BorrowRight{words: List<&2, U32>, value: A}

def kept(-A: Data, r: Array<Cell<A>> & Cell<A>, read: Cell<A>, old: Cell<A>, current: Cell<A>) ->
  Cell<A> & Cell<A> & Cell<A> & Cell<A>:
  (a, original) = r
  (read, old, current, original)

def current(-A: Data, r: Array<Cell<A>> & Cell<A>, original: Array<Cell<A>>, read: Cell<A>, old: Cell<A>) ->
  Cell<A> & Cell<A> & Cell<A> & Cell<A>:
  (a, changed) = r
  kept(A, Array.get(Cell<A>, original, 0), read, old, changed)

def swapped(-A: Data, r: Array<Cell<A>> & Cell<A>, original: Array<Cell<A>>, read: Cell<A>) ->
  Cell<A> & Cell<A> & Cell<A> & Cell<A>:
  (a, old) = r
  current(A, Array.get(Cell<A>, a, 0), original, read, old)

def cloned(-A: Data, r: Array<Cell<A>> & Array<Cell<A>>, replacement: Cell<A>, read: Cell<A>) ->
  Cell<A> & Cell<A> & Cell<A> & Cell<A>:
  (original, changed) = r
  swapped(A, Array.swap(Cell<A>, changed, 0, replacement), original, read)

def read_then_clone(-A: Data, r: Array<Cell<A>> & Cell<A>, replacement: Cell<A>) ->
  Cell<A> & Cell<A> & Cell<A> & Cell<A>:
  (a, read) = r
  cloned(A, Array.clone(Cell<A>, a), replacement, read)

def observe(-A: Data, a: Array<Cell<A>>, replacement: Cell<A>) ->
  Cell<A> & Cell<A> & Cell<A> & Cell<A>:
  read_then_clone(A, Array.get(Cell<A>, a, 0), replacement)

def show_num(c: Cell<U32>) -> String:
  match c:
    case BorrowLeft{v, words}:
      "L:" ++ U32.show(v) ++ ":" ++ List.show(~&2, ~U32, ~U32.show, words)
    case BorrowRight{words, v}:
      "R:" ++ U32.show(v) ++ ":" ++ List.show(~&2, ~U32, ~U32.show, words)

def show_text(c: Cell<String>) -> String:
  match c:
    case BorrowLeft{v, words}:
      "L:" ++ v ++ ":" ++ List.show(~&2, ~U32, ~U32.show, words)
    case BorrowRight{words, v}:
      "R:" ++ v ++ ":" ++ List.show(~&2, ~U32, ~U32.show, words)

def emit_num(r: Cell<U32> & Cell<U32> & Cell<U32> & Cell<U32>) -> IO(Unit):
  (read, old, changed, original) = r
  do IO<Unit>:
    a : Unit <- IO.print(show_num(read))
    b : Unit <- IO.print(show_num(old))
    c : Unit <- IO.print(show_num(changed))
    d : Unit <- IO.print(show_num(original))
    return Unit{}

def emit_text(r: Cell<String> & Cell<String> & Cell<String> & Cell<String>) -> IO(Unit):
  (read, old, changed, original) = r
  do IO<Unit>:
    a : Unit <- IO.print(show_text(read))
    b : Unit <- IO.print(show_text(old))
    c : Unit <- IO.print(show_text(changed))
    d : Unit <- IO.print(show_text(original))
    return Unit{}

def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- emit_num(observe(U32, ALeaf{BorrowLeft{5, [10, 20]}}, BorrowRight{[40, 50], 7}))
    b : Unit <- emit_num(observe(U32, ALeaf{BorrowRight{[3, 4], 9}}, BorrowLeft{11, [6]}))
    c : Unit <- emit_text(observe(String, ALeaf{BorrowLeft{String.append("o", "ld"), [1, 2]}}, BorrowRight{[8], String.append("n", "ew")}))
    d : Unit <- emit_text(observe(String, ALeaf{BorrowRight{[4, 5], String.append("ke", "ep")}}, BorrowLeft{String.append("sw", "ap"), [9, 10]}))
    return Unit{}
