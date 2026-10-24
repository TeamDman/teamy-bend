# SPDX-License-Identifier: MPL-2.0
import Base
import ../computed_match_specialize.bend as H
def count(n: H.Nf) -> U32:
  match n:
    case H.Zf{}: 0
    case H.Sf{p}: (1 + count(p) : U32)
def main() -> List<U32>: [count(H.dbl(H.Zf{})), count(H.dbl(H.Sf{H.Zf{}})), count(H.dbl(H.Sf{H.Sf{H.Zf{}}})), count(H.dbl(H.Sf{H.Sf{H.Sf{H.Sf{H.Zf{}}}}}))]
