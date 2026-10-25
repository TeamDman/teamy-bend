# SPDX-License-Identifier: MPL-2.0
import Base
def value() -> U32:
  +n = {3 : U32}
  a = U32.add(n, n)
  U32.add(a, n)

def main() -> IO(Unit): IO.print(U32.show(value()))
