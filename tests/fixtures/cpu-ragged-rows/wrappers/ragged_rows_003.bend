# SPDX-License-Identifier: MPL-2.0
import Base
import ../ragged_rows_003.bend as H
def observe(value: H.R) -> List<U32>:
  match value:
    case H.K100{}: [1]
    case H.K200{}: [2]
    case H.K300{}: [3]

def main() -> List<List<U32>>:
  [observe(H.main(H.WZ{}, H.WZ{}, H.Z{})),
   observe(H.main(H.WZ{}, H.WZ{}, H.S{H.Z{}})),
   observe(H.main(H.WZ{}, H.WZ{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WZ{}, H.WS{H.WZ{}}, H.Z{})),
   observe(H.main(H.WZ{}, H.WS{H.WZ{}}, H.S{H.Z{}})),
   observe(H.main(H.WZ{}, H.WS{H.WZ{}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WZ{}, H.WS{H.WS{H.WZ{}}}, H.Z{})),
   observe(H.main(H.WZ{}, H.WS{H.WS{H.WZ{}}}, H.S{H.Z{}})),
   observe(H.main(H.WZ{}, H.WS{H.WS{H.WZ{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WS{H.WZ{}}, H.WZ{}, H.Z{})),
   observe(H.main(H.WS{H.WZ{}}, H.WZ{}, H.S{H.Z{}})),
   observe(H.main(H.WS{H.WZ{}}, H.WZ{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WS{H.WZ{}}, H.WS{H.WZ{}}, H.Z{})),
   observe(H.main(H.WS{H.WZ{}}, H.WS{H.WZ{}}, H.S{H.Z{}})),
   observe(H.main(H.WS{H.WZ{}}, H.WS{H.WZ{}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WS{H.WZ{}}, H.WS{H.WS{H.WZ{}}}, H.Z{})),
   observe(H.main(H.WS{H.WZ{}}, H.WS{H.WS{H.WZ{}}}, H.S{H.Z{}})),
   observe(H.main(H.WS{H.WZ{}}, H.WS{H.WS{H.WZ{}}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WZ{}, H.Z{})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WZ{}, H.S{H.Z{}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WZ{}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WS{H.WZ{}}, H.Z{})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WS{H.WZ{}}, H.S{H.Z{}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WS{H.WZ{}}, H.S{H.S{H.Z{}}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WS{H.WS{H.WZ{}}}, H.Z{})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WS{H.WS{H.WZ{}}}, H.S{H.Z{}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WS{H.WS{H.WZ{}}}, H.S{H.S{H.Z{}}}))]
