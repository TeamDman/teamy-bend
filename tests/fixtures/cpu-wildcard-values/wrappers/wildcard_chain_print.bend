# SPDX-License-Identifier: MPL-2.0
import Base
import ../wildcard_chain_print.bend as H
def apply(x: U32, y: U32) -> Nat:
  H.main(x, y)
def partial(x: U32) -> @y:U32 -> Nat:
  H.main(x)
def main() -> List<U32>:
  [U32.from_nat(apply(0, 0)), U32.from_nat(apply(0, 7)), U32.from_nat(apply(0, 4294967295)), U32.from_nat(apply(7, 0)), U32.from_nat(apply(7, 7)), U32.from_nat(apply(7, 4294967295)), U32.from_nat(apply(4294967295, 0)), U32.from_nat(apply(4294967295, 7)), U32.from_nat(apply(4294967295, 4294967295))]
