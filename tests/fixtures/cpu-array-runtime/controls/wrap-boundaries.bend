# SPDX-License-Identifier: MPL-2.0
import Base

def sample() -> Array<U32>:
  ANode{ANode{ALeaf{10}, ALeaf{20}}, ANode{ALeaf{30}, ALeaf{40}}}

def value(r: Array<U32> & U32) -> U32:
  (a, v) = r
  v

def changed(r: Array<U32> & U32) -> U32:
  (a, old) = r
  U32.add(old, value(Array.get(U32, a, 3)))

def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- IO.print(U32.show(value(Array.get(U32, sample(), 0))))
    b : Unit <- IO.print(U32.show(value(Array.get(U32, sample(), 4))))
    c : Unit <- IO.print(U32.show(value(Array.get(U32, sample(), 4294967295))))
    d : Unit <- IO.print(U32.show(changed(Array.swap(U32, sample(), 4294967295, 99))))
    e : Unit <- IO.print(U32.show(value(Array.get(U32, Array.new(U32, 0n, 7), 4294967295))))
    f : Unit <- IO.print(U32.show(value(Array.size(U32, Array.new(U32, 0n, 9)))))
    return Unit{}
