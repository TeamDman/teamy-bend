# SPDX-License-Identifier: MPL-2.0
import Base
import ../wide_scrutinee_002.bend as H
def size(n: H.Nat) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: U32.add(1, size(p))

def observe(value: H.R) -> List<U32>:
  match value:
    case H.A{}: [0]
    case H.B{a, b, c}: [1, size(a), size(b), size(c)]
    case H.C{a, b, c}: [2, size(a), size(b), size(c)]

def main() -> List<List<U32>>:
  [observe(H.main(H.Z{}, H.Z{}, H.Z{}, H.Z{}, H.Z{})),
   observe(H.main(H.Z{}, H.Z{}, H.Z{}, H.Z{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.Z{}, H.Z{}, H.Z{}, H.S{H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.Z{}, H.Z{}, H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.Z{}, H.Z{}, H.S{H.S{H.Z{}}}, H.Z{}, H.Z{})),
   observe(H.main(H.Z{}, H.Z{}, H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.Z{}, H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.Z{}, H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.Z{}, H.S{H.S{H.Z{}}}, H.Z{}, H.Z{}, H.Z{})),
   observe(H.main(H.Z{}, H.S{H.S{H.Z{}}}, H.Z{}, H.Z{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.Z{}, H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.Z{}, H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{}, H.Z{})),
   observe(H.main(H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.Z{}, H.Z{}, H.Z{}, H.Z{})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.Z{}, H.Z{}, H.Z{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.Z{}, H.Z{}, H.S{H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.Z{}, H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}}, H.Z{}, H.Z{})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{}, H.Z{}, H.Z{})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{}, H.Z{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{}, H.Z{})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.Z{})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}, H.S{H.S{H.Z{}}}))]
