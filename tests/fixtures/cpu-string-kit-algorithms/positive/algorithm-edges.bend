# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/string_algorithms.bend as A

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(A.replace1("🙂éé", 1, 1, "X"))
    Unit <- IO.print(A.replace1("ab", 99, 1, "X"))
    Unit <- IO.print(A.sjoin("|", A.ssplit(44, ",a,,")))
    Unit <- IO.print(U32.show(A.occurs(0n, "z", "aaa")))
    Unit <- IO.print(U32.show(A.occurs(1n, "z", "aaa")))
    Unit <- IO.print(U32.show(A.occurs(2n, "aa", "aaa")))
    Unit <- IO.print(U32.show(A.occurs(3n, "aa", "aaa")))
    Unit <- IO.print(U32.show(A.occurs(3n, "", "")))
    IO.print(U32.show(A.occurs(3n, "é", "🙂éé")))
