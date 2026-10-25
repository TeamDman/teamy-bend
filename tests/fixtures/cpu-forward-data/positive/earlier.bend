# SPDX-License-Identifier: MPL-2.0
import Base
type Earlier<-A: Type> is Type:
  Item{value: A}
type Wrapper is Type:
  Wrapped{value: Earlier<U32>}
def extract(w: Wrapper) -> U32:
  match w:
    case Wrapped{Item{x}}: x
def main() -> U32: extract(Wrapped{Item{7}})
