# SPDX-License-Identifier: MPL-2.0
import Base
import ../mixed_computed_column_003_id.bend as H

def observe(value: H.Nt) -> U32:
  match value:
    case H.Zn{}: 0
    case H.Sn{p}: U32.add(1, observe(p))

def main() -> List<U32>:
  [observe(H.main(H.Zn{})),
   observe(H.main(H.Sn{H.Zn{}})),
   observe(H.main(H.Sn{H.Sn{H.Zn{}}}))]
