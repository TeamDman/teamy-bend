# SPDX-License-Identifier: MPL-2.0
import Base
def duplicate(a: Array<U32>) -> Array<U32> & Array<U32>:
  (a, a)
def main() -> Array<U32> & Array<U32>:
  duplicate(ALeaf{7})
