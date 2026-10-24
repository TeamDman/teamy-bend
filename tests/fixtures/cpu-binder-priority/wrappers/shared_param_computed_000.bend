# SPDX-License-Identifier: MPL-2.0
import Base
import ../shared_param_computed_000.bend as H
def size(n: H.Nat) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: U32.add(1, size(p))

def observe(value: H.Nat) -> List<U32>:
  [size(value)]

def main() -> List<List<U32>>:
  [observe(H.main(H.Z{})),
   observe(H.main(H.S{H.Z{}})),
   observe(H.main(H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.S{H.Z{}}}}))]
