# SPDX-License-Identifier: MPL-2.0
import Base
import ../computed_ref_call_000_f.bend as H

def observe(value: H.Bl) -> U32:
  match value:
    case H.T{}: 1
    case H.F{}: 0

def main() -> List<U32>:
  [observe(H.main(H.Zn{})),
   observe(H.main(H.Sn{H.Zn{}})),
   observe(H.main(H.Sn{H.Sn{H.Zn{}}})),
   observe(H.main(H.Sn{H.Sn{H.Sn{H.Zn{}}}}))]
