# SPDX-License-Identifier: MPL-2.0
import Base
import ../mixed_computed_column_001_flip.bend as H

def observe(value: H.Bit) -> U32:
  match value:
    case H.O{}: 0
    case H.I{}: 1

def main() -> List<U32>:
  [observe(H.main(H.O{})),
   observe(H.main(H.I{}))]
