# SPDX-License-Identifier: MPL-2.0
import Base
import ../var_row_column_rebind.bend as H
def size(n: H.W) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: U32.add(1, size(p))

def observe(value: H.R) -> List<U32>:
  match value:
    case H.K10{}: [10, 0]
    case H.KN{n}: [0, size(n)]
    case H.K99{}: [99, 0]

def main() -> List<List<U32>>:
  [observe(H.main(H.Z{})),
   observe(H.main(H.S{H.Z{}})),
   observe(H.main(H.S{H.S{H.Z{}}})),
   observe(H.main(H.S{H.S{H.S{H.Z{}}}})),
   observe(H.main(H.S{H.S{H.S{H.S{H.Z{}}}}}))]
