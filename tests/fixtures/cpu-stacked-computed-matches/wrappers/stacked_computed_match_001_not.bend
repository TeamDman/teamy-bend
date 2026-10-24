# SPDX-License-Identifier: MPL-2.0
import Base
import ../stacked_computed_match_001_not.bend as H

def observe(value: H.Bl) -> U32:
  match value:
    case H.T{}: 1
    case H.F{}: 0

def main() -> List<U32>:
  [observe(H.main(H.T{})),
   observe(H.main(H.F{}))]
