# SPDX-License-Identifier: MPL-2.0
import Base
import imports/defs.bend as D
def wrap(~x: U32) -> U32: D.pick(~x, 3)
def main() -> U32: wrap(~9)
