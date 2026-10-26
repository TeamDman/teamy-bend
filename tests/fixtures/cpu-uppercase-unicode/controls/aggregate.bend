# SPDX-License-Identifier: MPL-2.0
import Base
def main() -> List<U32>:
  [Char.to_u32('\U{0}'), Char.to_u32('\U{1}'), Char.to_u32('\U{22}'), Char.to_u32('\U{5c}'), Char.to_u32('\U{41}'), Char.to_u32('\U{00000041}'), Char.to_u32('\U{D7FF}'), Char.to_u32('\U{e000}'), Char.to_u32('\U{1f600}'), Char.to_u32('\U{0010fFfF}')]
