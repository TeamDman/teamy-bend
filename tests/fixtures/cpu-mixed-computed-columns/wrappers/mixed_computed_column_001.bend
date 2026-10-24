# SPDX-License-Identifier: MPL-2.0
import Base
import ../mixed_computed_column_001.bend as H

def observe(value: H.R) -> U32:
  match value:
    case H.K0{}: 0
    case H.K1{}: 1
    case H.K2{}: 2
    case H.K3{}: 3
    case H.K4{}: 4
    case H.K5{}: 5
    case H.K6{}: 6
    case H.K7{}: 7

def main() -> List<U32>:
  [observe(H.main(H.O{}, H.O{}, H.O{})),
   observe(H.main(H.O{}, H.O{}, H.I{})),
   observe(H.main(H.O{}, H.I{}, H.O{})),
   observe(H.main(H.O{}, H.I{}, H.I{})),
   observe(H.main(H.I{}, H.O{}, H.O{})),
   observe(H.main(H.I{}, H.O{}, H.I{})),
   observe(H.main(H.I{}, H.I{}, H.O{})),
   observe(H.main(H.I{}, H.I{}, H.I{}))]
