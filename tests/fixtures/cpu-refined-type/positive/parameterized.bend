# SPDX-License-Identifier: MPL-2.0
import Base
def loop(xs: List<&2, U32>) -> U32:
  match xs:
    case Nil{}: 0
    case Con{x, xs}: x
def main() -> U32: loop([7, 8])
