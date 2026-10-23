# SPDX-License-Identifier: MPL-2.0
# Authored bitwise, rounding, fallback and unselected-error controls.
import Base

def classify(x: F32) -> U32:
  match x:
    case 0.0: 10
    case 1.5: 20
    case y: F32.bits(y)

def guarded(x: F32) -> Nat:
  match x:
    case 1.5: 7n
    case y: Nat.add(Nat.mul(U32.to_nat(16777215), U32.to_nat(16777217)), 1n)

def word(x: U32) -> U32:
  match x:
    case 0: 99
    case y: y

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(U32.show(F32.bits(2.5e-3)))
    Unit <- IO.print(U32.show(F32.bits(1.000000059604644775390625)))
    Unit <- IO.print(U32.show(F32.bits(1.000000178813934326171875)))
    Unit <- IO.print(U32.show(F32.bits(1.0000000596046448)))
    Unit <- IO.print(U32.show(classify(0.0)))
    Unit <- IO.print(U32.show(classify(F32.neg(0.0))))
    Unit <- IO.print(U32.show(classify(1.5)))
    Unit <- IO.print(U32.show(classify(1.75)))
    Unit <- IO.print(Nat.show(guarded(1.5)))
    Unit <- IO.print(U32.show(word(0)))
    Unit <- IO.print(U32.show(word(1)))
    IO.print(U32.show(word(4294967295)))
