# SPDX-License-Identifier: MPL-2.0
import Base
law twice:
  for f: @+g: (Nat -> Nat) -> Nat
  Nat
@unsafe
def twice(f): f(x => 1n+x)
def wrong(+f: Nat -> Nat) -> Nat: f(0n)
def main() -> U32: 7
