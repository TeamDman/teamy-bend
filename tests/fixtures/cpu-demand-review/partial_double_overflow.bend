# SPDX-License-Identifier: MPL-2.0
import Base
def half() -> Nat: Nat.mul(U32.to_nat(8388608), U32.to_nat(16777216))
def head(n: Nat) -> String:
  match n:
    case 0n: "zero"
    case 1n+p: "positive"
def main() -> IO(Unit): IO.print(head(Nat.double(half())))
