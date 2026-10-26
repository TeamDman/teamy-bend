# SPDX-License-Identifier: MPL-2.0
import Base
def sum(a: Char, b: Char, c: Char) -> U32: U32.add(U32.add(Char.to_u32(a), Char.to_u32(b)), Char.to_u32(c))
def main() -> U32: sum('\'''\'''\'')
