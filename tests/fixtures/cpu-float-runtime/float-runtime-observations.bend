# SPDX-License-Identifier: MPL-2.0
# Authored finite bits/text, signed-zero storage and nonfinite class controls.
import Base

type FloatCell is Data:
  Hold{value: F32}

def unbox(cell: FloatCell) -> F32:
  match cell:
    case Hold{value}: value

def old_value(result: Array<F32> & F32) -> F32:
  (rest, old) = result
  old

def swap_out(value: F32) -> F32:
  old_value(Array.swap(F32, ANode{ALeaf{0.0}, ALeaf{value}}, 1, 0.0))

def sum_steps(n: Nat, acc: F32) -> F32:
  match n:
    case 0n: acc
    case 1n+p: sum_steps(p, F32.add(acc, 0.1))

def observe(+value: F32) -> IO(Unit):
  IO.print(U32.show(F32.bits(value)) ++ " " ++ F32.show(value))

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- observe(F32.floor(F32.neg(1.5)))
    Unit <- observe(F32.ceil(F32.neg(1.5)))
    Unit <- observe(F32.trunc(F32.neg(1.5)))
    Unit <- observe(F32.abs(F32.neg(1.5)))
    Unit <- observe(F32.floor(F32.neg(0.0)))
    Unit <- observe(F32.ceil(F32.neg(0.25)))
    Unit <- observe(F32.trunc(F32.neg(0.75)))
    Unit <- observe(F32.abs(F32.neg(0.0)))
    Unit <- observe(sum_steps(10n, 0.0))
    Unit <- observe(unbox(Hold{F32.neg(0.0)}))
    Unit <- observe(swap_out(F32.neg(0.0)))
    Unit <- observe(swap_out(1.25))
    Unit <- observe(F32.floor(F32.div(1.0, 0.0)))
    Unit <- observe(F32.ceil(F32.div(F32.neg(1.0), 0.0)))
    IO.print(F32.show(F32.trunc(F32.div(0.0, 0.0))))
