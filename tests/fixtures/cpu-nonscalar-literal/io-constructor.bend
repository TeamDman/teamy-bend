# SPDX-License-Identifier: MPL-2.0
import Base
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("before|")
    IO.print(SCon{Chr{55296}, SNil{}})
