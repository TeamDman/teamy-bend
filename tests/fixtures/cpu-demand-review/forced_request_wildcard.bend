# SPDX-License-Identifier: MPL-2.0
# Isolate the wildcard-request decision by demanding its returned Nat.
import Base
def reject(op: IO.OP<Unit>) -> Nat:
  match op:
    case Emit{x}: 0n
    case _: 0n
def main() -> IO(Unit):
  IO.print(Nat.show(reject(IO.print("SECRET")(Unit, x => Emit{x}))))
