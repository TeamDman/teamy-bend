# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/read_bounds.bend as R

def prefix() -> Nat:
  Nat.add(Nat.mul(U32.to_nat(28147), U32.to_nat(1000000000)), U32.to_nat(497671065))

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(R.mn(Nat.read.go("", Nat.read.max())))
    Unit <- IO.print(R.mn(Nat.read.go("4", prefix())))
    Unit <- IO.print(R.mn(Nat.read.go("5", prefix())))
    Unit <- IO.print(R.mn(Nat.read.go("6", prefix())))
    Unit <- IO.print(R.mn(Nat.read.go("0", Nat.add(prefix(), 1n))))
    Unit <- IO.print(R.mn(Nat.read.if("", Nat.read.max(), 1, False{})))
    Unit <- IO.print(R.mn(Nat.read.if("3", 1n, 2, True{})))
    Unit <- IO.print(R.mn(Nat.read.if("bad", Nat.read.max(), 999, False{})))
    Unit <- IO.print(R.mu(U32.read.go("", 4294967295)))
    Unit <- IO.print(R.mu(U32.read.go("4", 429496729)))
    Unit <- IO.print(R.mu(U32.read.go("5", 429496729)))
    Unit <- IO.print(R.mu(U32.read.go("6", 429496729)))
    Unit <- IO.print(R.mu(U32.read.go("0", 429496730)))
    Unit <- IO.print(R.mu(U32.read.if("", 4294967295, False{})))
    Unit <- IO.print(R.mu(U32.read.if("3", 12, True{})))
    IO.print(R.mu(U32.read.if("bad", 4294967295, False{})))
