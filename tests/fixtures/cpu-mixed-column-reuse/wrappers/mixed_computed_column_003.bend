# SPDX-License-Identifier: MPL-2.0
import Base
import ../mixed_computed_column_003.bend as H

def observe(value: H.W) -> U32:
  match value:
    case H.Z{}: 0
    case H.S{p}: U32.add(1, observe(p))

def main() -> List<U32>:
  [observe(H.main(H.Zn{}, H.Zn{})),
   observe(H.main(H.Zn{}, H.Sn{H.Zn{}})),
   observe(H.main(H.Zn{}, H.Sn{H.Sn{H.Zn{}}})),
   observe(H.main(H.Sn{H.Zn{}}, H.Zn{})),
   observe(H.main(H.Sn{H.Zn{}}, H.Sn{H.Zn{}})),
   observe(H.main(H.Sn{H.Zn{}}, H.Sn{H.Sn{H.Zn{}}})),
   observe(H.main(H.Sn{H.Sn{H.Zn{}}}, H.Zn{})),
   observe(H.main(H.Sn{H.Sn{H.Zn{}}}, H.Sn{H.Zn{}})),
   observe(H.main(H.Sn{H.Sn{H.Zn{}}}, H.Sn{H.Sn{H.Zn{}}}))]
