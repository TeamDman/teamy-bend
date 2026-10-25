# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/read_bounds.bend as R

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(R.mn(Nat.read("")))
    Unit <- IO.print(R.mn(Nat.read("0")))
    Unit <- IO.print(R.mn(Nat.read("000")))
    Unit <- IO.print(R.mn(Nat.read("00000000000000000281474976710655")))
    Unit <- IO.print(R.mn(Nat.read("281474976710654")))
    Unit <- IO.print(R.mn(Nat.read("281474976710656")))
    Unit <- IO.print(R.mn(Nat.read("999999999999999")))
    Unit <- IO.print(R.mn(Nat.read("+1")))
    Unit <- IO.print(R.mn(Nat.read("-1")))
    Unit <- IO.print(R.mn(Nat.read(" 1")))
    Unit <- IO.print(R.mn(Nat.read("1 ")))
    Unit <- IO.print(R.mn(Nat.read("\t1")))
    Unit <- IO.print(R.mn(Nat.read("١")))
    Unit <- IO.print(R.mn(Nat.read("é")))
    Unit <- IO.print(R.mn(Nat.read("1\0")))
    IO.print(R.mn(Nat.read("0x10")))
