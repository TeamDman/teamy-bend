# SPDX-License-Identifier: MPL-2.0
import Base
import ../ragged_rows_000.bend as H
def size(n: H.Nat) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: U32.add(1, size(p))

def observe(value: H.R) -> List<U32>:
  match value:
    case H.A{a, b}: [1, size(a), size(b)]
    case H.B{a, b}: [2, size(a), size(b)]
    case H.C{a}: [3, size(a)]
    case H.D{a, b}: [4, size(a), size(b)]
    case H.E{a, b}: [5, size(a), size(b)]
    case H.F{a}: [6, size(a)]

def main() -> List<List<U32>>:
  [observe(H.main(H.Z{}, H.Z{})),
   observe(H.main(H.Z{}, H.S{H.Z{}})),
   observe(H.main(H.Z{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.Z{}}, H.Z{})),
   observe(H.main(H.S{H.Z{}}, H.S{H.Z{}})),
   observe(H.main(H.S{H.Z{}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.S{H.Z{}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}))]
