# SPDX-License-Identifier: MPL-2.0
type T is Type:
  L{f: T -> T}
  V{}
def bad(t: T) -> T:
  match t:
    case L{
      f
    }:
      f(L{f})
    case V{}:
      V{}
def main() -> T: V{}
