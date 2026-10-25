# SPDX-License-Identifier: MPL-2.0
import Base
def visit(n: U32) -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(0)
    IO.print(U32.show(n))
def row(xs: List<&2, U32>) -> IO(Unit): List.for_each(~&2, ~U32, ~visit, xs)
def main() -> IO(Unit): List.for_each(~&2, ~List<&2, U32>, ~row, [[1,2],[],[3]])
