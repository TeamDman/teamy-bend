# SPDX-License-Identifier: MPL-2.0
import Base
def main() -> U32:
  -bad = Nat.mul(U32.to_nat(16777216), U32.to_nat(16777216))
  +n = {3 : U32}
  U32.add(n, n)
