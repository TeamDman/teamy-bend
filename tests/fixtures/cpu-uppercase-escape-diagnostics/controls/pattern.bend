# SPDX-License-Identifier: MPL-2.0
import Base
def f(x: Char) -> U32:
  match x:
    case '\U{}': 1
    case _: 0
def main() -> U32: f('A')
