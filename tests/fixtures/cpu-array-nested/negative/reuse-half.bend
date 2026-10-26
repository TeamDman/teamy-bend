# SPDX-License-Identifier: MPL-2.0
import Base

def bad(a: Array<U32>) -> Array<U32>:
  match a:
    case ALeaf{x}: ALeaf{x}
    case ANode{left, right}: ANode{left, left}

def main() -> Array<U32>:
  bad(ANode{ALeaf{1}, ALeaf{2}})
