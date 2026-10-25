# SPDX-License-Identifier: MPL-2.0
import Base
def bad(n: Nat) -> {n == n : Nat}: x => n
def main() -> {1n == 1n : Nat}: bad(1n)
