# SPDX-License-Identifier: MPL-2.0
import Base
def value() -> U32:
  +n = {4294967295 : U32}
  U32.add(n, n)

def main() -> IO(Unit): IO.print(U32.show(value()))
