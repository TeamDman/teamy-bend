# SPDX-License-Identifier: MPL-2.0
import Base
import ../cross_binder_promotion.bend as H
def size(n: H.Nat) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: U32.add(1, size(p))

def observe(value: H.Nat) -> List<U32>:
  [size(value)]

def main() -> List<List<U32>>:
  [observe(H.main(H.P{H.Z{}, H.Z{}}, H.Z{})),
   observe(H.main(H.P{H.Z{}, H.Z{}}, H.S{H.Z{}})),
   observe(H.main(H.P{H.Z{}, H.Z{}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.P{H.Z{}, H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.P{H.Z{}, H.S{H.Z{}}}, H.S{H.Z{}})),
   observe(H.main(H.P{H.Z{}, H.S{H.Z{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.P{H.Z{}, H.S{H.S{H.Z{}}}}, H.Z{})),
   observe(H.main(H.P{H.Z{}, H.S{H.S{H.Z{}}}}, H.S{H.Z{}})),
   observe(H.main(H.P{H.Z{}, H.S{H.S{H.Z{}}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.P{H.S{H.Z{}}, H.Z{}}, H.Z{})),
   observe(H.main(H.P{H.S{H.Z{}}, H.Z{}}, H.S{H.Z{}})),
   observe(H.main(H.P{H.S{H.Z{}}, H.Z{}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.P{H.S{H.Z{}}, H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.P{H.S{H.Z{}}, H.S{H.Z{}}}, H.S{H.Z{}})),
   observe(H.main(H.P{H.S{H.Z{}}, H.S{H.Z{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.P{H.S{H.Z{}}, H.S{H.S{H.Z{}}}}, H.Z{})),
   observe(H.main(H.P{H.S{H.Z{}}, H.S{H.S{H.Z{}}}}, H.S{H.Z{}})),
   observe(H.main(H.P{H.S{H.Z{}}, H.S{H.S{H.Z{}}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.P{H.S{H.S{H.Z{}}}, H.Z{}}, H.Z{})),
   observe(H.main(H.P{H.S{H.S{H.Z{}}}, H.Z{}}, H.S{H.Z{}})),
   observe(H.main(H.P{H.S{H.S{H.Z{}}}, H.Z{}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.P{H.S{H.S{H.Z{}}}, H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.P{H.S{H.S{H.Z{}}}, H.S{H.Z{}}}, H.S{H.Z{}})),
   observe(H.main(H.P{H.S{H.S{H.Z{}}}, H.S{H.Z{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.P{H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}}, H.Z{})),
   observe(H.main(H.P{H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}}, H.S{H.Z{}})),
   observe(H.main(H.P{H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}}, H.S{H.S{H.Z{}}}))]
