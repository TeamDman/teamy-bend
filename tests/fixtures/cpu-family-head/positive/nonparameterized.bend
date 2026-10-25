# SPDX-License-Identifier: MPL-2.0
import Base
type Flag is Data:
  Off{}
  On{}
def use(x: Flag) -> U32:
  match x:
    case Off{}: 0
    case On{}: 7
def main() -> U32: use(On{})
