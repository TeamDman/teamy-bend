# SPDX-License-Identifier: MPL-2.0
# Authored exact bits/text and shared valid/invalid read controls.
import Base

def observe(+value: F32) -> IO(Unit):
  IO.print(U32.show(F32.bits(value)) ++ " " ++ F32.show(value))

def read_bits(result: Maybe<&2, F32>) -> String:
  match result:
    case None{}: "none"
    case Some{value}: U32.show(F32.bits(value))

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- observe(F32.mod(7.5, 2.0))
    Unit <- observe(F32.neg(1.25))
    Unit <- observe(F32.pow(2.0, 10.0))
    Unit <- observe(F32.atan2(1.0, 1.0))
    Unit <- observe(F32.asin(1.0))
    Unit <- observe(F32.acos(1.0))
    Unit <- observe(F32.atan(1.0))
    Unit <- observe(U32.to_f32(7))
    Unit <- IO.print(U32.show(F32.to_u32(3.9)))
    Unit <- observe(F32.div(1.0, 3.0))
    Unit <- observe(0.0009765625)
    Unit <- IO.print(read_bits(F32.read("0.1")))
    Unit <- IO.print(read_bits(F32.read("-0")))
    Unit <- IO.print(read_bits(F32.read(".5")))
    Unit <- IO.print(read_bits(F32.read("1.5x")))
    Unit <- IO.print(read_bits(F32.read("")))
    Unit <- IO.print(read_bits(F32.read("1.5 ")))
    IO.print(read_bits(F32.read(F32.show(1.25))))
