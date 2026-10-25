# SPDX-License-Identifier: MPL-2.0
type T is Type: L{f: T -> T}
def bad(f: T, other: T) -> T:
  match f, other:
    case L{f}, L{g}:
      f(g(L{f}))
def main() -> T: L{t => t}
