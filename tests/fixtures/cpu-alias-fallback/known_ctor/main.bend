# SPDX-License-Identifier: MPL-2.0
import Base
import defs.bend as D
type D.Box is Data:
  D.Hold{}
def get(value: D.Box) -> U32:
  match value:
    case D.Hold{value}: value
def main() -> U32: get(D.Hold{9})
