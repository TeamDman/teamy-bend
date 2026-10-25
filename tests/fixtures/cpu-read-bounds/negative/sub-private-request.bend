# SPDX-License-Identifier: MPL-2.0
import Base

def reject(op: IO.OP<Unit>) -> U32:
  match op:
    case Emit{value}: 0
    case Halt{code, message}: code
def invalid() -> U32:
  reject(IO.print("must not execute")(Unit, value => Emit{value}))
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print("before")
    Unit <- IO.print(U32.show(U32.sub(0, invalid())))
    IO.print("after")
