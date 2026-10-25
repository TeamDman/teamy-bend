# SPDX-License-Identifier: MPL-2.0
import Base
def suffix(t: String) -> Bool:
  match t:
    case "i": True{}
    case t: False{}
def prefix(k: String -> Bool, s: String) -> Bool:
  match s:
    case SCon{'h', tail}: k(tail)
    case s: False{}
def is_hi(s: String) -> Bool: False{}
def main() -> Bool: is_hi("hi")
def open(tail: String) -> Bool: is_hi(SCon{'h', tail})
