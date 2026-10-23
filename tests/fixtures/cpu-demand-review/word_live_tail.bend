# SPDX-License-Identifier: MPL-2.0
import Base
def reject(op: IO.OP<Unit>) -> Word(31n):
  match op:
    case Emit{x}: Word.zero(31n)
    case Halt{code, message}: Word.zero(31n)
def invalid() -> U32:
  U32{WCon{True{}, reject(IO.print("SECRET")(Unit, x => Emit{x}))}}
def low(value: U32) -> Bool:
  match value:
    case U32{word}:
      match word:
        case WCon{bit, tail}: bit
def main() -> IO(Unit): IO.print(Bool.show(low(U32.inc(invalid()))))
