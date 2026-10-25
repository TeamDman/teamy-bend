# SPDX-License-Identifier: MPL-2.0
import Base

def copied(a: U32) -> U32:
  match a:
    case U32{word}: U32{word}
def apply(f: U32 -> U32, a: U32) -> U32: f(a)
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(U32.show(U32.sub(0, 1)))
    Unit <- IO.print(U32.show(U32.sub(4294967295, 4294967295)))
    Unit <- IO.print(U32.show(U32.sub(4294967295, 0)))
    Unit <- IO.print(U32.show(U32.sub(7, 3)))
    Unit <- IO.print(U32.show(U32.sub(2147483648, 1)))
    Unit <- IO.print(U32.show(U32.sub(0, 2147483648)))
    Unit <- IO.print(U32.show(U32.sub(copied(3), 5)))
    Unit <- IO.print(U32.show(U32.sub(3, copied(5))))
    Unit <- IO.print(U32.show(U32.sub(copied(4294967295), copied(4294967295))))
    IO.print(U32.show(apply(U32.sub(7), 9)))
