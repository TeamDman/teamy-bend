# SPDX-License-Identifier: MPL-2.0
import Base
def partial(n: Nat) -> Nat -> Nat: Nat.add(n)
def main() -> Nat: {partial(11n) : Nat -> Nat}(1n)
