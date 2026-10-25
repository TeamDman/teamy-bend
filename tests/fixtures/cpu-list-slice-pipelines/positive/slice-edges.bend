# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/list_slice.bend as S

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(S.u32s(List.take(&2, U32, [1, 2], 0n)))
    Unit <- IO.print(S.u32s(List.drop(&2, U32, [1, 2], 0n)))
    Unit <- IO.print(S.u32s(List.drop(&2, U32, [1, 2], 2n)))
    Unit <- IO.print(S.u32s(List.set(&2, U32, [], 0n, 9)))
    Unit <- IO.print(S.u32s(List.set(&2, U32, [1, 2], 0n, 9)))
    Unit <- IO.print(S.mu(List.get(&2, U32, [1, 2], 0n)))
    Unit <- IO.print(S.mu(List.get(&2, U32, [], 0n)))
    Unit <- IO.print(S.pairs(List.zip(&2, U32, &2, U32, [], [3])))
    Unit <- IO.print(S.pairs(List.zip(&2, U32, &2, U32, [1], [3, 4])))
    Unit <- IO.print(S.pairs(List.zip(&2, U32, &2, U32, [1, 2], [])))
    Unit <- IO.print(S.nats(List.range(0n)))
    Unit <- IO.print(S.u32s(List.replicate(U32, 0n, 7)))
    Unit <- IO.print(S.u32s(List.concat(&2, U32, [[], [3], [], [4], []])))
    Unit <- IO.print(S.u32s(List.reverse(&2, U32, [])))
    Unit <- IO.print(S.u32s(List.tail(&2, U32, [])))
    IO.print(S.u32s(List.append(&2, U32, [], [])))
