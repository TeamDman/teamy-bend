# SPDX-License-Identifier: MPL-2.0
import Base
import ../stuck_split_mint.bend as H
def apply(n: Nat) -> Bool: H.main(n)
def flag(b: Bool) -> U32:
  match b:
    case True{}: 1
    case False{}: 0
def main() -> List<U32>:
  [flag(apply(0n)), flag(apply(1n)), flag(apply(2n)), flag(apply(7n)), flag(apply(16n))]
