# SPDX-License-Identifier: MPL-2.0
import Base
import ../../cpu-duplicate-declaration/exploratory/alias/main.bend as A
def main() -> IO(Unit):
  IO.print(U32.show(A.main()))
