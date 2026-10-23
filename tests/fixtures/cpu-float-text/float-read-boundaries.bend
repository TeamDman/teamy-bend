# SPDX-License-Identifier: MPL-2.0
# Authored contract probe; upstream C/JS agreement is not assumed.
import Base

def read_bits(result: Maybe<&2, F32>) -> String:
  match result:
    case None{}: "none"
    case Some{value}: U32.show(F32.bits(value))

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print("decimal " ++ read_bits(F32.read("1.0000000596046448")))
    Unit <- IO.print("hex " ++ read_bits(F32.read("0x1.8p+1")))
    Unit <- IO.print("newline " ++ read_bits(F32.read("1.5\n")))
    IO.print("nul " ++ read_bits(F32.read("1.5\0tail")))
