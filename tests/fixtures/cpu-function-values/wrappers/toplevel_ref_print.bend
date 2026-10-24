# SPDX-License-Identifier: MPL-2.0
import Base
import ../toplevel_ref_print.bend as H
def main() -> List<U32>:
  [U32.from_nat(H.main()(0n)), U32.from_nat(H.main()(1n)), U32.from_nat(H.main()(2n)), U32.from_nat(H.main()(7n))]
