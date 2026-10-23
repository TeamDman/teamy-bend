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
type Box is Data:
  Box{-tag: Nat, value: Nat}
def erased(-n: Nat) -> IO(Unit): IO.print("erased")
def open(box: Box) -> IO(Unit):
  match box:
    case Box{tag, value}: IO.print(Nat.show(value))
def choose(b: Bool) -> IO(Unit):
  match b:
    case True{}: IO.print("selected")
    case False{}: IO.print(Nat.show(overflow()))
def ignore(f: Nat -> Nat, action: IO(Unit)) -> IO(Unit): IO.print("unused closures")
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- erased(overflow())
    Unit <- erased(invalid())
    Unit <- open(Box{overflow(), 7n})
    Unit <- choose(True{})
    ignore(n => overflow(), IO.print("SECRET"))
