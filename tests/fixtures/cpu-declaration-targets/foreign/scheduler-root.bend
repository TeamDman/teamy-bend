# SPDX-License-Identifier: MPL-2.0
import Base
def IO.spawn(-A: Type, act: IO(A)) -> IO(Unit): import "spawn.js"
def IO.sleep(ms: U32) -> IO(Unit): import "sleep.js"
def IO.now() -> IO(Nat): import "now.js"
