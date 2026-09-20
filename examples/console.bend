# SPDX-License-Identifier: MPL-2.0
import Base

def main() -> IO(U32):
  do IO<U32>:
    written : Unit <- IO.write("The answer is ")
    printed : Unit <- IO.print(U32.show((40 + 2 : U32)))
    return 7
