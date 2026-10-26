# SPDX-License-Identifier: MPL-2.0
import Base

def render(a: Array<U32>) -> String:
  match a:
    case ALeaf{x}: "[" ++ U32.show(x) ++ "]"
    case ANode{left, right}: render(left) ++ render(right)

def finish(r: Array<U32> & U32, kept: Array<U32>, low: U32, high: U32, seam: U32) -> IO(Unit):
  (rebuilt, next) = r
  do IO<Unit>:
    a : Unit <- IO.print("old:" ++ U32.show(low) ++ "," ++ U32.show(high))
    b : Unit <- IO.print("seam:" ++ U32.show(seam) ++ "," ++ U32.show(next))
    c : Unit <- IO.print("rebuilt:" ++ render(rebuilt))
    d : Unit <- IO.print("kept:" ++ render(kept))
    return Unit{}

def after_read(r: Array<U32> & U32, kept: Array<U32>, low: U32, high: U32, half: U32) -> IO(Unit):
  (rebuilt, seam) = r
  finish(Array.get(U32, rebuilt, half), kept, low, high, seam)

def after_high(r: Array<U32> & U32, left: Array<U32>, kept: Array<U32>, low: U32, +half: U32) -> IO(Unit):
  (right, high) = r
  after_read(Array.get(U32, ANode{left, right}, U32.sub(half, 1)), kept, low, high, half)

def after_low(r: Array<U32> & U32, right: Array<U32>, kept: Array<U32>, +half: U32) -> IO(Unit):
  (left, low) = r
  after_high(Array.swap(U32, right, half, 80), left, kept, low, half)

def split(a: Array<U32>, kept: Array<U32>, half: U32) -> IO(Unit):
  match a:
    case ALeaf{x}: IO.print("unexpected leaf:" ++ U32.show(x) ++ render(kept))
    case ANode{left, right}: after_low(Array.swap(U32, left, 4294967295, 90), right, kept, half)

def after_clone(r: Array<U32> & Array<U32>, half: U32) -> IO(Unit):
  (changed, kept) = r
  split(changed, kept, half)

def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- after_clone(Array.clone(U32, ANode{ALeaf{1}, ALeaf{2}}), 1)
    b : Unit <- after_clone(Array.clone(U32, ANode{ANode{ANode{ALeaf{1}, ALeaf{2}}, ANode{ALeaf{3}, ALeaf{4}}}, ANode{ANode{ALeaf{5}, ALeaf{6}}, ANode{ALeaf{7}, ALeaf{8}}}}), 4)
    return Unit{}
