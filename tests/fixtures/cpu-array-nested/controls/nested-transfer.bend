# SPDX-License-Identifier: MPL-2.0
import Base

def text(a: String, b: String) -> String:
  String.append(a, b)

def render(a: Array<String>) -> String:
  match a:
    case ALeaf{x}: "[" ++ x ++ "]"
    case ANode{left, right}: render(left) ++ render(right)

def grouped(a: Array<Array<String>>) -> String:
  match a:
    case ALeaf{inner}: "{" ++ render(inner) ++ "}"
    case ANode{left, right}: grouped(left) ++ grouped(right)

def finish(r: Array<String> & String, outer: Array<Array<String>>) -> IO(Unit):
  (inner, old) = r
  do IO<Unit>:
    a : Unit <- IO.print("old:" ++ old)
    b : Unit <- IO.print("moved:" ++ render(inner))
    c : Unit <- IO.print("outer:" ++ grouped(outer))
    return Unit{}

def after_outer(r: Array<Array<String>> & Array<String>) -> IO(Unit):
  (outer, moved) = r
  replaced = Array.set(Array<String>, outer, 0, ALeaf{text("s", "et")})
  finish(Array.swap(String, moved, 4294967295, text("n", "ew")), replaced)

def observe(outer: Array<Array<String>>) -> IO(Unit):
  after_outer(Array.swap(Array<String>, outer, 4294967295, ALeaf{text("re", "placement")}))

def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- observe(ALeaf{ALeaf{text("le", "af")}})
    b : Unit <- observe(ANode{ALeaf{ALeaf{text("dis", "card")}}, ALeaf{ANode{ALeaf{text("le", "ft")}, ALeaf{text("ri", "ght")}}}})
    return Unit{}
