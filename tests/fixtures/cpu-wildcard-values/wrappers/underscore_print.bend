# SPDX-License-Identifier: MPL-2.0
import Base
import ../underscore_print.bend as H
def apply(x: U32) -> Nat:
  H.main(x)
def main() -> List<U32>:
  [U32.from_nat(apply(0)), U32.from_nat(apply(7)), U32.from_nat(apply(4294967295))]
