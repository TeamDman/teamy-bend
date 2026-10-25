# SPDX-License-Identifier: MPL-2.0
import Base
def main() -> IO(Unit): List.for_each(~&2, ~U32, ~IO.print, [1,2])
