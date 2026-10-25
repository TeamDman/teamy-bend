# SPDX-License-Identifier: MPL-2.0
type Flag is Data: A{} B{}
type T is Type: L{f: T -> T}
def bad(t: T, flag: Flag) -> T:
  match t, flag:
    case L{first}, A{}:
      first(L{x => x})
    case L{second}, B{}:
      second(L{second})
def main() -> T: L{t => t}
