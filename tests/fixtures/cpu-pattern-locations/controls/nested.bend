# SPDX-License-Identifier: MPL-2.0
type T is Type: L{f: T -> T}
type Box is Type: B{v: T}
def bad(b: Box) -> T:
  match b:
    case B{L{f}}:
      f(L{f})
def main() -> Box: B{L{t => t}}
