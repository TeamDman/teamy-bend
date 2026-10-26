# SPDX-License-Identifier: MPL-2.0
import Base
def pick(c: Char) -> U32:
  match c:
    case ''': 39
    case _: 0
def main() -> U32: pick(''')
