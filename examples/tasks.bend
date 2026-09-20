# SPDX-License-Identifier: MPL-2.0
import Base

def finish_later() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(10)
    IO.print("The spawned task has finished.")

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, finish_later())
    IO.print("Main is finished; the scheduler is waiting for its task.")
