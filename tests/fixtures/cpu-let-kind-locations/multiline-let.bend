# SPDX-License-Identifier: MPL-2.0
type T is Type: L{f: T -> T}
def bad(f: T -> T) -> T:
  +g =
    f
  g(L{f})
def main() -> T: L{t => t}
