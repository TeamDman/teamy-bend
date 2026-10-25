# SPDX-License-Identifier: MPL-2.0
import Base
import imports/defs.bend as D
def D.pick(~x: U32, y: U32) -> U32: 99
def main() -> U32: D.pick(~9, 3)
