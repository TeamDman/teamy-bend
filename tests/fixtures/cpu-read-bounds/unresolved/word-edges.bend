# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/read_bounds.bend as R

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(R.mu(U32.read("")))
    Unit <- IO.print(R.mu(U32.read("0")))
    Unit <- IO.print(R.mu(U32.read("000000000000000000004294967295")))
    Unit <- IO.print(R.mu(U32.read("4294967294")))
    Unit <- IO.print(R.mu(U32.read("4294967296")))
    Unit <- IO.print(R.mu(U32.read("42949672950")))
    Unit <- IO.print(R.mu(U32.read("9999999999")))
    Unit <- IO.print(R.mu(U32.read("+1")))
    Unit <- IO.print(R.mu(U32.read("-1")))
    Unit <- IO.print(R.mu(U32.read(" 1")))
    Unit <- IO.print(R.mu(U32.read("1 ")))
    Unit <- IO.print(R.mu(U32.read("1:")))
    Unit <- IO.print(R.mu(U32.read("é")))
    Unit <- IO.print(R.mu(U32.read("١")))
    Unit <- IO.print(R.mu(U32.read("1\0")))
    IO.print(R.mu(U32.read("0x10")))
