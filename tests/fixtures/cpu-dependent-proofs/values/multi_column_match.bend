# SPDX-License-Identifier: MPL-2.0
import Base
import ../multi_column_match.bend as H
def make(n: Nat) -> H.Bit:
  match n:
    case 0n: H.F{}
    case 1n+p: H.T{}
def value(a: Nat, b: Nat) -> H.Bit: H.and2(make(a), make(b))
def main() -> List<H.Bit>: [value(1n, 1n), value(1n, 0n), value(0n, 1n), value(0n, 0n)]
