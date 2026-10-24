# SPDX-License-Identifier: MPL-2.0
import Base
import ../stacked_computed_match_000.bend as H

def observe(value: H.R) -> U32:
  match value:
    case H.K0{}: 0
    case H.K1{}: 1
    case H.K2{}: 2
    case H.K3{}: 3

def main() -> List<U32>:
  [observe(H.main(H.WZ{}, H.WZ{})),
   observe(H.main(H.WZ{}, H.WS{H.WZ{}})),
   observe(H.main(H.WZ{}, H.WS{H.WS{H.WZ{}}})),
   observe(H.main(H.WS{H.WZ{}}, H.WZ{})),
   observe(H.main(H.WS{H.WZ{}}, H.WS{H.WZ{}})),
   observe(H.main(H.WS{H.WZ{}}, H.WS{H.WS{H.WZ{}}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WZ{})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WS{H.WZ{}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}, H.WS{H.WS{H.WZ{}}}))]
