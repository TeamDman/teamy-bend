# SPDX-License-Identifier: MPL-2.0
# Authored finite arithmetic and conversion observations, without F32.read.
import Base

def observe(+value: F32) -> IO(Unit):
  IO.print(U32.show(F32.bits(value)) ++ " " ++ F32.show(value))

def fl(n: U32, d: U32) -> F32:
  F32.div(U32.to_f32(n), U32.to_f32(d))

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- observe((6.25 - 2.25 : F32))
    Unit <- observe((3.5 * 2.0 / 0.5 : F32))
    Unit <- observe(fl(15, 2))
    Unit <- observe(F32.mul(U32.to_f32(6), fl(15, 2)))
    Unit <- observe(F32.add(fl(1, 10), fl(2, 10)))
    Unit <- observe(fl(3, 10))
    Unit <- observe(F32.add(fl(16777216, 1), fl(1, 1)))
    Unit <- observe(U32.to_f32(16777217))
    Unit <- observe(U32.to_f32(16777219))
    Unit <- observe(fl(39, 10))
    Unit <- IO.print(U32.show(F32.to_u32(fl(39, 10))))
    Unit <- observe(U32.to_f32(123456))
    observe(F32.mul(F32.add(fl(1, 10), fl(2, 10)), fl(1000000, 1)))
