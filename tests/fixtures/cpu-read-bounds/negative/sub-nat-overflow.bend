# SPDX-License-Identifier: MPL-2.0
import Base

def invalid() -> U32:
  U32.from_nat(Nat.mul(U32.to_nat(16777216), U32.to_nat(16777216)))
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print("before")
    Unit <- IO.print(U32.show(U32.sub(invalid(), 0)))
    IO.print("after")
