# SPDX-License-Identifier: MPL-2.0
import Base
def id(n: Nat) -> Nat: n
def main() -> IO(Unit): IO.print(Nat.show({Nat.add(id(11n)) : Nat -> Nat}(1n)))
