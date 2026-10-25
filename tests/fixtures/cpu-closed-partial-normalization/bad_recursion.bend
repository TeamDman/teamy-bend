# SPDX-License-Identifier: MPL-2.0
import Base
def loop(n: Nat) -> Nat: {Nat.add(loop(n)) : Nat -> Nat}(1n)
