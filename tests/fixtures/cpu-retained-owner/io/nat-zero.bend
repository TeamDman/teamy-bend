# SPDX-License-Identifier: MPL-2.0
import Base
def value() -> Nat:
  +n = {0n : Nat}
  Nat.add(n, n)

def main() -> IO(Unit): IO.print(Nat.show(value()))
