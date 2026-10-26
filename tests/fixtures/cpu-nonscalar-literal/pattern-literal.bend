# SPDX-License-Identifier: MPL-2.0
import Base
def f(c: Char) -> U32:
  match c:
    case '\u{d800}': 1
    case _: 0
def main() -> U32: f('A')
