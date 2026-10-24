# SPDX-License-Identifier: MPL-2.0
import Base
import ../joint_fused_canonical_le.bend as H
def from_nat(n: Nat) -> H.Nat:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def value(a: Nat, b: Nat) -> H.Bool: H.main(from_nat(a), from_nat(b))
def main() -> List<H.Bool>:
  [value(0n, 0n), value(0n, 1n), value(0n, 2n), value(0n, 4n), value(1n, 0n), value(1n, 1n), value(1n, 2n), value(1n, 4n), value(2n, 0n), value(2n, 1n), value(2n, 2n), value(2n, 4n), value(4n, 0n), value(4n, 1n), value(4n, 2n), value(4n, 4n)]
