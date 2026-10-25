# SPDX-License-Identifier: MPL-2.0
import Base
def copy(n: Nat, word: Word(n)) -> Word(n):
  match n:
    case Zero{}: WNil{}
    case Succ{p}:
      match word:
        case WCon{bit, tail}: WCon{bit, copy(p, tail)}

def copied(value: U32) -> U32:
  match value:
    case U32{word}: U32{copy(32n, word)}

def make(value: U32) -> F32:
  match value:
    case U32{word}: F32{copy(32n, word)}

def low(value: U32) -> Bool:
  match value:
    case U32{word}:
      match word:
        case WCon{bit, tail}: bit

def apply(f: U32 -> U32, value: U32) -> U32: f(value)

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(U32.show(U32.shr(0)))
    Unit <- IO.print(U32.show(U32.shr(2147483649)))
    Unit <- IO.print(U32.show(U32.shr(4294967295)))
    Unit <- IO.print(U32.show(U32.shrn(4294967295, 31n)))
    Unit <- IO.print(U32.show(U32.shrn(4294967295, 32n)))
    Unit <- IO.print(U32.show(U32.shrn(4294967295, 33n)))
    Unit <- IO.print(U32.show(U32.and(4294967295, 0)))
    Unit <- IO.print(U32.show(U32.and(2863311530, 1431655765)))
    Unit <- IO.print(U32.show(U32.and(4294967295, 2147483649)))
    Unit <- IO.print(U32.show(U32.shr(copied(4294967295))))
    Unit <- IO.print(U32.show(U32.and(copied(2147483649), copied(4294967295))))
    IO.print(U32.show(apply(U32.and(4294967295), 2147483649)))
