# SPDX-License-Identifier: MPL-2.0
import Base
law ghost: U32
def bad(x: U32) -> U32:
  U32.add(x,
    ghost)
def main() -> U32: 0
