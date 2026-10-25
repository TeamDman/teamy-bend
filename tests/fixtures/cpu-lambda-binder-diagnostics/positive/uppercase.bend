# SPDX-License-Identifier: MPL-2.0
import Base
def apply(f: U32 -> U32, n: U32) -> U32: f(n)
def main() -> U32: apply(Y => Y, 8)
