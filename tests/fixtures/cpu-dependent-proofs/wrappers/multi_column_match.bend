# SPDX-License-Identifier: MPL-2.0
import Base
import ../multi_column_match.bend as H
def make(n: Nat) -> H.Bit:
  match n:
    case 0n: H.F{}
    case 1n+p: H.T{}
def value(a: Nat, b: Nat) -> H.Bit: H.and2(make(a), make(b))
def code(b: H.Bit) -> U32:
  match b:
    case H.T{}: 1
    case H.F{}: 0
def apply(a: Nat, b: Nat) -> List<U32>: [code(value(a,b))]
def main() -> List<List<U32>>: [apply(1n, 1n), apply(1n, 0n), apply(0n, 1n), apply(0n, 0n)]
