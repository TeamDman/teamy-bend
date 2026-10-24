# SPDX-License-Identifier: MPL-2.0
import Base
import ../computed_ref_call_001_step.bend as H

def observe(value: H.W) -> U32:
  match value:
    case H.Z{}: 0
    case H.S{p}: U32.add(1, observe(p))

def main() -> List<U32>:
  [observe(H.main(H.T{})),
   observe(H.main(H.F{}))]
