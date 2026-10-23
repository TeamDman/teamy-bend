# SPDX-License-Identifier: MPL-2.0
import Base
def limit() -> Nat: Nat.mul(U32.to_nat(16777215), U32.to_nat(16777217))
def discard(n: Nat) -> String: "kept"
def main() -> IO(Unit): IO.print(discard(Nat.add(limit(), 1n)))
