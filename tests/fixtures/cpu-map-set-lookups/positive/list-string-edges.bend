# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/list_string_lookups.bend as L

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(U32.show(L.digit99(List.get(&2, U32, Nil{}, 0n))))
    Unit <- IO.print(U32.show(L.digit99(List.get(&2, U32, List.set(&2, U32, [7, 8], 2n, 99), 1n))))
    Unit <- IO.print(U32.show(L.digit99(List.get(&2, U32, List.set(&2, U32, Nil{}, 0n, 99), 0n))))
    Unit <- IO.print(U32.show(Maybe.default(&2, U32, L.sfind("", ""), 99)))
    Unit <- IO.print(U32.show(Maybe.default(&2, U32, L.sfind("é", "🙂éé"), 99)))
    Unit <- IO.print(U32.show(Maybe.default(&2, U32, L.sfind("éé", "é"), 99)))
    Unit <- IO.print(U32.show(Maybe.default(&2, U32, L.sfind("aba", "ababa"), 99)))
    Unit <- IO.print(Bool.show(L.seq(L.strim(SCon{Chr{160}, SNil{}}), SCon{Chr{160}, SNil{}})))
    IO.print(L.strim(String.append(SCon{Chr{13}, SCon{Chr{9}, SNil{}}}, String.append("é", SCon{Chr{10}, SNil{}}))))
