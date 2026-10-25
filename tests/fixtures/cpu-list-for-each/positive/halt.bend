# SPDX-License-Identifier: MPL-2.0
import Base
def visit(n: U32) -> IO(Unit):
  match n:
    case 1: IO.print("first")
    case 3: IO.print("unreachable")
    case _: IO.die(Unit, 7, "stop")
def main() -> IO(Unit): List.for_each(~&2, ~U32, ~visit, [1,2,3])
