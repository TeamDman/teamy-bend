# SPDX-License-Identifier: MPL-2.0
# Exact finite/infinity/zero bits, comparison results and NaN text/classification.
# NaN payload/sign bits and runtime F32.read are deliberately not observed.
import Base

def observe(+value: F32) -> IO(Unit):
  IO.print(U32.show(F32.bits(value)) ++ " " ++ F32.show(value))

def bu(value: Bool) -> U32:
  match value:
    case False{}: 0
    case True{}: 1

def comparisons(+a: F32, +b: F32) -> String:
  U32.show(bu(F32.is_eq(a, b))) ++ " " ++
  U32.show(bu(F32.is_ne(a, b))) ++ " " ++
  U32.show(bu(F32.is_lt(a, b))) ++ " " ++
  U32.show(bu(F32.is_le(a, b))) ++ " " ++
  U32.show(bu(F32.is_gt(a, b))) ++ " " ++
  U32.show(bu(F32.is_ge(a, b)))

def step(+n: Nat, x: F32) -> F32:
  match n:
    case 0n: x
    case 1n+p:
      step(p, F32.div(F32.add(x, U32.to_f32(U32.add(1, U32.from_nat(p)))), 2.0))

def count(+n: Nat, acc: U32) -> U32:
  match n:
    case 0n: acc
    case 1n+p:
      count(p, U32.add(acc, bu(F32.is_lt(U32.to_f32(U32.add(1, U32.from_nat(p))), 5.5))))

def finite_observations() -> IO(Unit):
  +result = step(20n, 1.5)
  do IO<Unit>:
    Unit <- observe((1.5 * 2.0 : F32))
    Unit <- observe((0.25 + 2.75 : F32))
    Unit <- observe((10.0 / 4.0 : F32))
    Unit <- IO.print(U32.show(F32.to_u32((10.0 / 4.0 : F32))))
    Unit <- observe(result)
    Unit <- observe(F32.mul(result, U32.to_f32(1000000)))
    IO.print(U32.show(count(9n, 0)))

def nonfinite_observations(+nan: F32, +inf: F32, +neg_inf: F32, +neg_zero: F32) -> IO(Unit):
  do IO<Unit>:
    Unit <- observe(inf)
    Unit <- observe(neg_inf)
    Unit <- observe(neg_zero)
    Unit <- IO.print(F32.show(nan))
    Unit <- IO.print(comparisons(nan, nan))
    Unit <- IO.print(comparisons(nan, 1.0))
    Unit <- IO.print(comparisons(inf, 1.0))
    Unit <- IO.print(comparisons(neg_inf, 1.0))
    IO.print(comparisons(neg_zero, 0.0))

def conversions(+nan: F32, inf: F32, neg_inf: F32, neg_zero: F32) -> IO(Unit):
  IO.print(U32.show(F32.to_u32(nan)) ++ " " ++
    U32.show(F32.to_u32(inf)) ++ " " ++
    U32.show(F32.to_u32(neg_inf)) ++ " " ++
    U32.show(F32.to_u32((0.0 - 2.5 : F32))) ++ " " ++
    U32.show(F32.to_u32(neg_zero)) ++ " " ++
    U32.show(F32.to_u32(0.5)) ++ " " ++
    U32.show(F32.to_u32(1.0)) ++ " " ++
    U32.show(F32.to_u32(U32.to_f32(4294967040))) ++ " " ++
    U32.show(F32.to_u32(U32.to_f32(4294967295))))

def main() -> IO(Unit):
  +nan = F32.div(0.0, 0.0)
  +inf = F32.div(1.0, 0.0)
  +neg_inf = F32.div((0.0 - 1.0 : F32), 0.0)
  +neg_zero = F32.mul(0.0, (0.0 - 1.0 : F32))
  do IO<Unit>:
    Unit <- finite_observations()
    Unit <- nonfinite_observations(nan, inf, neg_inf, neg_zero)
    conversions(nan, inf, neg_inf, neg_zero)
