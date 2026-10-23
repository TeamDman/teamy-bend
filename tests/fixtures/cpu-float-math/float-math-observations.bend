# SPDX-License-Identifier: MPL-2.0
# Authored bit/text observations for the pinned unary, trig and hyperbolic cases.
import Base

def observe(+value: F32) -> IO(Unit):
  IO.print(U32.show(F32.bits(value)) ++ " " ++ F32.show(value))

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- observe(F32.exp(1.0))
    Unit <- observe(F32.log(F32.exp(1.0)))
    Unit <- observe(F32.log2(8.0))
    Unit <- observe(F32.log10(1000.0))
    Unit <- observe(F32.sqrt(2.0))
    Unit <- observe(F32.sin(1.0))
    Unit <- observe(F32.cos(1.0))
    Unit <- observe(F32.tan(1.0))
    Unit <- observe(F32.sinh(1.0))
    Unit <- observe(F32.cosh(1.0))
    Unit <- observe(F32.tanh(1.0))
    Unit <- observe(F32.tanh(20.0))
    Unit <- observe(F32.sqrt(F32.neg(0.0)))
    Unit <- observe(F32.log(0.0))
    Unit <- observe(F32.exp(100.0))
    IO.print(F32.show(F32.sqrt(F32.neg(1.0))))
