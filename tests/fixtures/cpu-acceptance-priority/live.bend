# SPDX-License-Identifier: MPL-2.0
import Base
law twice:
  for f: @+g: (Nat -> Nat) -> Nat
  Nat
@unsafe
def twice(f): f(x => 1n+x)
@unsafe
def observe() -> Nat: twice(g => g(g(3n)))
def main() -> Nat: observe()
