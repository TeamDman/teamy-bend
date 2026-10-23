# SPDX-License-Identifier: MPL-2.0
import Base
def head(n: Nat) -> String:
  match n:
    case 0n: "zero"
    case 1n+p: "positive"
def reject(op: IO.OP<Unit>) -> Nat:
  match op:
    case Emit{x}: 0n
    case _: 0n
def invalid() -> Nat: reject(IO.print("SECRET")(Unit, x => Emit{x}))
def main() -> IO(Unit): IO.print(head(Nat.add(Succ{invalid()}, invalid())))
