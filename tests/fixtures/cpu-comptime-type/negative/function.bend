# SPDX-License-Identifier: MPL-2.0
import Base
def bad(f: Nat, x: Nat) -> Nat: f(x)
def main() -> Nat: bad(1n, 3n)
