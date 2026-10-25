# SPDX-License-Identifier: MPL-2.0
import Base
import left.bend as L
import right.bend as R
import left.bend as Again
def add(a: L.A, b: R.B) -> U32:
  match a, b:
    case L.Same{x}, R.Same{y}: (x + y : U32)
def main() -> U32: add(L.Same{3}, R.Same{4})
