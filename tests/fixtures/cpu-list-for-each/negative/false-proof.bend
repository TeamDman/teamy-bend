# SPDX-License-Identifier: MPL-2.0
import Base
def wrong() -> {0n == 1n : Nat}: {==}
def visit(n: Nat) -> IO(Unit): IO.pure(Unit, Unit{})
def main() -> IO(Unit): List.for_each(~&2, ~Nat, ~visit, [])
