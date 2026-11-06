# SPDX-License-Identifier: MPL-2.0
import Base

type Three is Data: Three{a: U32, b: U32, c: U32}
def discard(array: Array<Three>) -> U32: 0
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("before")
    IO.print(U32.show(discard(Array.new(Three, 30n, Three{1, 2, 3}))))
