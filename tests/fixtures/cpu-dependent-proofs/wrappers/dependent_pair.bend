# SPDX-License-Identifier: MPL-2.0
import Base
import ../dependent_pair.bend as H
def count(n: H.N) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def apply() -> List<U32>: [count(H.fst(H.packed()))]
def main() -> List<List<U32>>: [apply()]
