# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/list_ops.bend as L
import ../original/list_slice.bend as S

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(S.u32s(L.lrange(0n)))
    Unit <- IO.print(S.u32s(L.lrange(1n)))
    Unit <- IO.print(S.u32s(L.lmsq([])))
    Unit <- IO.print(S.u32s(L.lfodd([0, 1, 2, 3, 4])))
    Unit <- IO.print(U32.show(L.lsum(L.lminc([]))))
    Unit <- IO.print(U32.show(L.lhead(&2, U32, 99, [])))
    Unit <- IO.print(S.u32s(L.ltail(&2, U32, [])))
    Unit <- IO.print(S.u32s(L.lset(&2, U32, 0n, 9, [])))
    Unit <- IO.print(S.u32s(L.lset(&2, U32, 3n, 9, [1, 2])))
    Unit <- IO.print(S.pairs(L.lzip(&2, U32, &2, U32, [1], [3, 4])))
    Unit <- IO.print(U32.show(L.lcount(2, [])))
    Unit <- IO.print(U32.show(L.lcount(2, [1, 3, 1])))
    Unit <- IO.print(Bool.show(L.lany4([])))
    Unit <- IO.print(Bool.show(L.lany4([4])))
    Unit <- IO.print(Bool.show(L.lall0([])))
    IO.print(Bool.show(L.lall0([0])))
