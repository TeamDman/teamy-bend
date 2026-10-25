# SPDX-License-Identifier: MPL-2.0
import Base
import original/defs.bend as D
def D.local() -> U32: 9
def main() -> IO(Unit): IO.print(U32.show(D.local()))
