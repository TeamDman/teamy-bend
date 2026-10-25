# SPDX-License-Identifier: MPL-2.0
import Base
def call(f: U32 -> U32, x: U32) -> U32: f(x)
def main() -> U32:
  +n = {3 : U32}
  f = {x => U32.add(n, x) : U32 -> U32}
  U32.add(call(f, n), n)
