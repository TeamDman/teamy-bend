# SPDX-License-Identifier: MPL-2.0
import Base
import ../stacked_computed_match_001_sel.bend as H

def observe(value: H.R) -> U32:
  match value:
    case H.K1{}: 1
    case H.K2{}: 2
    case H.K3{}: 3
    case H.K4{}: 4

def main() -> List<U32>:
  [observe(H.main(H.T{}, H.T{}, H.T{})),
   observe(H.main(H.T{}, H.T{}, H.F{})),
   observe(H.main(H.T{}, H.F{}, H.T{})),
   observe(H.main(H.T{}, H.F{}, H.F{})),
   observe(H.main(H.F{}, H.T{}, H.T{})),
   observe(H.main(H.F{}, H.T{}, H.F{})),
   observe(H.main(H.F{}, H.F{}, H.T{})),
   observe(H.main(H.F{}, H.F{}, H.F{}))]
