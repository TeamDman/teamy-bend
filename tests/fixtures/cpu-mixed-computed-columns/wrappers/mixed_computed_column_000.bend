# SPDX-License-Identifier: MPL-2.0
import Base
import ../mixed_computed_column_000.bend as H

def observe(value: H.R) -> U32:
  match value:
    case H.R1{}: 1
    case H.R2{}: 2
    case H.R3{}: 3
    case H.R4{}: 4

def main() -> List<U32>:
  [observe(H.main(H.WZ{}, H.Z{})),
   observe(H.main(H.WZ{}, H.S{H.Z{}})),
   observe(H.main(H.WZ{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WS{H.WZ{}}, H.Z{})),
   observe(H.main(H.WS{H.WZ{}}, H.S{H.Z{}})),
   observe(H.main(H.WS{H.WZ{}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.Z{})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.S{H.Z{}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WS{H.WS{H.WS{H.WZ{}}}}, H.Z{})),
   observe(H.main(H.WS{H.WS{H.WS{H.WZ{}}}}, H.S{H.Z{}})),
   observe(H.main(H.WS{H.WS{H.WS{H.WZ{}}}}, H.S{H.S{H.Z{}}}))]
