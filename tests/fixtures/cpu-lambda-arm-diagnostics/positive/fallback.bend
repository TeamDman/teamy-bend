# SPDX-License-Identifier: MPL-2.0
import Base
def apply(f: Bool -> U32, x: Bool) -> U32: f(x)
def main() -> U32:
  apply(\{(True): 7; (x => 3)}, False{})
