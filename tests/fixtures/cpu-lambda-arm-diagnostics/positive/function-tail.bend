# SPDX-License-Identifier: MPL-2.0
import Base
def apply(f: U32 -> U32) -> U32: f(4)
def main() -> U32:
  apply(\{(x => x)})
