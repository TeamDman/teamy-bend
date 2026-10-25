# SPDX-License-Identifier: MPL-2.0
import Base
def value() -> Nat:
  +n = Nat.mul(U32.to_nat(16777216), U32.to_nat(16777216))
  Nat.add(n, n)

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print("before")
    Unit <- IO.print(Nat.show(value()))
    IO.print("after")
