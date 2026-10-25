# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/string_roundtrip.bend as R

def lengths(xs: List<&2, String>) -> String:
  List.show(~&2, ~String, ~(s => Nat.show(String.length(s))), xs)
def nul() -> String: SCon{Chr{0}, SNil{}}
def nul_text() -> String: "a" ++ nul() ++ nul() ++ "b"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(String.join(String.split("🙂é🙂🙂", '🙂'), "|"))
    Unit <- IO.print(Nat.show(List.length(&2, String, String.split("🙂é🙂🙂", '🙂'))))
    Unit <- IO.print(lengths(String.split("🙂é🙂🙂", '🙂')))
    Unit <- IO.print(Bool.show(String.eq(String.join(String.split("🙂é🙂🙂", '🙂'), "🙂"), "🙂é🙂🙂")))
    Unit <- IO.print(Bool.show(String.eq(String.join(String.split("", ','), ","), "")))
    Unit <- IO.print(Nat.show(List.length(&2, String, String.split("", ','))))
    Unit <- IO.print(Nat.show(String.length(String.append("🙂", "é"))))
    Unit <- IO.print(Nat.show(String.length(String.join(String.split(nul_text(), Chr{0}), nul()))))
    Unit <- IO.print(Bool.show(String.eq(String.join(String.split(nul_text(), Chr{0}), nul()), nul_text())))
    Unit <- IO.print("<" ++ String.join([], "🙂") ++ ">")
    Unit <- IO.print(Nat.show(List.length(&2, String, String.split(String.join([], "🙂"), '🙂'))))
    Unit <- IO.print(Bool.show(R.beq('🙂', '🙂')))
    IO.print(Bool.show(R.beq('🙂', 'é')))
