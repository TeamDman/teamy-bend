# SPDX-License-Identifier: MPL-2.0
# The existing native/JS zero-timer regression, used as a differential control.
import Base
def late(tag: String) -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(0)
    Unit <- IO.sleep(0)
    IO.print(tag)

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, late("a"))
    Unit <- IO.spawn(Unit, late("b"))
    Unit <- IO.spawn(Unit, IO.print("ready"))
    IO.print("main")
