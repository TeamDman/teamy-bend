# SPDX-License-Identifier: MPL-2.0
import Base
def twice(+x: Nat) -> Nat: Nat.add(x, x)
def main() -> Nat: twice(3n)
