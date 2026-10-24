# SPDX-License-Identifier: MPL-2.0
import Base
import ../type_param_match_001.bend as H
def from_nat(n: Nat) -> H.Nat:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def value(k: Nat, b: Nat, c: Nat, d: Nat) -> H.R:
  match k:
    case 0n: H.main(H.Nat, from_nat(b), from_nat(c), from_nat(d))
    case 1n+p: H.main((H.Nat -> H.Nat), from_nat(b), from_nat(c), from_nat(d))
def main() -> List<H.R>:
  [value(0n, 0n, 0n, 0n), value(0n, 0n, 0n, 1n), value(0n, 0n, 0n, 2n), value(0n, 0n, 1n, 0n), value(0n, 0n, 1n, 1n), value(0n, 0n, 1n, 2n), value(0n, 0n, 2n, 0n), value(0n, 0n, 2n, 1n), value(0n, 0n, 2n, 2n), value(0n, 1n, 0n, 0n), value(0n, 1n, 0n, 1n), value(0n, 1n, 0n, 2n), value(0n, 1n, 1n, 0n), value(0n, 1n, 1n, 1n), value(0n, 1n, 1n, 2n), value(0n, 1n, 2n, 0n), value(0n, 1n, 2n, 1n), value(0n, 1n, 2n, 2n), value(0n, 2n, 0n, 0n), value(0n, 2n, 0n, 1n), value(0n, 2n, 0n, 2n), value(0n, 2n, 1n, 0n), value(0n, 2n, 1n, 1n), value(0n, 2n, 1n, 2n), value(0n, 2n, 2n, 0n), value(0n, 2n, 2n, 1n), value(0n, 2n, 2n, 2n), value(1n, 0n, 0n, 0n), value(1n, 0n, 0n, 1n), value(1n, 0n, 0n, 2n), value(1n, 0n, 1n, 0n), value(1n, 0n, 1n, 1n), value(1n, 0n, 1n, 2n), value(1n, 0n, 2n, 0n), value(1n, 0n, 2n, 1n), value(1n, 0n, 2n, 2n), value(1n, 1n, 0n, 0n), value(1n, 1n, 0n, 1n), value(1n, 1n, 0n, 2n), value(1n, 1n, 1n, 0n), value(1n, 1n, 1n, 1n), value(1n, 1n, 1n, 2n), value(1n, 1n, 2n, 0n), value(1n, 1n, 2n, 1n), value(1n, 1n, 2n, 2n), value(1n, 2n, 0n, 0n), value(1n, 2n, 0n, 1n), value(1n, 2n, 0n, 2n), value(1n, 2n, 1n, 0n), value(1n, 2n, 1n, 1n), value(1n, 2n, 1n, 2n), value(1n, 2n, 2n, 0n), value(1n, 2n, 2n, 1n), value(1n, 2n, 2n, 2n)]
