# SPDX-License-Identifier: MPL-2.0
import Base
def inc(n: Nat) -> Nat: 1n+n
def good(~f: Nat -> Nat, x: Nat) -> Nat: f(x)
def main() -> Nat: Nat.add(good(~inc, 1n), good(~inc, 2n))
