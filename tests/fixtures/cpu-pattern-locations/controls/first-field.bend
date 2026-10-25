# SPDX-License-Identifier: MPL-2.0
type T is Type: L{f: T -> T, g: T -> T}
def bad(t: T) -> T:
  match t:
    case L{f, g}:
      f(g(L{f, g}))
def main() -> T: L{t => t, t => t}
