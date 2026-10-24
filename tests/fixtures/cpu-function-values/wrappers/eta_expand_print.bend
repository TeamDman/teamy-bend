# SPDX-License-Identifier: MPL-2.0
import Base
import ../eta_expand_print.bend as H
def apply(n: Nat) -> Nat:
  H.main()(n)
def main() -> List<U32>:
  [U32.from_nat(apply(0n)), U32.from_nat(apply(1n)), U32.from_nat(apply(2n)), U32.from_nat(apply(7n))]
