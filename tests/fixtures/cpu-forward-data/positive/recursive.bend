# SPDX-License-Identifier: MPL-2.0
import Base
type Chain is Data:
  End{}
  Link{value: U32, rest: Chain}
def total(xs: Chain) -> U32:
  match xs:
    case End{}: 0
    case Link{x, xs}: (x + total(xs) : U32)
def main() -> U32: total(Link{3, Link{4, End{}}})
