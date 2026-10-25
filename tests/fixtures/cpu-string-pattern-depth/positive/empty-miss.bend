# SPDX-License-Identifier: MPL-2.0
import Base
def f(s: String) -> Bool:
  match s:
    case "": True{}
    case s: False{}
def main() -> Bool: f("h")
