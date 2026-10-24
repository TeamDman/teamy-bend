# SPDX-License-Identifier: MPL-2.0
import Base
import ../stuck_first.bend as H
def depth(n: H.N) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: U32.add(depth(p), 1)
def main() -> List<U32>:
  [depth(H.main()(H.Z{})), depth(H.main()(H.S{H.Z{}})), depth(H.main()(H.S{H.S{H.Z{}}}))]
