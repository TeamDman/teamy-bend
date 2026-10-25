# SPDX-License-Identifier: MPL-2.0
import Base
type Flag is Data: Off{} On{}
def read(x: Flag) -> U32:
  match x:
    case Off{}: 2
    case On{}: 5
def main() -> U32: read(On{})
