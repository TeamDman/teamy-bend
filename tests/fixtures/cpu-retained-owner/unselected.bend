# SPDX-License-Identifier: MPL-2.0
import Base
def choose(flag: Bool, n: U32) -> U32:
  match flag:
    case True{}: U32.add(n, 3)
    case False{}: U32.from_nat(Nat.mul(U32.to_nat(16777216), U32.to_nat(16777216)))
def main() -> U32:
  +n = {3 : U32}
  choose(True{}, U32.add(n, n))
