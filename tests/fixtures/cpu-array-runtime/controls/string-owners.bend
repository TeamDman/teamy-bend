# SPDX-License-Identifier: MPL-2.0
import Base

def value(r: Array<String> & String) -> String:
  (a, v) = r
  v

def finish(r: Array<String> & String, original: Array<String>, read: String) -> IO(Unit):
  (changed, old) = r
  do IO<Unit>:
    a : Unit <- IO.print(read)
    b : Unit <- IO.print(old)
    c : Unit <- IO.print(value(Array.get(String, changed, 0)))
    d : Unit <- IO.print(value(Array.get(String, original, 0)))
    return Unit{}

def read_then_swap(r: Array<String> & String, keep: Array<String>) -> IO(Unit):
  (change2, read) = r
  finish(Array.swap(String, change2, 4, "new"), keep, read)

def branch(r: Array<String> & Array<String>) -> IO(Unit):
  (keep, change) = r
  read_then_swap(Array.get(String, change, 0), keep)

def main() -> IO(Unit):
  branch(Array.clone(String, Array.new(String, 2n, String.append("ro", "ot"))))
