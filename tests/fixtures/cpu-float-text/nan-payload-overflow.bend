# SPDX-License-Identifier: MPL-2.0
import Base
def bits(result: Maybe<&2, F32>) -> String:
  match result:
    case None{}: "none"
    case Some{x}: U32.show(F32.bits(x))
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(bits(F32.read("nan(18446744073709551615)")))
    Unit <- IO.print(bits(F32.read("nan(18446744073709551616)")))
    Unit <- IO.print(bits(F32.read("-nan(0x10000000000000000)")))
    IO.print(bits(F32.read("nan(18446744073709551616x)")))
