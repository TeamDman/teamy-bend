# SPDX-License-Identifier: MPL-2.0
import Base
def pick(c: Char) -> U32:
  match c:
    case '': 0
    case _: 1
def main() -> U32: pick('a')
