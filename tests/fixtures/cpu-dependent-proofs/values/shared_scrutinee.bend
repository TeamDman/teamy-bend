# SPDX-License-Identifier: MPL-2.0
import Base
import ../shared_scrutinee.bend as H
def make(n: Nat) -> H.N:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{make(p)}
def value(n: Nat) -> H.N: H.add(make(n), H.Z{})
def main() -> List<H.N>: [value(0n), value(1n), value(2n), value(4n)]
