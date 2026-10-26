# SPDX-License-Identifier: MPL-2.0
import Base

def render(a: Array<String>) -> String:
  match a:
    case ALeaf{x}: "[" ++ x ++ "]"
    case ANode{left, right}: render(left) ++ render(right)

def discard(a: Array<String>, keep: Array<String>) -> Array<String>:
  keep

def after_read(r: Array<String> & String, assigned: Array<String>, old: String, i: U32) -> IO(Unit):
  (retained, read) = r
  other_changed = Array.set(String, retained, i, String.append("ke", "ep"))
  do IO<Unit>:
    a : Unit <- IO.print("old:" ++ old)
    b : Unit <- IO.print("read:" ++ read)
    c : Unit <- IO.print("first:" ++ render(assigned))
    d : Unit <- IO.print("second:" ++ render(other_changed))
    return Unit{}

def after_swap(r: Array<String> & String, kept: Array<String>, +i: U32) -> IO(Unit):
  (changed, old) = r
  assigned = Array.set(String, changed, 0, String.append("s", "et"))
  after_read(Array.get(String, kept, i), assigned, old, i)

def after_unread(r: Array<String> & Array<String>, right: Array<String>, +i: U32) -> IO(Unit):
  (kept, unread) = r
  kept_after_drop = discard(unread, kept)
  after_swap(Array.swap(String, right, i, String.append("n", "ew")), kept_after_drop, i)

def after_clone(r: Array<String> & Array<String>, i: U32) -> IO(Unit):
  (left, right) = r
  after_unread(Array.clone(String, left), right, i)

def observe(a: Array<String>, i: U32) -> IO(Unit):
  after_clone(Array.clone(String, a), i)

def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- observe(Array.new(String, 0n, String.append("le", "af")), 0)
    b : Unit <- observe(Array.new(String, 2n, String.append("bo", "x")), 3)
    return Unit{}
