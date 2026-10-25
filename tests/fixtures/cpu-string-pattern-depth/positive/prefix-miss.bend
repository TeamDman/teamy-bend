# SPDX-License-Identifier: MPL-2.0
import Base
def f(s: String) -> Bool:
  match s:
    case "h": True{}
    case s: False{}
def main() -> Bool: f("hi")
