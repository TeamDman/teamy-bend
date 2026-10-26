# SPDX-License-Identifier: MPL-2.0
import Base

def finish(r: Array<F32> & F32, old: F32, observed: F32) -> IO(Unit):
  (original2, signed_zero) = r
  do IO<Unit>:
    a : Unit <- IO.print(U32.show(F32.bits(old)))
    b : Unit <- IO.print(U32.show(F32.bits(observed)))
    c : Unit <- IO.print(U32.show(F32.bits(signed_zero)))
    return Unit{}

def read_changed(r: Array<F32> & F32, original: Array<F32>, old: F32) -> IO(Unit):
  (changed2, observed) = r
  finish(Array.get(F32, original, 1), old, observed)

def swapped(r: Array<F32> & F32, original: Array<F32>) -> IO(Unit):
  (changed, old) = r
  read_changed(Array.get(F32, changed, 1), original, old)

def report(r: Array<F32> & Array<F32>) -> IO(Unit):
  (original, copy) = r
  swapped(Array.swap(F32, copy, 3, 1.5), original)

def main() -> IO(Unit):
  report(Array.clone(F32, Array.new(F32, 1n, F32.neg(0.0))))
