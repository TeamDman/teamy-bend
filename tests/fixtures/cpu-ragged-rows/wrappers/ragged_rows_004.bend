# SPDX-License-Identifier: MPL-2.0
import Base
import ../ragged_rows_004.bend as H
def observe(value: H.R) -> List<U32>:
  match value:
    case H.R1{}: [1]
    case H.R2{}: [2]
    case H.R3{}: [3]

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
