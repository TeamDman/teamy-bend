# SPDX-License-Identifier: MPL-2.0
import Base
def value() -> F32:
  +n = {1.5 : F32}
  F32.add(n, n)

def main() -> IO(Unit): IO.print(F32.show(value()))
