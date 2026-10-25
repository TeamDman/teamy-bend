# SPDX-License-Identifier: MPL-2.0
import Base
def visit(n: U32) -> IO(U32): IO.pure(U32, n)
def main() -> IO(Unit): List.for_each(~&2, ~U32, ~visit, [1,2])
