# SPDX-License-Identifier: MPL-2.0
import Base
def app(~f: Nat -> Nat, x: Nat) -> Nat: f(x)
def main() -> Nat: app(~(x => Nat.add(x, 1n)), 3n)
