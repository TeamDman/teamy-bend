# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/string_kit.bend as K

def lengths(xs: List<&2, String>) -> String:
  List.show(~&2, ~String, ~(s => Nat.show(String.length(s))), xs)
def nbsp() -> String: SCon{Chr{160}, SNil{}}
def nul() -> String: SCon{Chr{0}, SNil{}}
def nul_text() -> String: String.append("a", String.append(nul(), String.append(nul(), "b")))
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(Nat.show(String.length("🙂é")))
    Unit <- IO.print(String.reverse("a🙂é"))
    Unit <- IO.print(K.mc(String.get("🙂é", 1n)))
    Unit <- IO.print(lengths(String.split("", ',')))
    Unit <- IO.print(lengths(String.split(",a,,", ',')))
    Unit <- IO.print(lengths(String.lines("\r\n\n")))
    Unit <- IO.print(String.to_upper("aé🙂"))
    Unit <- IO.print(String.to_lower("ZÉ🙂"))
    Unit <- IO.print(Bool.show(Char.is_alpha('é')))
    Unit <- IO.print(Bool.show(Char.is_space(Chr{160})))
    Unit <- IO.print(Nat.show(String.length(String.trim(String.append(nbsp(), String.append(" a ", nbsp()))))))
    Unit <- IO.print("<" ++ String.repeat("🙂", 0n) ++ ">")
    Unit <- IO.print(lengths(String.split(nul_text(), Chr{0})))
    IO.print(Bool.show(String.eq(String.join(String.split(nul_text(), Chr{0}), nul()), nul_text())))
