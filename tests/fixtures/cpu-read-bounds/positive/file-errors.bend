# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/read_bounds.bend as R

def main() -> IO(Unit):
  do IO<Unit>:
    a : Result<&1, &1, U32 & String, File> <- File.open("", "r")
    Unit <- R.rc(a)
    b : Result<&1, &1, U32 & String, File> <- File.open("", "w")
    Unit <- R.rc(b)
    c : Result<&1, &1, U32 & String, File> <- File.open("", "a")
    Unit <- R.rc(c)
    d : Result<&1, &1, U32 & String, File> <- File.open("", "r+")
    R.rc(d)
