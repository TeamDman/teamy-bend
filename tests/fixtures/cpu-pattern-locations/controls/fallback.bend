# SPDX-License-Identifier: MPL-2.0
type T is Type: L{f: T -> T}
type Pair is Type: P{x: T, y: T}
def bad(t: T) -> Pair:
  match t:
    case renamed:
      P{renamed, renamed}
def main() -> T: L{t => t}
