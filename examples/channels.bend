# SPDX-License-Identifier: MPL-2.0
import Base

def main() -> IO(Unit):
  do IO<Unit>:
    worker : Chan(U32) <- IO.fork(U32, IO.pure(U32, 42))
    result : U32 <- IO.join(U32, worker)
    IO.print(String.append("The worker returned ", U32.show(result)))
