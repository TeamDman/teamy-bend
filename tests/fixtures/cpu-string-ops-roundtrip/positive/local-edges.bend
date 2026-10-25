# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/string_ops.bend as S

def lengths(xs: List<&2, String>) -> String:
  List.show(~&2, ~String, ~(s => U32.show(S.slen(s))), xs)
def nul() -> String: SCon{Chr{0}, SNil{}}
def nul_text() -> String: "a" ++ nul() ++ nul() ++ "b"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(U32.show(S.slen("a🙂é")))
    Unit <- IO.print(U32.show(Maybe.default(&2, U32, S.sfind("", ""), 99)))
    Unit <- IO.print(U32.show(Maybe.default(&2, U32, S.sfind("ana", "banana"), 99)))
    Unit <- IO.print(U32.show(Maybe.default(&2, U32, S.sfind("é", "🙂éé"), 99)))
    Unit <- IO.print(U32.show(Maybe.default(&2, U32, S.sfind("z", ""), 99)))
    Unit <- IO.print(Bool.show(S.seq_(S.strim(" \t "), "\t")))
    Unit <- IO.print(Bool.show(String.eq(String.trim(" \t "), "")))
    Unit <- IO.print(S.supper("aéZ"))
    Unit <- IO.print("<" ++ S.stake(0n, "x") ++ ">")
    Unit <- IO.print(S.stake(9n, "🙂é"))
    Unit <- IO.print(S.sdrop(2n, "🙂éx"))
    Unit <- IO.print("<" ++ S.sdrop(9n, "🙂é") ++ ">")
    Unit <- IO.print(S.sjoin("|", S.ssplit(128578, "🙂é🙂🙂")))
    Unit <- IO.print(lengths(S.ssplit(128578, "🙂é🙂🙂")))
    IO.print(Bool.show(S.seq_(S.sjoin(nul(), S.ssplit(0, nul_text())), nul_text())))
