# SPDX-License-Identifier: MPL-2.0
import Base
import ../stacked_computed_match_000_sel.bend as H

def observe(value: H.R) -> U32:
  match value:
    case H.K0{}: 0
    case H.K1{}: 1
    case H.K2{}: 2
    case H.K3{}: 3

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
