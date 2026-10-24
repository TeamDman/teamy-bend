# SPDX-License-Identifier: MPL-2.0
import Base
import ../stacked_computed_match_000_nat.bend as H

def observe(value: H.Nt) -> U32:
  match value:
    case H.Zn{}: 0
    case H.Sn{p}: U32.add(1, observe(p))

def main() -> List<U32>:
  [observe(H.main(H.WZ{})),
   observe(H.main(H.WS{H.WZ{}})),
   observe(H.main(H.WS{H.WS{H.WZ{}}}))]
