# SPDX-License-Identifier: MPL-2.0
import Base
type T is Type: N{} S{prev: T}
def f(x: T) -> Bool:
  match x:
    case S{S{N{}}}: True{}
    case S{v}: False{}
    case N{}: False{}
def main() -> Bool: f(S{N{}})
