# SPDX-License-Identifier: MPL-2.0
import Base
law ghost: U32
def both(~held: U32) -> U32:
  U32.add(held, held)
def bad() -> U32: both(~ghost)
def ghost(): 3
def main() -> U32: bad()
