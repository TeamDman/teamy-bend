# SPDX-License-Identifier: MPL-2.0
import Base
import ../wrappers/list_ctor_000.bend as O
def field_text(field: List<U32>) -> String:
  List.show(~&1, ~U32, ~(x => U32.show(x)), field)
def row_text(row: List<List<U32>>) -> String:
  List.show(~&1, ~List<U32>, ~field_text, row)
def main() -> IO(Unit):
  IO.print(List.show(~&1, ~List<List<U32>>, ~row_text, O.main()))
