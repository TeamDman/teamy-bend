# SPDX-License-Identifier: MPL-2.0
import Base
def limit() -> Nat: Nat.mul(U32.to_nat(16777215), U32.to_nat(16777217))
def overflow() -> Nat: Nat.add(limit(), 1n)
def reject(op: IO.OP<Unit>) -> Nat:
  match op:
    case Emit{x}: 0n
    case Halt{code, message}: 0n
def invalid() -> Nat: reject(IO.print("SECRET")(Unit, x => Emit{x}))
def pair(a: Nat, b: Nat) -> String: "kept"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("before")
    n : Nat <- IO.pure(Nat, overflow())
    Unit <- IO.print(pair(n, invalid()))
    IO.print("after")
