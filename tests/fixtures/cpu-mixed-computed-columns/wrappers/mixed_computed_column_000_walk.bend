# SPDX-License-Identifier: MPL-2.0
import Base
import ../mixed_computed_column_000_walk.bend as H

def observe(value: H.W) -> U32:
  match value:
    case H.WZ{}: 0
    case H.WS{p}: U32.add(1, observe(p))

def main() -> List<U32>:
  [observe(H.main(H.Z{}, H.WZ{})),
   observe(H.main(H.Z{}, H.WS{H.WZ{}})),
   observe(H.main(H.Z{}, H.WS{H.WS{H.WZ{}}})),
   observe(H.main(H.S{H.Z{}}, H.WZ{})),
   observe(H.main(H.S{H.Z{}}, H.WS{H.WZ{}})),
   observe(H.main(H.S{H.Z{}}, H.WS{H.WS{H.WZ{}}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.WZ{})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.WS{H.WZ{}})),
   observe(H.main(H.S{H.S{H.Z{}}}, H.WS{H.WS{H.WZ{}}}))]
