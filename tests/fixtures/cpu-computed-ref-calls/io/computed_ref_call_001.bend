# SPDX-License-Identifier: MPL-2.0
import Base
import ../wrappers/computed_ref_call_001.bend as O

def main() -> IO(Unit):
  IO.print(List.show(~&1, ~U32, ~(x => U32.show(x)), O.main()))
