# SPDX-License-Identifier: MPL-2.0
import Base
import ../eval_split_apply.bend as H
def make(n: Nat) -> H.MN:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{make(p)}
def value(n: Nat) -> H.MN: H.wob(make(n))
def main() -> List<H.MN>:
  [value(0n), value(1n), value(2n), value(4n)]
