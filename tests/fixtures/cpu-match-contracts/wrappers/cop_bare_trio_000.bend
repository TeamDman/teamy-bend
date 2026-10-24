# SPDX-License-Identifier: MPL-2.0
import Base
import ../cop_bare_trio_000.bend as H
def count(n: H.Nat) -> U32:
  match n:
    case H.Zero{}: 0
    case H.Succ{p}: (1 + count(p) : U32)
def main() -> List<U32>: [count(H.addself(H.Zero{})), count(H.addself(H.Succ{H.Zero{}})), count(H.addself(H.Succ{H.Succ{H.Zero{}}})), count(H.addself(H.Succ{H.Succ{H.Succ{H.Succ{H.Zero{}}}}})), count(H.Nat.add(H.Zero{}, H.Zero{})), count(H.Nat.add(H.Zero{}, H.Succ{H.Zero{}})), count(H.Nat.add(H.Zero{}, H.Succ{H.Succ{H.Zero{}}})), count(H.Nat.add(H.Zero{}, H.Succ{H.Succ{H.Succ{H.Succ{H.Zero{}}}}})), count(H.Nat.add(H.Succ{H.Zero{}}, H.Zero{})), count(H.Nat.add(H.Succ{H.Zero{}}, H.Succ{H.Zero{}})), count(H.Nat.add(H.Succ{H.Zero{}}, H.Succ{H.Succ{H.Zero{}}})), count(H.Nat.add(H.Succ{H.Zero{}}, H.Succ{H.Succ{H.Succ{H.Succ{H.Zero{}}}}})), count(H.Nat.add(H.Succ{H.Succ{H.Zero{}}}, H.Zero{})), count(H.Nat.add(H.Succ{H.Succ{H.Zero{}}}, H.Succ{H.Zero{}})), count(H.Nat.add(H.Succ{H.Succ{H.Zero{}}}, H.Succ{H.Succ{H.Zero{}}})), count(H.Nat.add(H.Succ{H.Succ{H.Zero{}}}, H.Succ{H.Succ{H.Succ{H.Succ{H.Zero{}}}}})), count(H.Nat.add(H.Succ{H.Succ{H.Succ{H.Succ{H.Zero{}}}}}, H.Zero{})), count(H.Nat.add(H.Succ{H.Succ{H.Succ{H.Succ{H.Zero{}}}}}, H.Succ{H.Zero{}})), count(H.Nat.add(H.Succ{H.Succ{H.Succ{H.Succ{H.Zero{}}}}}, H.Succ{H.Succ{H.Zero{}}})), count(H.Nat.add(H.Succ{H.Succ{H.Succ{H.Succ{H.Zero{}}}}}, H.Succ{H.Succ{H.Succ{H.Succ{H.Zero{}}}}}))]
