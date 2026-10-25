# SPDX-License-Identifier: MPL-2.0
type I is Data: A{} B{}
type O is Data: W{i: I}
type P is Data: Pair{a: I, b: I}
def bad(o: O) -> P:
  match o:
    case W{A{}}:
      Pair{A{}, A{}}
    case W{x}:
      Pair{x, x}
def main() -> P: Pair{A{}, A{}}
