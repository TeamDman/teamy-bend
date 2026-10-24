# SPDX-License-Identifier: MPL-2.0
import Base
import ../wrappers/wide_scrutinee_003.bend as O
def row_text(row: List<U32>) -> String:
  List.show(~&1, ~U32, ~(x => U32.show(x)), row)
def main() -> IO(Unit):
  IO.print(List.show(~&1, ~List<U32>, ~row_text, O.main()))
